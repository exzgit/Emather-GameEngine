use wgpu;
use std::mem::size_of;
use bytemuck::{Pod, Zeroable, cast_slice};
#[allow(unused_imports)]
use std::ops::Range;
use wgpu::util::DeviceExt;

/// Vertex structure for 3D models
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    /// Creates a vertex buffer layout for the GPU
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Mesh raw data before creating GPU buffers
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_id: usize,
}

/// Mesh with GPU buffers
#[allow(unused)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub material_id: usize,
}

impl Mesh {
    /// Creates a new mesh from mesh data
    pub fn new(device: &wgpu::Device, mesh_data: &MeshData) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&mesh_data.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&mesh_data.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        Self {
            vertex_buffer,
            index_buffer,
            num_vertices: mesh_data.vertices.len() as u32,
            num_indices: mesh_data.indices.len() as u32,
            material_id: mesh_data.material_id,
        }
    }
    
    #[allow(dead_code)]
    /// Creates a simple quad mesh
    pub fn create_quad(device: &wgpu::Device, size: f32) -> Self {
        let half_size = size / 2.0;
        
        let vertices = vec![
            Vertex { position: [-half_size, -half_size, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [half_size, -half_size, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [half_size, half_size, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-half_size, half_size, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        let mesh_data = MeshData {
            vertices,
            indices,
            material_id: 0,
        };
        
        Self::new(device, &mesh_data)
    }
    
    /// Creates a simple cube mesh
    pub fn create_cube(device: &wgpu::Device, size: f32) -> Self {
        let half_size = size / 2.0;
        
        #[rustfmt::skip]
        let vertices = vec![
            // Front face
            Vertex { position: [-half_size, -half_size, half_size], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [half_size, -half_size, half_size], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [half_size, half_size, half_size], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [-half_size, half_size, half_size], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color: [1.0, 0.0, 0.0] },
            
            // Back face
            Vertex { position: [half_size, -half_size, -half_size], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [-half_size, -half_size, -half_size], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [-half_size, half_size, -half_size], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [half_size, half_size, -half_size], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0], color: [0.0, 1.0, 0.0] },
            
            // Top face
            Vertex { position: [-half_size, half_size, half_size], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [half_size, half_size, half_size], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [half_size, half_size, -half_size], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-half_size, half_size, -half_size], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], color: [0.0, 0.0, 1.0] },
            
            // Bottom face
            Vertex { position: [-half_size, -half_size, -half_size], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0, 1.0, 0.0] },
            Vertex { position: [half_size, -half_size, -half_size], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 0.0] },
            Vertex { position: [half_size, -half_size, half_size], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0, 1.0, 0.0] },
            Vertex { position: [-half_size, -half_size, half_size], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 0.0] },
            
            // Right face
            Vertex { position: [half_size, -half_size, half_size], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0, 0.0, 1.0] },
            Vertex { position: [half_size, -half_size, -half_size], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0, 0.0, 1.0] },
            Vertex { position: [half_size, half_size, -half_size], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0, 0.0, 1.0] },
            Vertex { position: [half_size, half_size, half_size], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0, 0.0, 1.0] },
            
            // Left face
            Vertex { position: [-half_size, -half_size, -half_size], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-half_size, -half_size, half_size], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-half_size, half_size, half_size], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-half_size, half_size, -half_size], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color: [0.0, 1.0, 1.0] },
        ];
        
        let indices = vec![
            0, 1, 2, 0, 2, 3,       // Front face
            4, 5, 6, 4, 6, 7,       // Back face
            8, 9, 10, 8, 10, 11,    // Top face
            12, 13, 14, 12, 14, 15, // Bottom face
            16, 17, 18, 16, 18, 19, // Right face
            20, 21, 22, 20, 22, 23, // Left face
        ];
        
        let mesh_data = MeshData {
            vertices,
            indices,
            material_id: 0,
        };
        
        Self::new(device, &mesh_data)
    }
    
    #[allow(dead_code)]
    /// Creates a sphere mesh
    pub fn create_sphere(device: &wgpu::Device, radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            for segment in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                
                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();
                
                let normal = [x/radius, y/radius, z/radius];
                let tex_coord = [
                    segment as f32 / segments as f32,
                    ring as f32 / rings as f32,
                ];
                
                vertices.push(Vertex {
                    position: [x, y, z],
                    normal,
                    tex_coords: tex_coord,
                    color: [1.0, 1.0, 1.0],
                });
            }
        }
        
        // Generate indices
        for ring in 0..rings {
            for segment in 0..segments {
                let first = ring * (segments + 1) + segment;
                let second = first + segments + 1;
                
                indices.push(first as u32);
                indices.push(second as u32);
                indices.push((first + 1) as u32);
                
                indices.push(second as u32);
                indices.push((second + 1) as u32);
                indices.push((first + 1) as u32);
            }
        }
        
        let mesh_data = MeshData {
            vertices,
            indices,
            material_id: 0,
        };
        
        Self::new(device, &mesh_data)
    }

    #[allow(unused)]
    pub fn create_custom_mesh(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u32>) {
        
    }

} 