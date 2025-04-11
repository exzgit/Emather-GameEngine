use wgpu;
use crate::engine::renderer::mesh::Vertex;

/// Pipeline configuration for rendering with required fields
#[derive(Debug)]
#[allow(dead_code)]
pub struct PipelineConfig {
    pub layout: Option<wgpu::PipelineLayout>,
    pub vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    pub shader: Option<wgpu::ShaderModule>,
    pub vertex_entry: String,
    pub fragment_entry: String,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub color_format: wgpu::TextureFormat,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub cull_mode: Option<wgpu::Face>,
    pub sample_count: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            layout: None, // Must be set before use
            vertex_layouts: vec![Vertex::desc()],
            shader: None, // Must be set before use
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_format: wgpu::TextureFormat::Bgra8UnormSrgb, // Default format
            depth_format: Some(wgpu::TextureFormat::Depth32Float),
            cull_mode: Some(wgpu::Face::Back),
            sample_count: 1,
        }
    }
}

#[allow(dead_code)]
/// Create a render pipeline from config
pub fn create_render_pipeline(
    device: &wgpu::Device,
    config: &PipelineConfig,
) -> wgpu::RenderPipeline {
    let color_target = Some(wgpu::ColorTargetState {
        format: config.color_format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    });
    
    let depth_stencil = config.depth_format.map(|format| wgpu::DepthStencilState {
        format,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: config.layout.as_ref(),
        vertex: wgpu::VertexState {
            module: config.shader.as_ref().unwrap(),
            entry_point: &config.vertex_entry,
            buffers: &config.vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: config.shader.as_ref().unwrap(),
            entry_point: &config.fragment_entry,
            targets: &[color_target],
        }),
        primitive: wgpu::PrimitiveState {
            topology: config.primitive_topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: config.cull_mode,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil,
        multisample: wgpu::MultisampleState {
            count: config.sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

#[allow(dead_code)]
/// Create a PBR pipeline with standard configuration
pub fn create_pbr_pipeline(
    device: &wgpu::Device,
    layout: wgpu::PipelineLayout, 
    shader: wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let mut config = PipelineConfig::default();
    config.layout = Some(layout);
    config.shader = Some(shader);
    config.color_format = format;
    
    create_render_pipeline(device, &config)
}

#[allow(dead_code)]
/// Create a simple pipeline for grid/helper objects
pub fn create_grid_pipeline(
    device: &wgpu::Device,
    layout: wgpu::PipelineLayout, 
    shader: wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let mut config = PipelineConfig::default();
    config.layout = Some(layout);
    config.shader = Some(shader);
    config.color_format = format;
    config.primitive_topology = wgpu::PrimitiveTopology::LineList;
    config.cull_mode = None;
    
    create_render_pipeline(device, &config)
} 