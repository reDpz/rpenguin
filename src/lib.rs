#[macro_use]
mod macros;

mod engine;
use engine::{
    camera::*,
    instance::*,
    mesh::Mesh,
    render_pipeline, texture,
    vert::{Vert, VertexBufferLayoutDescriptor},
    *,
};

use timer::InstantTimer;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferUsages, PresentMode, ShaderStages,
};

use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use cgmath::prelude::*;

use std::time::Instant;

const CURSOR_VISIBILITY: bool = false;

const NUM_INSTANCES_PER_ROW: u32 = 1000;
const NUM_INSTANCE_ROWS: u32 = 1000;
// gotta understand a bit about what this means
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

const TOPOLOGIES: [wgpu::PrimitiveTopology; 3] = [
    wgpu::PrimitiveTopology::TriangleList,
    wgpu::PrimitiveTopology::PointList,
    wgpu::PrimitiveTopology::LineList,
];

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("wgpu-testing")
        .with_active(true)
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    // lock the cursor
    window
        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
        .unwrap_or_else(|_| eprintln!("couldn't grab cursor"));
    window.set_cursor_visible(CURSOR_VISIBILITY);

    // create our meshes
    let (vertices, indices) = Mesh::to_vertex_indices(&[
        // Mesh::cube((-3.0, 0.0, 0.0), (0.5, 0.5, 0.5)),
        Mesh::cube((0.0, 0.0, 0.0), (0.5, 0.5, 0.5)),
        // Mesh::cube((3.0, 0.0, 0.0), (1.5, 1.5, 1.5)),
    ]);

    let mut state = State::new(&window, vertices, indices).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::Resized(new_size) => state.resize(*new_size),

                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => (),
                                // reconfiguring the surface recreates the swapchain
                                Err(wgpu::SurfaceError::Lost) => state.reconfigure_surface(),
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                Err(e) => eprintln!("{e:?}"),
                            }
                        }

                        _ => (),
                    }
                }
            }
            Event::DeviceEvent { ref event, .. } => {
                state.dinput(event);
            }

            Event::AboutToWait => {
                // Redraw requested will only happen if we request it
                state.window().request_redraw();
            }

            _ => {}
        })
        .unwrap();
}

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    clear_color: wgpu::Color,

    timer: InstantTimer,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    rp_builder: render_pipeline::RenderPipelineBuilder<'a>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    pub last_frame_draw: Instant,

    vertices: Vec<Vert>,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,

    indices: Vec<u16>,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    depth_texture: engine::texture::Texture,

    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,

    temp_index: usize,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window, vertices: Vec<Vert>, indices: Vec<u16>) -> State<'a> {
        let size = window.inner_size();

        // instance is like vulkan instance i assume
        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, // so this should be picking vulkan by default if it supports it
            ..Default::default()
        });

        // this is what we render to
        let surface = wgpu_instance.create_surface(window).unwrap();

        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // power preference is a struct containing priority values for GPUs, by default
                // its: Discrete, Integrated, None
                power_preference: wgpu::PowerPreference::default(),

                // We need to ensure that the device we pick is compatible with our window's
                // surface
                compatible_surface: Some(&surface),

                // if this is true we will be doing "Software rendering" i.e. not using the GPU
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // for now no features should be required
                    required_features: wgpu::Features::empty(),

                    // If using webgl consider using if cfg!
                    required_limits: wgpu::Limits::downlevel_defaults(), // using downlevel as instructed by docs
                    label: None,

                    // was not provided in the tutorial so I'm going to assume default is good
                    // enough
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Could not retrieve device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        println!(
            "Surface present modes: {:?}",
            surface_capabilities.present_modes
        );

        // assuming srgb surface texture, if it's not srgb then it will come out darker
        // With vulkano we were using rgb rather than srgb by default
        let surface_format = surface_capabilities
            .formats
            .iter()
            // get the first srgb surface texture format
            .find(|f| !f.is_srgb())
            .copied()
            // Option could be none if there are no srgb texture formats so just pick the first
            .unwrap_or(surface_capabilities.formats[0]);

        // dictates how surface textures will be created
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // prefer no vsync
            present_mode: PresentMode::AutoNoVsync,
            // even the tutorial writer has no clue what this is
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            // wonder what will happen if i just set this to 0 lol
            desired_maximum_frame_latency: 2,
        };

        // little helpful line for debugging on end user
        println!(
            "Using \"{}\" as primary gpu ({:?})",
            adapter.get_info().name,
            adapter.get_info().device_type
        );

        // Tutorial didn't say to configure the surface however it's necessary
        surface.configure(&device, &config);

        /*--------------------- TEXTURES ---------------------*/

        let diffuse_bytes = include_bytes!(crate_path!("assets/images/lunaistabby-cat.png"));
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            Some("cat"),
            wgpu::AddressMode::ClampToEdge,
        )
        .unwrap(); // teehee

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        // this needs to be the same as "Texture entry above", what does that mean
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        // depth texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, Some("Depth Texture"));

        /*--------------------- Instances ---------------------*/

        // so this is the "rust way" of iterating twice
        let instances = (0..NUM_INSTANCE_ROWS)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        // le buffer
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        /*--------------------- CAMERA ---------------------*/

        let camera = Camera {
            // 1 unit up, 2 units back position
            // +z is outside screen
            eye_pos: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, -1.0).into(),

            // literally what you think it is
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(500.0);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        /*--------------------- SHADERS ---------------------*/

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!(crate_path!("assets/shaders/shader.wgsl")).into(),
            ),
        });

        let vertex_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        };
        let vertex_buffer = device.create_buffer_init(&vertex_descriptor);

        let index_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        };

        let index_buffer = device.create_buffer_init(&index_descriptor);

        let rp_builder = render_pipeline::RenderPipelineBuilder::new(
            &device,
            render_pipeline::ShaderCollection {
                shaders: vec![shader],
                ..Default::default()
            },
            vec![Vert::desc(), InstanceRaw::desc()],
            vec![texture_bind_group_layout, camera_bind_group_layout],
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
        );

        let vertex_count = vertices.len() as u32;
        let index_count = indices.len() as u32;

        // helpful timer
        let timer = InstantTimer::from_secs_f32(0.5);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },

            timer,

            instances,
            instance_buffer,

            rp_builder,

            render_pipeline: None,
            last_frame_draw: Instant::now(),

            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            vertices,
            vertex_buffer,
            vertex_count,

            indices,
            index_buffer,
            index_count,

            depth_texture,

            diffuse_bind_group,
            diffuse_texture,

            temp_index: 0,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // some error checking to ensure we dont set it to invalid sizes
        if new_size.width > 0 && new_size.height > 0 {
            // update our stuff
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure_surface();

            // update camera
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;

            // update depth texture
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                Some("Depth Texture"),
            );
        }
    }

    fn reconfigure_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    fn dinput(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera_controller.process_mouse(delta);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => 'change: {
                /* if position.x < 0.0 || position.y < 0.0 {
                    break 'change;
                } */
                // we are goin to change the clear color with relative positions, this would
                // essentially show the UV color of the screen at the position the mouse is at.
                // we want to shift the entire thing to the center
                // The usual procedure is to multiply by 2 and then - 1

                /*              let x = (position.x as f32 / self.size.width as f32) * 2.0 - 1.0;
                // negate Y because that's how it is
                let y = -((position.y as f32 / self.size.height as f32) * 2.0 - 1.0);

                self.vertices[0].position = [x, y, 0.0];

                self.recreate_vertex_buffer_from_self(); */
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                self.window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap_or_else(|_| eprintln!("couldn't let go of cursor"));
                self.window.set_cursor_visible(true);
                self.camera_controller.enabled = false;
            }
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.window
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .unwrap_or_else(|_| eprintln!("couldn't grab cursor"));
                self.window.set_cursor_visible(CURSOR_VISIBILITY);
                self.camera_controller.enabled = true;
            }

            // WindowEvent::Focused(true) => {}
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    ..
                } => {
                    self.temp_index = (self.temp_index + 1) % 3;
                    // self.swap_texture(CUTE_CAT_IMAGES[self.temp_index]);
                    self.rp_builder.topology = TOPOLOGIES[self.temp_index];
                }
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::KeyI),
                    ..
                } => println!("hello world (why not)"),

                _ => (),
            },
            _ => (),
        }
        // temp
        self.camera_controller.process_input(event)
    }

    fn update(&mut self) {
        // printfps, why not?
        // printfps(self.get_frame_delta());
        let delta = self.get_frame_delta();

        // update camera
        self.camera_controller
            .update_camera(&mut self.camera, delta);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        if self.timer.tick_reset() {
            printfps(delta);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        self.ensure_pipeline();
        // not even remotely sure about what the borrow checker is doing here
        let pipeline = self.render_pipeline.as_ref().unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                // this is what @location(0) in fs shader targets
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(pipeline);

            // textures
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

            // camera
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // so just our vertices
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // render_pass.draw(0..self.vertex_count, 0..1);
            render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as _);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.last_frame_draw = Instant::now();

        Ok(())
    }

    fn ensure_pipeline(&mut self) {
        if self.render_pipeline.is_none() {
            // build pipeline then just send it back
            self.render_pipeline = Some(self.rp_builder.build(&self.device));
        }
    }

    fn get_pipeline(&mut self) -> &wgpu::RenderPipeline {
        self.ensure_pipeline();
        self.render_pipeline.as_ref().unwrap()
    }

    fn rebuild_pipeline(&mut self) {
        self.render_pipeline = Some(self.rp_builder.build(&self.device));
    }

    fn recreate_vertex_buffer(&mut self) {
        // there should be a better way of doing this
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: BufferUsages::VERTEX,
        };
        self.vertex_buffer = self.device.create_buffer_init(&descriptor);

        self.vertex_count = self.vertices.len() as u32;
    }

    fn get_frame_delta(&self) -> f32 {
        self.last_frame_draw.elapsed().as_secs_f32()
    }

    fn get_frame_delta_f64(&self) -> f64 {
        self.last_frame_draw.elapsed().as_secs_f64()
    }

    fn swap_texture(&mut self, image_bytes: &[u8]) {
        self.diffuse_texture = texture::Texture::from_bytes(
            &self.device,
            &self.queue,
            image_bytes,
            None,
            wgpu::AddressMode::ClampToEdge,
        )
        .unwrap();

        self.diffuse_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &self.rp_builder.bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
                },
            ],
        });
    }
}

/* fn distance_v2(point_one: &(f32, f32), point_two: &(f32, f32)) -> f32 {
    ((point_one.0 - point_two.0).powi(2) + (point_one.1 - point_two.1).powi(2)).sqrt()
} */

fn printfps(delta: f32) {
    let fps = 1.0 / delta;

    print!("{}[2J", 27 as char);
    println!("{} FPS", fps as i32)
}
