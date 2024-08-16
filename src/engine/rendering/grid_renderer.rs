// TODO: add fields for customization
use crate::{render_pipeline::RenderPipelineBuilder, wgpu};
pub struct GridRenderer {}

impl GridRenderer {
    pub fn draw(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
        view: &wgpu::TextureView,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
    }
}
