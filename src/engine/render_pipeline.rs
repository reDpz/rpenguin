pub struct RenderPipelineBuilder<'a> {
    pub sc: ShaderCollection,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub render_pipeline_layout: wgpu::PipelineLayout,
    render_targets: Vec<Option<wgpu::ColorTargetState>>,
    pub topology: wgpu::PrimitiveTopology,
    pub v_buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,

    pub label: String,
    pub front_face: wgpu::FrontFace,
    pub cull_mode: Option<wgpu::Face>,
    pub polygon_mode: wgpu::PolygonMode,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        device: &wgpu::Device,
        shader_collection: ShaderCollection,
        vertex_buffer_descriptors: Vec<wgpu::VertexBufferLayout<'a>>,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        surface_format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let ref_layouts: Vec<&wgpu::BindGroupLayout> = bind_group_layouts.iter().collect();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &ref_layouts,
                push_constant_ranges: &[], // enable FEATURES::PUSH_CONSTANTS if you use this
            });

        let render_targets = vec![Some(wgpu::ColorTargetState {
            format: surface_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        Self {
            sc: shader_collection,
            bind_group_layouts,
            render_pipeline_layout,
            render_targets,
            topology,
            v_buffers: vertex_buffer_descriptors,
            depth_stencil,

            label: "Render Pipeline".to_string(),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }

    pub fn set_bind_group_layouts(
        &mut self,
        device: &wgpu::Device,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    ) {
        self.bind_group_layouts = bind_group_layouts;
        let ref_layouts: Vec<&wgpu::BindGroupLayout> = self.bind_group_layouts.iter().collect();
        self.render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline layout"),
                bind_group_layouts: &ref_layouts,
                push_constant_ranges: &[],
            });
    }

    pub fn build(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: Some(&self.render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.sc.shaders[self.sc.vert_index],
                entry_point: &self.sc.vert_entry,
                buffers: &self.v_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.sc.shaders[self.sc.frag_index],
                entry_point: &self.sc.frag_entry,
                targets: &self.render_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: self.front_face,
                // it's literally this easy
                cull_mode: self.cull_mode,
                // Any other value REQUIRES Features::NON_FILL_POLYGON_MODE
                polygon_mode: self.polygon_mode,

                // REQUIRES Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
                // REQUIRES Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
            },
            // FIXME: do it without cloning
            depth_stencil: self.depth_stencil.clone(),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub fn set_blend_mode(&mut self, index: usize, mode: Option<wgpu::BlendState>) {
        todo!()
        // self.render_targets[index].unwrap().blend = mode;
    }
}

pub struct ShaderCollection {
    /// All the shaders
    pub shaders: Vec<wgpu::ShaderModule>,
    /// The fragment function
    pub frag_entry: String,
    /// The index of the shader where the function is located
    pub frag_index: usize,
    /// The vertex function
    pub vert_entry: String,
    /// The index of the shader where the function is located
    pub vert_index: usize,
}

impl Default for ShaderCollection {
    /// WARN: you will need to specify at least one shader module manually otherwise this will panic
    fn default() -> Self {
        Self {
            shaders: Vec::new(),
            frag_entry: "fs_main".to_string(),
            frag_index: 0,
            vert_entry: "vs_main".to_string(),
            vert_index: 0,
        }
    }
}
