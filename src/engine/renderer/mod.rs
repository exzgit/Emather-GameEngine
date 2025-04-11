pub mod texture;
pub mod mesh;
mod pipeline;
mod shader;

use wgpu;
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::engine::scene::Model;
use cgmath::SquareMatrix;

// Re-export key structs
pub use texture::Texture;
pub use mesh::{Mesh, Vertex, MeshData};
#[allow(unused_imports)]
pub use shader::ShaderManager;

#[allow(dead_code)]
/// Create a debugging material for grids and gizmos
fn create_debug_material(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> crate::engine::scene::MaterialInstance {
    // Create a white texture for the debug material
    let white_texture = texture::Texture::create_colored_texture(
        device,
        queue,
        [1.0, 1.0, 1.0],
        "Debug Material Texture"
    );
    
    // Create a uniform buffer for transform matrices
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Debug Uniform Buffer"),
        size: 128, // Space for matrices
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    // Create a light buffer (unused but required by the shader)
    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Debug Light Buffer"),
        size: 64, // Space for light data
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    // Create the bind group for this material
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Debug Bind Group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&white_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&white_texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: light_buffer.as_entire_binding(),
            },
        ],
    });
    
    crate::engine::scene::MaterialInstance {
        material_id: 0,
        diffuse_bind_group: bind_group,
    }
}

#[allow(dead_code)]
/// Main renderer that handles all rendering operations
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    depth_texture: texture::Texture,
    models: Vec<Model>,
    grid_model: Option<Model>,  // Separate grid model
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    grid_pipeline: Option<wgpu::RenderPipeline>,  // Specialized pipeline for grid
    shader_manager: shader::ShaderManager,
    multisampled_framebuffer: Option<wgpu::TextureView>,
    sample_count: u32,
}

impl Renderer {
    /// Creates a new renderer
    pub async fn new(window: &Window) -> Self {
        // Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // Surface
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        
        // Adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions { 
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        ).await.expect("Failed to request adapter");

        // Device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.expect("Failed to create device");

        // Surface configuration
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![format],
        };

        surface.configure(&device, &config);
        
        // Create depth texture
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "Depth Texture");
        
        // Create a combined bind group layout for model rendering
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Combined Bind Group Layout"),
            entries: &[
                // Uniform buffer (matrix transforms)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Light buffer (for sun light)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create the shader manager
        let shader_manager = shader::ShaderManager::new(&device);
        
        // Get default shader module for rendering
        let shader_module = shader_manager.get("pbr").expect("PBR shader should be available");
        
        // Create the pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Create the render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &[mesh::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create a specialized shader for the grid
        let grid_shader_src = r#"
            struct Uniforms {
                view_proj: mat4x4<f32>,
                model: mat4x4<f32>,
            };

            @group(0) @binding(0)
            var<uniform> uniforms: Uniforms;

            struct VertexInput {
                @location(0) position: vec3<f32>,
                @location(1) normal: vec3<f32>,
                @location(2) tex_coords: vec2<f32>,
                @location(3) color: vec3<f32>,
            };

            struct VertexOutput {
                @builtin(position) clip_position: vec4<f32>,
                @location(0) color: vec3<f32>,
            };

            @vertex
            fn vs_main(in: VertexInput) -> VertexOutput {
                var out: VertexOutput;
                out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(in.position, 1.0);
                out.color = in.color;
                return out;
            }

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return vec4<f32>(in.color, 1.0);
            }
        "#;
        
        // Create grid shader module
        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(grid_shader_src)),
        });
        
        // Create grid pipeline
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: "vs_main",
                buffers: &[mesh::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let mut renderer = Self {
            device,
            queue,
            surface,
            config,
            adapter,
            depth_texture,
            models: Vec::new(),
            grid_model: None,
            bind_group_layout,
            render_pipeline,
            grid_pipeline: Some(grid_pipeline),
            shader_manager,
            multisampled_framebuffer: None,
            sample_count: 1,
        };
        
        // Initialize debug visualization (grid and axis gizmos)
        renderer.init_debug_visualization();
        
        renderer
    }
    
    /// Resizes the renderer surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // Update dimensions
            self.config.width = width;
            self.config.height = height;
            
            // Configure the surface with our updated config
            self.surface.configure(&self.device, &self.config);
            
            // Create a new depth texture with the updated dimensions
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device, 
                &self.config, 
                "Depth Texture"
            );
        }
    }
    
    /// Returns a reference to the device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    
    /// Returns a reference to the adapter
    #[allow(unused)]
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
    
    /// Returns a reference to the queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    
    /// Returns a reference to the bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
    
    #[allow(dead_code)]
    /// Returns a reference to the depth texture
    pub fn depth_texture(&self) -> &texture::Texture {
        &self.depth_texture
    }
    
    /// Get a reference to a model by index
    pub fn get_model(&self, index: usize) -> Option<&Model> {
        self.models.get(index)
    }
    
    /// Add a model to the renderer
    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }
    
    /// Render the scene with the current camera
    pub fn render_scene(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface
            .get_current_texture()
            .map_err(|e| {
                // If we lost the surface, try to reconfigure it
                if e == wgpu::SurfaceError::Lost {
                    self.resize(self.config.width, self.config.height);
                }
                e
            })?;

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Draw all models
            for model in self.models.iter() {
                model.draw(&mut render_pass);
            }
        }

        // Submit the work
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
    
    /// Initialize grid and axis gizmos (helper objects)
    fn init_debug_visualization(&mut self) {
        // Create grid mesh
        let grid_size = 20;
        let grid_spacing = 1.0;
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // X lines
        for i in -grid_size..=grid_size {
            let z = i as f32 * grid_spacing;
            let start_idx = vertices.len() as u32;
            
            // Color: brighter to be more visible
            let color = if i == 0 {
                [1.0, 1.0, 1.0] // White for center line (Z-axis)
            } else {
                [0.5, 0.5, 0.5] // Brighter grey for regular grid line
            };
            
            vertices.push(Vertex {
                position: [-grid_size as f32 * grid_spacing, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                color,
            });
            
            vertices.push(Vertex {
                position: [grid_size as f32 * grid_spacing, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
                color,
            });
            
            indices.push(start_idx);
            indices.push(start_idx + 1);
        }
        
        // Z lines
        for i in -grid_size..=grid_size {
            let x = i as f32 * grid_spacing;
            let start_idx = vertices.len() as u32;
            
            // Color: brighter to be more visible
            let color = if i == 0 {
                [1.0, 1.0, 1.0] // White for center line (X-axis)
            } else {
                [0.5, 0.5, 0.5] // Brighter grey for regular grid line
            };
            
            vertices.push(Vertex {
                position: [x, 0.0, -grid_size as f32 * grid_spacing],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                color,
            });
            
            vertices.push(Vertex {
                position: [x, 0.0, grid_size as f32 * grid_spacing],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
                color,
            });
            
            indices.push(start_idx);
            indices.push(start_idx + 1);
        }
        
        // Add axis lines (XYZ gizmo)
        // X axis (red) - made thicker/longer
        let start_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            color: [1.0, 0.0, 0.0], // Red for X
        });
        
        vertices.push(Vertex {
            position: [10.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            color: [1.0, 0.0, 0.0], // Red for X
        });
        
        indices.push(start_idx);
        indices.push(start_idx + 1);
        
        // Y axis (green) - made thicker/longer
        let start_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            color: [0.0, 1.0, 0.0], // Green for Y
        });
        
        vertices.push(Vertex {
            position: [0.0, 10.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            color: [0.0, 1.0, 0.0], // Green for Y
        });
        
        indices.push(start_idx);
        indices.push(start_idx + 1);
        
        // Z axis (blue) - made thicker/longer
        let start_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            color: [0.0, 0.0, 1.0], // Blue for Z
        });
        
        vertices.push(Vertex {
            position: [0.0, 0.0, 10.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            color: [0.0, 0.0, 1.0], // Blue for Z
        });
        
        indices.push(start_idx);
        indices.push(start_idx + 1);
        
        // Create mesh data
        let mesh_data = mesh::MeshData {
            vertices,
            indices,
            material_id: 0,
        };
        
        // Create the grid mesh
        let grid_mesh = mesh::Mesh::new(self.device(), &mesh_data);
        
        // Create a uniform buffer for the grid transform
        let uniform_data = crate::engine::core::application::Uniforms {
            view_proj: cgmath::Matrix4::identity().into(),
            model: cgmath::Matrix4::identity().into(),
        };
        
        let uniform_buffer = self.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Grid Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        // Create a light buffer for the grid
        let mut sun_light = crate::engine::scene::SunLight::new();
        sun_light.direction = [0.0, -1.0, 0.0];
        sun_light.color = [1.0, 1.0, 1.0];
        sun_light.intensity = 0.7;
        
        let light_buffer = self.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Grid Light Buffer"),
                contents: bytemuck::cast_slice(&[sun_light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        // Create a white texture for the grid material
        let white_texture = texture::Texture::create_colored_texture(
            self.device(),
            self.queue(),
            [1.0, 1.0, 1.0],
            "Grid Material Texture"
        );
        
        // Create bind group
        let bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&white_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&white_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create material instance
        let material = crate::engine::scene::model::MaterialInstance {
            material_id: 0,
            diffuse_bind_group: bind_group,
        };
        
        // Create the grid model
        let grid_model = Model {
            meshes: vec![grid_mesh],
            materials: vec![material],
        };
        
        // Store the grid model separately
        self.grid_model = Some(grid_model);
    }
} 