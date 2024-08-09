#[macro_use]
mod macros;

mod engine;
use engine::prelude::*;

use timer::InstantTimer;
use vert::BasicVertex;
use wgpu::util::{DeviceExt, RenderEncoder};

use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

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
    let vertices = BasicVertex::DEFAULT_TRIANGLE;

    let mut state = State::new(&window, vertices.to_vec()).await;

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

pub struct State<'a, V: VertexBufferLayoutDescriptor + bytemuck::Pod> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,

    vertices: Vec<V>,
}

impl<'a, V: VertexBufferLayoutDescriptor + bytemuck::Pod> State<'a, V> {
    async fn new(window: &'a Window, vertices: Vec<V>) -> State<'a, V> {
        todo!()
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
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
