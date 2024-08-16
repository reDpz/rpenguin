#[macro_use]
mod macros;

mod engine;
use engine::{egui_tools, prelude::*};

mod particle;

use egui_wgpu::wgpu::util::DeviceExt;
use egui_wgpu::{wgpu, ScreenDescriptor};
use glam::Vec3Swizzles;
use particle::simulation::{NBodySimulation, ParticleInstance};
use render_pipeline::RenderPipelineBuilder;

use winit::keyboard::KeyCode;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use std::time::Instant;

/* const TOPOLOGIES: [wgpu::PrimitiveTopology; 3] = [
    wgpu::PrimitiveTopology::TriangleList,
    wgpu::PrimitiveTopology::PointList,
    wgpu::PrimitiveTopology::LineList,
]; */

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RPenguin")
        .with_active(true)
        // .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    // lock the cursor
    /* window
    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
    .unwrap_or_else(|_| eprintln!("couldn't grab cursor")); */
    // window.set_cursor_visible(CURSOR_VISIBILITY);

    // create our meshes

    let mut state = State::new(&window).await;

    let mut last_render_time = Instant::now();
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
                            let now = Instant::now();
                            let delta = (now - last_render_time).as_secs_f32();
                            state.update(delta);
                            last_render_time = now;
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
    clear_color: wgpu::Color,
    egui_renderer: egui_tools::EguiRenderer,

    nbody_simulation: NBodySimulation,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    mouse_position: glam::Vec2,

    camera: Camera2D,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    camera_controller: CameraController2D,

    pipeline_builder: render_pipeline::RenderPipelineBuilder<'a>,
    pipeline: wgpu::RenderPipeline,

    instances: Vec<ParticleInstance>,
    instance_buffer: wgpu::Buffer,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = wgpu_instance.create_surface(window).unwrap();

        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                // use cpu/software rendering
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    // NOTE: This is where you add features
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Could not find a device, are you sure your GPU supports VULKAN?");

        let surface_capabilities = surface.get_capabilities(&adapter);

        // i dont want srgb
        let surface_format: wgpu::TextureFormat = surface_capabilities
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            // this should never be the case
            .expect("Surface does not support RGB");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            // prerendered frames, i think
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        /* ----------------- EGUI ----------------- */

        let egui_renderer = egui_tools::EguiRenderer::new(&device, config.format, None, 1, &window);

        /* ----------------- N BODY SIMULATION ----------------- */

        let nbody_simulation = create_simulation();

        /* ----------------- CAMERA ----------------- */

        let mut camera = Camera2D::new(size.width as f32 / size.height as f32);
        camera.zoom = 10.0;
        /* {
            let center = nbody_simulation.center();
            camera.position = glam::Vec3::new(center.x, center.y, 0.0);
        } */
        camera.update_projection_matrix();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera projection matrix"),
            contents: bytemuck::cast_slice(&[camera.proj]),
            // you need to enable bytemuck feature in glam to do this ^^^^^^^^^
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            // this is obviously a uniform ^^^  this is required to copy to it ^
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

        let camera_controller = CameraController2D::new(6.0);

        /* ----------------- SHADERS ----------------- */

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!(crate_path!("assets/shaders/shader.wgsl")).into(),
            ),
        });

        /* ----------------- VERTEX BUFFER ----------------- */

        /* let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }); */

        /* ----------------- INSTANCE BUFFER ----------------- */

        let instances = nbody_simulation.instances();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        /* ----------------- RENDER PIPELINE ----------------- */

        let pipeline_builder = RenderPipelineBuilder::new(
            &device,
            render_pipeline::ShaderCollection {
                shaders: vec![shader],
                ..Default::default()
            },
            vec![ParticleInstance::desc()],
            vec![camera_bind_group_layout],
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            None,
        );

        let pipeline = pipeline_builder.build(&device);

        // before initializing the surface should be configured
        surface.configure(&device, &config);

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            egui_renderer,

            nbody_simulation,

            pipeline_builder,
            pipeline,

            camera,
            camera_buffer,
            camera_bind_group,

            camera_controller,

            // The window must be declared after the surface so
            // it gets dropped after it as the surface contains
            // unsafe references to the window's resources.
            window,
            mouse_position: glam::Vec2::ZERO,

            instances,
            instance_buffer,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            self.config.height = new_size.height;
            self.config.width = new_size.width;
            self.reconfigure_surface();

            // camera
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn dinput(&mut self, event: &DeviceEvent) {
        //
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.egui_renderer.handle_input(self.window, event);

        self.camera_controller.input(event);
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                if state.is_pressed() {
                    match keycode {
                        KeyCode::KeyR => {
                            self.nbody_simulation = create_simulation();
                        }
                        KeyCode::Space => {
                            self.nbody_simulation.is_running = !self.nbody_simulation.is_running;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position.x = position.x as f32;
                self.mouse_position.y = position.y as f32;
            }
            _ => (),
        }
        false
    }

    // TODO: delta is behaving oddly
    fn update(&mut self, delta: f32) {
        // let delta = self.last_render.elapsed().as_secs_f32();
        // let delta = 1.0 / 30.0;
        // println!("delta: {delta}");
        // printfps(delta);

        self.camera.update_projection_matrix();
        self.camera_controller.process(&mut self.camera, delta);

        self.nbody_simulation.update(delta);
        self.instances = self.nbody_simulation.instances();
        self.recreate_instance_buffer();

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.proj]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

            render_pass.draw(0..3, 0..self.instances.len() as u32);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };

        self.egui_renderer.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            self.window,
            &view,
            &screen_descriptor,
            |ctx| {
                egui::Window::new("Info")
                    .resizable(true)
                    .vscroll(true)
                    .default_open(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Cursor: ");
                            ui.label(format!(
                                "{:.2}, {:.2}",
                                self.mouse_position.x, self.mouse_position.y
                            ));
                        });

                        // TODO: figure out how to convert from screenspace to world space
                        let world_pos = self.mouse_position;

                        ui.horizontal(|ui| {
                            ui.label("World Position:");
                            // camera projection matrix translates world space to screen space, we
                            // want to go from screen space to world space
                            ui.label(format!("x:{:.2}, y:{:.2}", world_pos.x, -world_pos.y))
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            let particle = self.nbody_simulation.particle_at(world_pos);
                        })
                    });
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        // self.last_render = Instant::now();

        Ok(())
    }

    fn recreate_instance_buffer(&mut self) {
        self.instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&self.instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
    }

    #[inline]
    fn rebuild_pipeline(&mut self) {
        self.pipeline = self.pipeline_builder.build(&self.device);
    }

    #[inline]
    fn reconfigure_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
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

fn create_simulation() -> NBodySimulation {
    let area = 10.0;
    NBodySimulation::rand_distribute(
        glam::Vec2 { x: area, y: area },
        glam::Vec2 { x: -area, y: -area },
        3,
    )
}
