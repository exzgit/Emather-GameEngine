use std::path::Path;
use cgmath::{Matrix4, SquareMatrix, Vector3, Quaternion, Rotation3};
use anyhow::Result;
use crate::engine::scene::Model;
use crate::engine::resources::ModelLoader;
use wgpu;
use nalgebra::Matrix4 as NMatrix4;

/// Component trait for GameObject
#[allow(unused)]
pub trait Component {
    fn update(&mut self, dt: f32);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Transform component for managing position, rotation and scale
#[derive(Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    model_matrix: Matrix4<f32>,
    dirty: bool,
}

impl Transform {
    /// Create a new transform
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_y(cgmath::Rad(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
            model_matrix: Matrix4::identity(),
            dirty: true,
        }
    }

    /// Create a transform with a position
    pub fn with_position(position: Vector3<f32>) -> Self {
        let mut transform = Self::new();
        transform.position = position;
        transform.dirty = true;
        transform
    }
    
    /// Get the model matrix (mutably)
    #[allow(unused)]
    pub fn model_matrix(&mut self) -> Matrix4<f32> {
        if self.dirty {
            self.recalculate_matrix();
        }
        self.model_matrix
    }
    
    /// Get the model matrix (immutably)
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        // Return the cached matrix even if dirty
        // Call model_matrix() first if you need an up-to-date matrix
        self.model_matrix
    }
    
    /// Recalculate the model matrix
    #[allow(unused)]
    fn recalculate_matrix(&mut self) {
        let translation = Matrix4::from_translation(self.position);
        let rotation = Matrix4::from(self.rotation);
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        
        self.model_matrix = translation * rotation * scale;
        self.dirty = false;
    }

    pub fn to_matrix(&self) -> NMatrix4<f32> {
        // Convert cgmath Vector3 to nalgebra Vector3
        let na_position = nalgebra::Vector3::new(self.position.x, self.position.y, self.position.z);
        let translation = NMatrix4::new_translation(&na_position);
        
        // Convert cgmath Quaternion to nalgebra UnitQuaternion
        let na_rotation = nalgebra::UnitQuaternion::new_normalize(
            nalgebra::Quaternion::new(
                self.rotation.s,
                self.rotation.v.x,
                self.rotation.v.y,
                self.rotation.v.z
            )
        );
        let rotation = na_rotation.to_homogeneous();
        
        // Convert cgmath Vector3 to nalgebra Vector3
        let na_scale = nalgebra::Vector3::new(self.scale.x, self.scale.y, self.scale.z);
        let scale = NMatrix4::new_nonuniform_scaling(&na_scale);
        
        translation * rotation * scale
    }
}

impl Component for Transform {
    fn update(&mut self, _dt: f32) {
        // Transformation updates happen when properties change
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Model component for rendering 3D models
pub struct ModelComponent {
    pub model: Option<Model>,
    pub model_path: Option<String>,
    pub uniform_buffer: Option<wgpu::Buffer>,
}

#[allow(unused)]
impl ModelComponent {
    /// Create a new empty model component
    pub fn new() -> Self {
        Self {
            model: None,
            model_path: None,
            uniform_buffer: None,
        }
    }
    
    /// Create a model component with a path to load
    pub fn with_path(path: &str) -> Self {
        Self {
            model: None,
            model_path: Some(path.to_string()),
            uniform_buffer: None,
        }
    }
    
    /// Load the model asynchronously
    pub async fn load_model(
        &mut self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform_buffer: wgpu::Buffer,
        light_buffer: Option<&wgpu::Buffer>,
    ) -> Result<()> {
        if let Some(path) = &self.model_path {
            let model_loader = ModelLoader::new(device, queue, bind_group_layout);
            let model = model_loader.load_obj(
                Path::new(path),
                &uniform_buffer,
                light_buffer,
            ).await?;
            
            self.model = Some(model);
            self.uniform_buffer = Some(uniform_buffer);
        }
        Ok(())
    }
    
    /// Update the transform matrix in the uniform buffer
    pub fn update_transform(&self, queue: &wgpu::Queue, view_proj: Matrix4<f32>, model_matrix: Matrix4<f32>) {
        if let Some(buffer) = &self.uniform_buffer {
            #[repr(C)]
            #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            struct Uniforms {
                view_proj: [[f32; 4]; 4],
                model: [[f32; 4]; 4],
            }
            
            let uniforms = Uniforms {
                view_proj: view_proj.into(),
                model: model_matrix.into(),
            };
            
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }
}

impl Component for ModelComponent {
    fn update(&mut self, _dt: f32) {
        // Model updates are handled elsewhere
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Game object with components
#[allow(unused)]
pub struct GameObject {
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
    pub transform: Transform, 
    pub active: bool,
    pub children: Vec<GameObject>,
}

impl GameObject {
    /// Create a new game object
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            components: Vec::new(),
            transform: Transform::new(),
            active: true,
            children: Vec::new(),
        }
    }
    
    /// Add a component to the game object
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.push(Box::new(component));
    }
    
    /// Get a component by type
    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        for component in &self.components {
            if let Some(c) = component.as_any().downcast_ref::<T>() {
                return Some(c);
            }
        }
        None
    }
    
    /// Get a mutable component by type
    #[allow(unused)]
    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        for component in &mut self.components {
            if let Some(c) = component.as_any_mut().downcast_mut::<T>() {
                return Some(c);
            }
        }
        None
    }
    
    /// Update the game object and its components
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        
        // Update components
        for component in &mut self.components {
            component.update(dt);
        }
        
        // Update children
        for child in &mut self.children {
            child.update(dt);
        }
    }
    
    /// Add a child game object
    #[allow(unused)]
    pub fn add_child(&mut self, child: GameObject) {
        self.children.push(child);
    }
    
    /// Create a game object with a model from a file path
    pub async fn with_model(
        name: &str, 
        model_path: &str,
        position: Vector3<f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        light_buffer: Option<&wgpu::Buffer>,
    ) -> Result<Self> {
        let mut game_object = Self::new(name);
        game_object.transform = Transform::with_position(position);
        
        // Create uniform buffer for the model
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Uniform Buffer for {}", name)),
            size: 2 * 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Add model component
        let mut model_component = ModelComponent::with_path(model_path);
        model_component.load_model(
            device, 
            queue, 
            bind_group_layout,
            uniform_buffer,
            light_buffer,
        ).await?;
        
        game_object.add_component(model_component);
        
        Ok(game_object)
    }
} 