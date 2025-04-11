use std::path::PathBuf;
use wgpu;
use crate::engine::renderer::mesh::Mesh;

/// Material data for rendering
#[derive(Debug)]
#[allow(dead_code)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<PathBuf>,
    pub normal_texture: Option<PathBuf>,
    pub metallic_texture: Option<PathBuf>,
    pub roughness_texture: Option<PathBuf>,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub shininess: f32,
}

#[allow(dead_code)]
/// Material instance with GPU resources
pub struct MaterialInstance {
    pub material_id: usize,
    pub diffuse_bind_group: wgpu::BindGroup,
}

/// 3D Model with meshes and materials
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<MaterialInstance>,
}

#[allow(dead_code)]
impl Model {
    /// Create a new empty model
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
        }
    }
    
    /// Add a mesh to the model
    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }
    
    /// Add a material instance to the model
    pub fn add_material(&mut self, material: MaterialInstance) {
        self.materials.push(material);
    }
    
    /// Draw the model with the given render pass
    pub fn draw<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) 
    where 'a: 'b {
        for mesh in &self.meshes {
            let material = &self.materials[mesh.material_id];
            
            render_pass.set_bind_group(0, &material.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }
} 