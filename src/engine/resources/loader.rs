use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use crate::engine::renderer::{Texture, Mesh, MeshData, Vertex};
use crate::engine::scene::{Model, Material};
use crate::engine::scene::model::MaterialInstance;
#[allow(unused_imports)]
use wgpu::{self, util::DeviceExt};
#[allow(unused_imports)]
use image;
use tobj;

/// Trait for loading resources
#[allow(dead_code)]
pub trait ResourceLoader<T> {
    /// Load a resource from a path
    fn load(&self, path: &Path) -> Result<T>;
}

/// Texture loader for loading textures
pub struct TextureLoader<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

#[allow(dead_code)]
impl<'a> TextureLoader<'a> {
    /// Create a new texture loader
    pub fn new(device: &'a wgpu::Device, queue: &'a wgpu::Queue) -> Self {
        Self { device, queue }
    }
    
    /// Load a texture from a file
    pub fn load_from_file(&self, path: &Path) -> Result<Texture> {
        let bytes = fs::read(path)?;
        Texture::from_bytes(self.device, self.queue, &bytes, &path.to_string_lossy())
    }
    
    /// Create a default texture for missing textures
    pub fn create_default(&self) -> Texture {
        Texture::create_default_texture(self.device, self.queue)
    }
    
    /// Create a colored texture from a color
    pub fn create_colored(&self, color: [f32; 3], label: &str) -> Texture {
        Texture::create_colored_texture(self.device, self.queue, color, label)
    }
}

impl<'a> ResourceLoader<Texture> for TextureLoader<'a> {
    fn load(&self, path: &Path) -> Result<Texture> {
        self.load_from_file(path)
    }
}

/// Helper function to find PBR textures
fn find_pbr_texture(base_path: &Path, texture_type: &str, material_name: &str) -> Option<PathBuf> {
    // Common suffixes for PBR textures
    let suffixes = match texture_type {
        "normal" => vec!["_normal", "_nrm", "_n", "-normal", "-nrm", "-n", "_Normal"],
        "metallic" => vec!["_metallic", "_metalness", "_metal", "_m", "-metallic", "-m", "_Metallic"],
        "roughness" => vec!["_roughness", "_rough", "_r", "-roughness", "-r", "_Roughness"],
        _ => vec![],
    };
    
    // Check if base directory exists
    let base_dir = base_path.parent().unwrap_or(Path::new(""));
    if !base_dir.exists() {
        return None;
    }
    
    // Try each suffix
    for suffix in suffixes {
        // Try with material name
        let material_base = material_name.split_whitespace().next().unwrap_or(material_name);
        let test_path = base_dir.join(format!("{}{}.png", material_base, suffix));
        if test_path.exists() {
            return Some(test_path);
        }
        
        // Try with base file name
        if let Some(base_name) = base_path.file_stem() {
            if let Some(base_str) = base_name.to_str() {
                let test_path = base_dir.join(format!("{}{}.png", base_str, suffix));
                if test_path.exists() {
                    return Some(test_path);
                }
                
                // Try with jpg extension
                let test_path = base_dir.join(format!("{}{}.jpg", base_str, suffix));
                if test_path.exists() {
                    return Some(test_path);
                }
            }
        }
    }
    
    None
}

#[allow(dead_code)]
/// Model loader for loading 3D models
pub struct ModelLoader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    texture_loader: TextureLoader<'a>,
    bind_group_layout: &'a wgpu::BindGroupLayout,
}

impl<'a> ModelLoader<'a> {
    /// Create a new model loader
    pub fn new(
        device: &'a wgpu::Device, 
        queue: &'a wgpu::Queue, 
        bind_group_layout: &'a wgpu::BindGroupLayout
    ) -> Self {
        let texture_loader = TextureLoader::new(device, queue);
        Self { 
            device, 
            queue,
            texture_loader,
            bind_group_layout,
        }
    }
    
    /// Load a model from an OBJ file
    pub async fn load_obj(
        &self, 
        path: &Path, 
        uniform_buffer: &wgpu::Buffer,
        light_buffer: Option<&wgpu::Buffer>,
    ) -> Result<Model> {
        // Check if file exists first
        if !path.exists() {
            return Err(anyhow::anyhow!("OBJ file not found: {:?}", path));
        }
        
        let obj_dir = path.parent().unwrap_or_else(|| Path::new(""));
        
        // Load the obj file
        let obj_result = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        );

        let (obj_models, obj_materials) = match obj_result {
            Ok((models, materials)) => (models, materials),
            Err(e) => {
                eprintln!("Failed to load OBJ file: {:?}", e);
                return Err(anyhow::anyhow!("Failed to load OBJ file: {:?}", e));
            }
        };

        // Load the MTL file
        let mtl_materials = match obj_materials {
            Ok(materials) => materials,
            Err(e) => {
                eprintln!("No MTL file found or error loading it: {:?}", e);
                vec![]
            }
        };

        let mut materials = Vec::new();
        let mut mesh_data = Vec::new();

        // Process materials from MTL file
        if !mtl_materials.is_empty() {
            for mat in mtl_materials {
                let diffuse_path = if !mat.diffuse_texture.is_empty() {
                    Some(obj_dir.join(&mat.diffuse_texture))
                } else {
                    None
                };
                
                // Try to find PBR textures based on naming conventions
                let normal_texture = if let Some(diff_path) = &diffuse_path {
                    find_pbr_texture(diff_path, "normal", &mat.name)
                } else {
                    None
                };
                
                let metallic_texture = if let Some(diff_path) = &diffuse_path {
                    find_pbr_texture(diff_path, "metallic", &mat.name)
                } else {
                    None
                };
                
                let roughness_texture = if let Some(diff_path) = &diffuse_path {
                    find_pbr_texture(diff_path, "roughness", &mat.name)
                } else {
                    None
                };
                
                // Debug output
                if normal_texture.is_some() || metallic_texture.is_some() || roughness_texture.is_some() {
                    println!("Found PBR textures for {}: Normal: {:?}, Metallic: {:?}, Roughness: {:?}",
                             mat.name, normal_texture, metallic_texture, roughness_texture);
                }
        
                // Guess PBR values from MTL if available
                let metallic = if mat.specular[0] > 0.9 && mat.specular[1] > 0.9 && mat.specular[2] > 0.9 {
                    0.9 // Likely metallic if specular is high and uniform
                } else {
                    0.0 // Non-metallic by default
                };
                
                let roughness = 1.0 - (mat.shininess / 1000.0).min(1.0);
        
                materials.push(Material {
                    name: mat.name,
                    diffuse_texture: diffuse_path,
                    normal_texture,
                    metallic_texture,
                    roughness_texture,
                    ambient: mat.ambient,
                    diffuse: mat.diffuse,
                    specular: mat.specular,
                    metallic,
                    roughness,
                    shininess: mat.shininess,
                });
            }
        }
        
        // Add a default material if none was loaded
        if materials.is_empty() {
            materials.push(Material {
                name: "Default".to_string(),
                diffuse_texture: None,
                normal_texture: None,
                metallic_texture: None,
                roughness_texture: None,
                ambient: [0.1, 0.1, 0.1],
                diffuse: [0.7, 0.7, 0.7],
                specular: [1.0, 1.0, 1.0],
                metallic: 0.0,
                roughness: 0.5,
                shininess: 32.0,
            });
        }

        // Process each mesh in the obj file
        for model in obj_models.iter() {
            let mesh = &model.mesh;

            // Get material id, or use default material
            let material_id = mesh.material_id.unwrap_or(0).min(materials.len() - 1);

            let mut vertices = Vec::new();
            
            // Combine position, normal, and texture coordinates into our custom Vertex type
            for i in 0..mesh.positions.len() / 3 {
                // Ensure we don't exceed array bounds
                if i * 3 + 2 >= mesh.positions.len() {
                    break;
                }
                
                let pos = [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ];
                
                let normal = if !mesh.normals.is_empty() && i * 3 + 2 < mesh.normals.len() {
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ]
                } else {
                    [0.0, 1.0, 0.0]
                };
                
                let tex_coords = if !mesh.texcoords.is_empty() && i * 2 + 1 < mesh.texcoords.len() {
                    [
                        mesh.texcoords[i * 2],
                        1.0 - mesh.texcoords[i * 2 + 1], // Flip Y axis for texture coords
                    ]
                } else {
                    [0.0, 0.0]
                };
                
                vertices.push(Vertex {
                    position: pos,
                    normal,
                    tex_coords,
                    color: [1.0, 1.0, 1.0], // Add default white color
                });
            }
            
            mesh_data.push(MeshData {
                vertices,
                indices: mesh.indices.clone(),
                material_id,
            });
        }

        // Create material instances
        let mut material_instances = Vec::new();
        
        // Create a default sun light buffer outside the loop to extend its lifetime
        let default_sun_light = crate::engine::scene::SunLight::default();
        let default_light_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Default Sun Light Buffer"),
                contents: bytemuck::cast_slice(&[default_sun_light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        for (i, material) in materials.iter().enumerate() {
            // Load diffuse texture
            let texture = if let Some(path) = &material.diffuse_texture {
                match self.texture_loader.load_from_file(path) {
                    Ok(texture) => texture,
                    Err(_) => self.texture_loader.create_colored(material.diffuse, &format!("Material {}", i)),
                }
            } else {
                self.texture_loader.create_colored(material.diffuse, &format!("Material {}", i))
            };
            
            // Create bind group entries
            let mut entries = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ];
            
            // Add light buffer if provided
            if let Some(light_buf) = light_buffer {
                entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_buf.as_entire_binding(),
                });
            } else {
                // Use the default light buffer created outside the loop
                entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: default_light_buffer.as_entire_binding(),
                });
            }
            
            // Create bind group
            let bind_group = self.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: self.bind_group_layout,
                    entries: &entries,
                    label: Some(&format!("material_bind_group_{}", i)),
                }
            );
            
            material_instances.push(MaterialInstance {
                material_id: i,
                diffuse_bind_group: bind_group,
            });
        }

        // Create meshes with vertex and index buffers
        let mut meshes = Vec::new();
        
        for mesh_data in &mesh_data {
            let mesh = Mesh::new(&self.device, mesh_data);
            meshes.push(mesh);
        }
        
        Ok(Model {
            meshes,
            materials: material_instances,
        })
    }
} 