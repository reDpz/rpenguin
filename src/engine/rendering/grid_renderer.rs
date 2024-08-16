// TODO: add fields for customization
use crate::{
    render_pipeline::{RenderPipelineBuilder, ShaderCollection},
    wgpu,
};
pub struct GridRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl GridRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // yes this will add some miliseconds of overhead at worst
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!(crate_path!("assets/shaders/engine/grid.wgsl")).into(),
            ),
        });

        let shader_collection = ShaderCollection {
            shaders: vec![shader],
            ..Default::default()
        };

        let builder = RenderPipelineBuilder::new(
            device,
            shader_collection,
            vec![],
            // you'll need to add the buffer here
            vec![],
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            None,
        );

        let render_pipeline = builder.build(device);

        Self { render_pipeline }
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        // screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Grid"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.draw(0..6, 0..1);
        }
    }
}
