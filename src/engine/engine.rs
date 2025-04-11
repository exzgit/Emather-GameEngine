// #[allow(unused)]
// use winit::{
//     event::{Event, WindowEvent, ElementState, MouseButton, DeviceEvent, KeyboardInput, VirtualKeyCode},
//     event_loop::{ControlFlow, EventLoop},
//     window::{Window, CursorGrabMode},
//     dpi::PhysicalPosition,
// };
// use std::borrow::Cow;
// use wgpu;
// use wgpu::util::DeviceExt;
// use cgmath::{Vector3, Matrix4, Point3, Deg, Rad, InnerSpace};
// use std::path::Path;
// use std::mem::size_of;
// use bytemuck::{Pod, Zeroable, cast_slice};
// use std::f32::consts::PI;
// use std::time::Instant;
// use image::GenericImageView;
// use anyhow::*;
// use std::path::PathBuf;
// #[allow(unused)]
// use std::collections::HashMap;
// use std::result::Result::Ok;

// // Vertex structure for 3D models
// #[repr(C)]
// #[derive(Copy, Clone, Debug, Pod, Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     normal: [f32; 3],
//     tex_coords: [f32; 2],
// }
// #[allow(unused, dead_code)]
// impl Vertex {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: size_of::<[f32; 6]>() as wgpu::BufferAddress,
//                     shader_location: 2,
//                     format: wgpu::VertexFormat::Float32x2,
//                 },
//             ],
//         }
//     }
// }

// // Texture structure to hold texture data
// #[allow(unused, dead_code)]
// struct Texture {
//     texture: wgpu::Texture,
//     view: wgpu::TextureView,
//     sampler: wgpu::Sampler,
// }

// #[allow(unused, dead_code)]
// impl Texture {
//     fn create_depth_texture(
//         device: &wgpu::Device,
//         config: &wgpu::SurfaceConfiguration,
//         label: &str,
//     ) -> Self {
//         let size = wgpu::Extent3d {
//             width: config.width,
//             height: config.height,
//             depth_or_array_layers: 1,
//         };
//         let desc = wgpu::TextureDescriptor {
//             label: Some(label),
//             size,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Depth32Float,
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
//             view_formats: &[],
//         };
//         let texture = device.create_texture(&desc);
//         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::ClampToEdge,
//             address_mode_v: wgpu::AddressMode::ClampToEdge,
//             address_mode_w: wgpu::AddressMode::ClampToEdge,
//             mag_filter: wgpu::FilterMode::Linear,
//             min_filter: wgpu::FilterMode::Linear,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             compare: Some(wgpu::CompareFunction::LessEqual),
//             lod_min_clamp: 0.0,
//             lod_max_clamp: 100.0,
//             ..Default::default()
//         });

//         Self { texture, view, sampler }
//     }

//     fn from_bytes(
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         bytes: &[u8],
//         label: &str,
//     ) -> Result<Self> {
//         let img = image::load_from_memory(bytes)?;
//         Self::from_image(device, queue, &img, Some(label))
//     }

//     fn from_image(
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         img: &image::DynamicImage,
//         label: Option<&str>,
//     ) -> Result<Self> {
//         let rgba = img.to_rgba8();
//         let dimensions = img.dimensions();

//         let size = wgpu::Extent3d {
//             width: dimensions.0,
//             height: dimensions.1,
//             depth_or_array_layers: 1,
//         };
//         let texture = device.create_texture(&wgpu::TextureDescriptor {
//             label,
//             size,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         queue.write_texture(
//             wgpu::ImageCopyTexture {
//                 aspect: wgpu::TextureAspect::All,
//                 texture: &texture,
//                 mip_level: 0,
//                 origin: wgpu::Origin3d::ZERO,
//             },
//             &rgba,
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(4 * dimensions.0),
//                 rows_per_image: Some(dimensions.1),
//             },
//             size,
//         );

//         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::Repeat,
//             address_mode_v: wgpu::AddressMode::Repeat,
//             address_mode_w: wgpu::AddressMode::Repeat,
//             mag_filter: wgpu::FilterMode::Linear,
//             min_filter: wgpu::FilterMode::Linear,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             ..Default::default()
//         });

//         Ok(Self { texture, view, sampler })
//     }

//     fn create_default_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
//         // Create a simple white texture for models with missing textures
//         let data = vec![255u8; 4 * 4 * 4]; // 4x4 white RGBA texture
//         let size = wgpu::Extent3d {
//             width: 4,
//             height: 4,
//             depth_or_array_layers: 1,
//         };
//         let texture = device.create_texture(&wgpu::TextureDescriptor {
//             label: Some("Default Texture"),
//             size,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         queue.write_texture(
//             wgpu::ImageCopyTexture {
//                 aspect: wgpu::TextureAspect::All,
//                 texture: &texture,
//                 mip_level: 0,
//                 origin: wgpu::Origin3d::ZERO,
//             },
//             &data,
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(16), // 4 * 4 bytes
//                 rows_per_image: Some(4),
//             },
//             size,
//         );

//         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::Repeat,
//             address_mode_v: wgpu::AddressMode::Repeat,
//             address_mode_w: wgpu::AddressMode::Repeat,
//             mag_filter: wgpu::FilterMode::Linear,
//             min_filter: wgpu::FilterMode::Linear,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             ..Default::default()
//         });

//         Self { texture, view, sampler }
//     }

//     fn create_colored_texture(
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         color: [f32; 3],
//         label: &str,
//     ) -> Self {
//         // Create a single-color texture based on material color
//         // Convert the floating point color [0.0-1.0] to bytes [0-255]
//         let r = (color[0] * 255.0) as u8;
//         let g = (color[1] * 255.0) as u8;
//         let b = (color[2] * 255.0) as u8;
        
//         // Create a 4x4 texture with the material color
//         let mut data = Vec::with_capacity(4 * 4 * 4);
//         for _ in 0..16 {
//             data.push(r);
//             data.push(g);
//             data.push(b);
//             data.push(255u8);
//         }
        
//         let size = wgpu::Extent3d {
//             width: 4,
//             height: 4,
//             depth_or_array_layers: 1,
//         };
        
//         let texture = device.create_texture(&wgpu::TextureDescriptor {
//             label: Some(label),
//             size,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         queue.write_texture(
//             wgpu::ImageCopyTexture {
//                 aspect: wgpu::TextureAspect::All,
//                 texture: &texture,
//                 mip_level: 0,
//                 origin: wgpu::Origin3d::ZERO,
//             },
//             &data,
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(16), // 4 * 4 bytes per row (RGBA)
//                 rows_per_image: Some(4),
//             },
//             size,
//         );

//         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::Repeat,
//             address_mode_v: wgpu::AddressMode::Repeat,
//             address_mode_w: wgpu::AddressMode::Repeat,
//             mag_filter: wgpu::FilterMode::Linear,
//             min_filter: wgpu::FilterMode::Linear,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             ..Default::default()
//         });

//         Self { texture, view, sampler }
//     }
// }

// // Material data
// #[derive(Debug)]
// #[allow(unused, dead_code)]
// struct Material {
//     name: String,
//     diffuse_texture: Option<PathBuf>,
//     normal_texture: Option<PathBuf>,
//     metallic_texture: Option<PathBuf>,
//     roughness_texture: Option<PathBuf>,
//     ambient: [f32; 3],
//     diffuse: [f32; 3],
//     specular: [f32; 3],
//     metallic: f32,
//     roughness: f32,
//     shininess: f32,
// }

// // Model structure to hold mesh data
// struct Model {
//     meshes: Vec<Mesh>,
//     materials: Vec<MaterialInstance>,
// }

// #[allow(unused, dead_code)]
// struct MaterialInstance {
//     material_id: usize,
//     diffuse_bind_group: wgpu::BindGroup,
// }

// // Mesh structure to hold vertex and index data
// struct Mesh {
//     // name: String,
//     vertex_buffer: wgpu::Buffer,
//     index_buffer: wgpu::Buffer,
//     num_indices: u32,
//     material_id: usize,
// }

// // Uniform data for camera and model transformation
// #[repr(C)]
// #[derive(Debug, Copy, Clone, Pod, Zeroable)]
// struct Uniforms {
//     view_proj: [[f32; 4]; 4],
//     model: [[f32; 4]; 4],
// }

// struct MeshData {
//     vertices: Vec<Vertex>,
//     indices: Vec<u32>,
//     material_id: usize,
// }

// // Camera controller for handling movement and rotation
// struct CameraController {
//     speed: f32,
//     sensitivity: f32,
//     position: Point3<f32>,
//     yaw: Rad<f32>,
//     pitch: Rad<f32>,
//     is_forward_pressed: bool,
//     is_backward_pressed: bool,
//     is_left_pressed: bool,
//     is_right_pressed: bool,
//     is_up_pressed: bool,
//     is_down_pressed: bool,
//     last_mouse_pos: Option<PhysicalPosition<f64>>,
//     initial_mouse_pos: Option<PhysicalPosition<f64>>, // Store initial position when right-click starts
//     last_update: Instant,
//     should_exit_render_loop: bool,
// }

// impl CameraController {
//     fn new(position: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Self {
//         Self {
//             speed: 3.0,
//             sensitivity: 0.004,
//             position,
//             yaw,
//             pitch,
//             is_forward_pressed: false,
//             is_backward_pressed: false,
//             is_left_pressed: false,
//             is_right_pressed: false,
//             is_up_pressed: false,
//             is_down_pressed: false,
//             last_mouse_pos: None,
//             initial_mouse_pos: None,
//             last_update: Instant::now(),
//             should_exit_render_loop: false,
//         }
//     }

//     fn process_keyboard(&mut self, event: &WindowEvent, sun_controller: &mut SunController) -> bool {
//         match event {
//             WindowEvent::KeyboardInput { 
//                 input: KeyboardInput {
//                     virtual_keycode: Some(key_code),
//                     state,
//                     ..
//                 },
//                 ..
//             } => {
//                 let is_pressed = *state == ElementState::Pressed;
//                 match key_code {
//                     VirtualKeyCode::W | VirtualKeyCode::Up => {
//                         self.is_forward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::A | VirtualKeyCode::Left => {
//                         self.is_left_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::S | VirtualKeyCode::Down => {
//                         self.is_backward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::D | VirtualKeyCode::Right => {
//                         self.is_right_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::Space => {
//                         self.is_up_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::LShift => {
//                         self.is_down_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::Escape => {
//                         self.should_exit_render_loop = true;
//                         true
//                     }
//                     // Sun controls
//                     VirtualKeyCode::J => {
//                         if is_pressed {
//                             // Move sun left
//                             sun_controller.horizontal_angle += 0.1;
//                             sun_controller.update_direction();
//                         }
//                         true
//                     },
//                     VirtualKeyCode::L => {
//                         if is_pressed {
//                             // Move sun right
//                             sun_controller.horizontal_angle -= 0.1;
//                             sun_controller.update_direction();
//                         }
//                         true
//                     },
//                     VirtualKeyCode::I => {
//                         if is_pressed {
//                             // Move sun up
//                             sun_controller.vertical_angle = (sun_controller.vertical_angle + 0.1).min(PI * 0.48);
//                             sun_controller.update_direction();
//                         }
//                         true
//                     },
//                     VirtualKeyCode::K => {
//                         if is_pressed {
//                             // Move sun down
//                             sun_controller.vertical_angle = (sun_controller.vertical_angle - 0.1).max(0.05);
//                             sun_controller.update_direction();
//                         }
//                         true
//                     },
//                     _ => false,
//                 }
//             }
//             _ => false,
//         }
//     }

//     fn process_mouse_move(&mut self, position: PhysicalPosition<f64>, mouse_pressed: bool, window: &Window) -> bool {
//         if !mouse_pressed {
//             self.last_mouse_pos = Some(position);
//             return false;
//         }

//         if let Some(last_pos) = self.last_mouse_pos {
//             // Get delta from last position
//             let dx = position.x - last_pos.x;
//             let dy = position.y - last_pos.y;
            
//             // Update camera angles
//             self.yaw += Rad(dx as f32 * self.sensitivity);
//             self.pitch -= Rad(dy as f32 * self.sensitivity);
            
//             // Limit pitch to avoid gimbal lock
//             self.pitch = Rad(self.pitch.0.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01));
            
//             // Reset cursor position to initial position if available
//             if let Some(initial_pos) = self.initial_mouse_pos {
//                 let _ = window.set_cursor_position(initial_pos);
//                 self.last_mouse_pos = Some(initial_pos);
//             } else {
//                 self.last_mouse_pos = Some(position);
//             }
            
//             return true;
//         }
        
//         self.last_mouse_pos = Some(position);
//         false
//     }

//     fn update_camera(&mut self) -> bool {
//         let now = Instant::now();
//         let dt = now.duration_since(self.last_update).as_secs_f32();
//         self.last_update = now;

//         let mut moved = false;

//         // Calculate forward and right vectors
//         let forward = Vector3::new(
//             self.yaw.0.cos() * self.pitch.0.cos(),
//             self.pitch.0.sin(),
//             self.yaw.0.sin() * self.pitch.0.cos(),
//         ).normalize();

//         let right = forward.cross(Vector3::unit_y()).normalize();
//         let up = Vector3::unit_y();

//         // Update position based on input
//         if self.is_forward_pressed {
//             self.position += forward * self.speed * dt;
//             moved = true;
//         }
//         if self.is_backward_pressed {
//             self.position -= forward * self.speed * dt;
//             moved = true;
//         }
//         if self.is_right_pressed {
//             self.position += right * self.speed * dt;
//             moved = true;
//         }
//         if self.is_left_pressed {
//             self.position -= right * self.speed * dt;
//             moved = true;
//         }
//         if self.is_up_pressed {
//             self.position += up * self.speed * dt;
//             moved = true;
//         }
//         if self.is_down_pressed {
//             self.position -= up * self.speed * dt;
//             moved = true;
//         }

//         moved
//     }

//     fn get_view_matrix(&self) -> Matrix4<f32> {
//         // Calculate the look-at direction
//         let forward = Vector3::new(
//             self.yaw.0.cos() * self.pitch.0.cos(),
//             self.pitch.0.sin(),
//             self.yaw.0.sin() * self.pitch.0.cos(),
//         ).normalize();

//         // The target is the position plus the forward vector
//         let target = self.position + forward;

//         // Create the view matrix
//         Matrix4::look_at_rh(
//             self.position,
//             target,
//             Vector3::unit_y(),
//         )
//     }
// }

// // Add a new vertex type for the grid lines
// #[repr(C)]
// #[derive(Copy, Clone, Debug, Pod, Zeroable)]
// struct GridVertex {
//     position: [f32; 3],
//     color: [f32; 3],
// }

// impl GridVertex {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<GridVertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//             ],
//         }
//     }
// }

// // Create a grid structure to hold the vertices and render data
// struct Grid {
//     vertex_buffer: wgpu::Buffer,
//     num_vertices: u32,
// }

// impl Grid {
//     fn new(device: &wgpu::Device, size: f32, divisions: u32) -> Self {
//         let mut vertices = Vec::new();
        
//         // Generate grid lines along X axis
//         for i in 0..=divisions {
//             let position_x = (i as f32 / divisions as f32) * size - (size / 2.0);
            
//             // Main X axis (red)
//             let color_x = if i == divisions / 2 {
//                 [1.0, 0.0, 0.0] // Red for center line
//             } else {
//                 [0.5, 0.5, 0.5] // Grey for other lines
//             };
            
//             // Line along X axis
//             vertices.push(GridVertex {
//                 position: [position_x, 0.0, -size / 2.0],
//                 color: color_x,
//             });
//             vertices.push(GridVertex {
//                 position: [position_x, 0.0, size / 2.0],
//                 color: color_x,
//             });
//         }
        
//         // Generate grid lines along Z axis
//         for i in 0..=divisions {
//             let position_z = (i as f32 / divisions as f32) * size - (size / 2.0);
            
//             // Main Z axis (blue)
//             let color_z = if i == divisions / 2 {
//                 [0.0, 0.0, 1.0] // Blue for center line
//             } else {
//                 [0.5, 0.5, 0.5] // Grey for other lines
//             };
            
//             // Line along Z axis
//             vertices.push(GridVertex {
//                 position: [-size / 2.0, 0.0, position_z],
//                 color: color_z,
//             });
//             vertices.push(GridVertex {
//                 position: [size / 2.0, 0.0, position_z],
//                 color: color_z,
//             });
//         }
        
//         // Draw Y axis as a vertical line (green)
//         vertices.push(GridVertex {
//             position: [0.0, -size / 2.0, 0.0],
//             color: [0.0, 1.0, 0.0], // Green
//         });
//         vertices.push(GridVertex {
//             position: [0.0, size / 2.0, 0.0],
//             color: [0.0, 1.0, 0.0], // Green
//         });
        
//         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Grid Vertex Buffer"),
//             contents: bytemuck::cast_slice(&vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         });
        
//         Self {
//             vertex_buffer,
//             num_vertices: vertices.len() as u32,
//         }
//     }
// }

// // Add these at the module level (top of file with other structs)
// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct SunLight {
//     direction: [f32; 3],
//     _padding1: u32,
//     color: [f32; 3],
//     intensity: f32,
//     // PBR settings
//     use_pbr: u32,      // Flag to enable/disable PBR rendering
//     metallic_factor: f32,  // Default metallic factor when no texture
//     roughness_factor: f32, // Default roughness factor when no texture
//     _padding2: f32,    // Keep 16-byte alignment
// }

// struct SunController {
//     horizontal_angle: f32,
//     vertical_angle: f32,
//     use_pbr: bool,
//     metallic: f32,
//     roughness: f32,
// }

// impl SunController {
//     fn new() -> Self {
//         let mut controller = Self {
//             horizontal_angle: std::f32::consts::PI * 0.25,
//             vertical_angle: std::f32::consts::PI * 0.3,
//             use_pbr: true,
//             metallic: 0.8,
//             roughness: 0.2,
//         };
//         // Initialize direction when created
//         controller.update_direction();
//         controller
//     }
    
//     fn update_direction(&mut self) {
//         // This method exists just to maintain compatibility with the code that calls it
//         // The actual direction calculation happens in get_light()
//     }
    
//     fn get_light(&self) -> SunLight {
//         let x = self.horizontal_angle.cos() * self.vertical_angle.cos();
//         let y = self.vertical_angle.sin();
//         let z = self.horizontal_angle.sin() * self.vertical_angle.cos();
        
//         SunLight {
//             direction: [x, y, z],
//             _padding1: 0,
//             color: [1.0, 0.98, 0.9], // More white-yellow for daylight
//             intensity: 2.0, // Increased for brighter lighting
//             use_pbr: if self.use_pbr { 1 } else { 0 },
//             metallic_factor: self.metallic,
//             roughness_factor: self.roughness,
//             _padding2: 0.0,
//         }
//     }
    
//     fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
//         match event {
//             WindowEvent::KeyboardInput { 
//                 input: KeyboardInput {
//                     virtual_keycode: Some(key_code),
//                     state,
//                     ..
//                 },
//                 ..
//             } => {
//                 let is_pressed = *state == ElementState::Pressed;
//                 match key_code {
//                     // Sun direction controls
//                     VirtualKeyCode::J => {
//                         if is_pressed {
//                             self.horizontal_angle += 0.1;
//                             self.update_direction();
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::L => {
//                         if is_pressed {
//                             self.horizontal_angle -= 0.1;
//                             self.update_direction();
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::I => {
//                         if is_pressed {
//                             self.vertical_angle = (self.vertical_angle + 0.1).min(std::f32::consts::PI * 0.48);
//                             self.update_direction();
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::K => {
//                         if is_pressed {
//                             self.vertical_angle = (self.vertical_angle - 0.1).max(0.05);
//                             self.update_direction();
//                             true
//                         } else { false }
//                     },
//                     // PBR controls
//                     VirtualKeyCode::P => {
//                         if is_pressed {
//                             self.use_pbr = !self.use_pbr;
//                             println!("PBR rendering: {}", if self.use_pbr { "ON" } else { "OFF" });
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::M => {
//                         if is_pressed {
//                             self.metallic = (self.metallic + 0.1).min(1.0);
//                             println!("Metallic: {:.1}", self.metallic);
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::N => {
//                         if is_pressed {
//                             self.metallic = (self.metallic - 0.1).max(0.0);
//                             println!("Metallic: {:.1}", self.metallic);
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::R => {
//                         if is_pressed {
//                             self.roughness = (self.roughness + 0.1).min(1.0);
//                             println!("Roughness: {:.1}", self.roughness);
//                             true
//                         } else { false }
//                     },
//                     VirtualKeyCode::F => {
//                         if is_pressed {
//                             self.roughness = (self.roughness - 0.1).max(0.0);
//                             println!("Roughness: {:.1}", self.roughness);
//                             true
//                         } else { false }
//                     },
//                     _ => false,
//                 }
//             },
//             _ => false,
//         }
//     }
// }

// // Add this function to create a sun sphere
// fn create_sun_sphere(_device: &wgpu::Device, radius: f32) -> MeshData {
//     // Generate a simple UV sphere to represent the sun
//     let mut vertices: Vec<Vertex> = Vec::new();
//     let mut indices: Vec<u32> = Vec::new();
    
//     let segments = 32;
//     let rings = 32;
    
//     // Generate vertices
//     for ring in 0..=rings {
//         let phi = std::f32::consts::PI * ring as f32 / rings as f32;
//         for segment in 0..=segments {
//             let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
            
//             let x = radius * phi.sin() * theta.cos();
//             let y = radius * phi.cos();
//             let z = radius * phi.sin() * theta.sin();
            
//             let normal = [x/radius, y/radius, z/radius];
//             let tex_coord = [
//                 segment as f32 / segments as f32,
//                 ring as f32 / rings as f32,
//             ];
            
//             vertices.push(Vertex {
//                 position: [x, y, z],
//                 normal,
//                 tex_coords: tex_coord,
//             });
//         }
//     }
    
//     // Generate indices
//     for ring in 0..rings {
//         for segment in 0..segments {
//             let first = ring * (segments + 1) + segment;
//             let second = first + segments + 1;
            
//             indices.push(first as u32);
//             indices.push(second as u32);
//             indices.push((first + 1) as u32);
            
//             indices.push(second as u32);
//             indices.push((second + 1) as u32);
//             indices.push((first + 1) as u32);
//         }
//     }
    
//     // Create the MeshData struct using the existing fields
//     MeshData {
//         vertices,
//         indices,
//         material_id: 0,
//     }
// }

// // Helper function to look for PBR textures based on common naming conventions
// fn find_pbr_texture(base_path: &Path, texture_type: &str, material_name: &str) -> Option<PathBuf> {
//     // Common suffixes for PBR textures
//     let suffixes = match texture_type {
//         "normal" => vec!["_normal", "_nrm", "_n", "-normal", "-nrm", "-n", "_Normal"],
//         "metallic" => vec!["_metallic", "_metalness", "_metal", "_m", "-metallic", "-m", "_Metallic"],
//         "roughness" => vec!["_roughness", "_rough", "_r", "-roughness", "-r", "_Roughness"],
//         _ => vec![],
//     };
    
//     // Check if base directory exists
//     let base_dir = base_path.parent().unwrap_or(Path::new(""));
//     if !base_dir.exists() {
//         return None;
//     }
    
//     // Try each suffix
//     for suffix in suffixes {
//         // Try with material name
//         let material_base = material_name.split_whitespace().next().unwrap_or(material_name);
//         let test_path = base_dir.join(format!("{}{}.png", material_base, suffix));
//         if test_path.exists() {
//             return Some(test_path);
//         }
        
//         // Try with base file name
//         if let Some(base_name) = base_path.file_stem() {
//             if let Some(base_str) = base_name.to_str() {
//                 let test_path = base_dir.join(format!("{}{}.png", base_str, suffix));
//                 if test_path.exists() {
//                     return Some(test_path);
//                 }
                
//                 // Try with jpg extension
//                 let test_path = base_dir.join(format!("{}{}.jpg", base_str, suffix));
//                 if test_path.exists() {
//                     return Some(test_path);
//                 }
//             }
//         }
//     }
    
//     None
// }

// pub async fn run(event_loop: EventLoop<()>, window: Window) {
//     window.set_maximized(true);
//     let size = window.inner_size();
//     let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//         backends: wgpu::Backends::all(),
//         dx12_shader_compiler: Default::default(),
//     });
    
//     let surface = unsafe { instance.create_surface(&window) }.unwrap();
    
//     let adapter = instance.request_adapter(
//         &wgpu::RequestAdapterOptions { 
//             power_preference: wgpu::PowerPreference::default(),
//             force_fallback_adapter: false,
//             compatible_surface: Some(&surface),
//         }
//     ).await.expect("Failed to request adapter");

//     let (device, queue) = adapter.request_device(
//         &wgpu::DeviceDescriptor {
//             label: None,
//             features: wgpu::Features::empty(),
//             limits: wgpu::Limits::default(),
//         },
//         None,
//     ).await.expect("Failed to create device");

//     let surface_caps = surface.get_capabilities(&adapter);
//     let format = surface_caps.formats.iter()
//         .copied()
//         .find(|f| f.is_srgb())
//         .unwrap_or(surface_caps.formats[0]);
        
//     let mut config = wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format,
//         width: size.width.max(1),
//         height: size.height.max(1),
//         present_mode: wgpu::PresentMode::Fifo,
//         alpha_mode: surface_caps.alpha_modes[0],
//         view_formats: vec![],
//     };

//     surface.configure(&device, &config);
    
//     // Initialize camera controller
//     let mut camera_controller = CameraController::new(
//         Point3::new(0.0, 3.0, 10.0),
//         Rad(3.0 * PI / 2.0),
//         Rad(-PI / 8.0),
//     );
    
//     // Create the sun controller
//     let mut sun_controller = SunController::new();

//     // Create the sun light buffer
//     let sun_light = sun_controller.get_light();
//     let sun_light_buffer = device.create_buffer_init(
//         &wgpu::util::BufferInitDescriptor {
//             label: Some("Sun Light Buffer"),
//             contents: bytemuck::cast_slice(&[sun_light]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         }
//     );

//     // Create shader and pipeline here, inside the run function
//     let sun_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: Some("Sun Shader"),
//         source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
//             struct Uniforms {
//                 view_proj: mat4x4<f32>,
//                 model: mat4x4<f32>,
//             };

//             @group(0) @binding(0)
//             var<uniform> uniforms: Uniforms;

//             @group(0) @binding(1)
//             var t_diffuse: texture_2d<f32>;
//             @group(0) @binding(2)
//             var s_diffuse: sampler;

//             struct SunLight {
//                 direction: vec3<f32>,
//                 _padding1: u32,
//                 color: vec3<f32>,
//                 intensity: f32,
//                 use_pbr: u32,
//                 metallic_factor: f32,
//                 roughness_factor: f32,
//                 _padding2: f32,
//             };

//             @group(0) @binding(3)
//             var<uniform> sun_light: SunLight;

//             struct VertexInput {
//                 @location(0) position: vec3<f32>,
//                 @location(1) normal: vec3<f32>,
//                 @location(2) tex_coords: vec2<f32>,
//             };

//             struct VertexOutput {
//                 @builtin(position) clip_position: vec4<f32>,
//                 @location(0) position: vec3<f32>,
//                 @location(1) normal: vec3<f32>,
//                 @location(2) tex_coords: vec2<f32>,
//                 @location(3) view_dir: vec3<f32>,
//             };

//             @vertex
//             fn vs_main(
//                 in: VertexInput,
//             ) -> VertexOutput {
//                 var out: VertexOutput;
//                 out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(in.position, 1.0);
                
//                 // Transform position and normal to world space
//                 let model_matrix = uniforms.model;
//                 out.position = (model_matrix * vec4<f32>(in.position, 1.0)).xyz;
//                 out.normal = normalize((model_matrix * vec4<f32>(in.normal, 0.0)).xyz);
                
//                 // Calculate view direction (from position to camera)
//                 // This assumes camera at (0,0,0) in view space, which is a reasonable approximation
//                 out.view_dir = normalize(-out.position);
                
//                 out.tex_coords = in.tex_coords;
//                 return out;
//             }

//             // PBR helper functions
//             fn distributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
//                 let a = roughness * roughness;
//                 let a2 = a * a;
//                 let NdotH = max(dot(N, H), 0.0);
//                 let NdotH2 = NdotH * NdotH;
                
//                 let num = a2;
//                 let denom = (NdotH2 * (a2 - 1.0) + 1.0);
//                 return num / (3.14159265359 * denom * denom);
//             }
            
//             fn geometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
//                 let r = (roughness + 1.0);
//                 let k = (r * r) / 8.0;
                
//                 return NdotV / (NdotV * (1.0 - k) + k);
//             }
            
//             fn geometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
//                 let NdotV = max(dot(N, V), 0.0);
//                 let NdotL = max(dot(N, L), 0.0);
//                 let ggx2 = geometrySchlickGGX(NdotV, roughness);
//                 let ggx1 = geometrySchlickGGX(NdotL, roughness);
                
//                 return ggx1 * ggx2;
//             }
            
//             fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
//                 return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
//             }

//             @fragment
//             fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//                 // Sample the texture
//                 let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb;
                
//                 // Normal and view vectors
//                 let N = normalize(in.normal);
//                 let V = normalize(in.view_dir);
                
//                 // Material properties
//                 let metallic = sun_light.metallic_factor;
//                 let roughness = sun_light.roughness_factor;
//                 let ao = 1.0; // Ambient occlusion default
                
//                 // Reflectance at normal incidence (Fresnel F0)
//                 // For dielectric (non-metal) materials, use 0.04 (4% reflectivity)
//                 // For metals, use their albedo (color) 
//                 let F0 = mix(vec3<f32>(0.04), albedo, metallic);
                
//                 // Direct lighting calculation
//                 let light_dir = normalize(-sun_light.direction);
//                 let light_color = sun_light.color * sun_light.intensity;
                
//                 // Calculate the light parameters for PBR
//                 let H = normalize(V + light_dir);
//                 let NdotL = max(dot(N, light_dir), 0.0);
                
//                 // Cook-Torrance BRDF
//                 let NDF = distributionGGX(N, H, roughness);
//                 let G = geometrySmith(N, V, light_dir, roughness);
//                 let F = fresnelSchlick(max(dot(H, V), 0.0), F0);
                
//                 let kS = F; // Specular contribution
//                 let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic); // Diffuse contribution
                
//                 // Specular component
//                 let numerator = NDF * G * F;
//                 let denominator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
//                 let specular = numerator / denominator;
                
//                 // Combine diffuse and specular
//                 var Lo = vec3<f32>(0.0);
//                 if (NdotL > 0.0) {
//                     Lo += (kD * albedo / 3.14159265359 + specular) * light_color * NdotL;
//                 }
                
//                 // Ambient lighting
//                 let ambient = vec3<f32>(0.2) * albedo * ao;
                
//                 // Final color
//                 var final_color = vec3<f32>(0.0);
                
//                 // Debug visualization modes (based on material properties)
//                 // Debug mode: 0 = normal rendering, 1 = metallic visualization, 2 = roughness visualization
//                 let debug_mode = 0;  // Change this to 1 or 2 to see metallic/roughness
                
//                 if (debug_mode == 1) {
//                     // Visualize metallic
//                     final_color = vec3<f32>(metallic, metallic, metallic);
//                 } else if (debug_mode == 2) {
//                     // Visualize roughness
//                     final_color = vec3<f32>(roughness, roughness, roughness);
//                 } else if (sun_light.use_pbr != 0u) {  // Use != 0u instead of > 0
//                     // Use full PBR
//                     final_color = ambient + Lo;
//                 } else {
//                     // Use simple diffuse/ambient lighting
//                     let diffuse = max(dot(N, light_dir), 0.0);
//                     let ambient = 0.4;
//                     final_color = albedo * (ambient + diffuse * sun_light.intensity * sun_light.color);
//                 }
                
//                 // Apply tone mapping (HDR -> LDR)
//                 final_color = final_color / (final_color + vec3<f32>(1.0));
                
//                 // Apply gamma correction
//                 final_color = pow(final_color, vec3<f32>(1.0/2.2));
                
//                 return vec4<f32>(final_color, 1.0);
//             }
//         "#)),
//     });

//     // Create a bind group layout that includes the light
//     let sun_bind_group_layout = device.create_bind_group_layout(
//         &wgpu::BindGroupLayoutDescriptor {
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                         multisampled: false,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 3,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//             ],
//             label: Some("sun_bind_group_layout"),
//         }
//     );

//     // Create camera and transformation matrices
//     let aspect = config.width as f32 / config.height as f32;
//     let projection = cgmath::perspective(Deg(60.0), aspect, 0.1, 100.0);
//     let model_transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0))
//         * Matrix4::from_scale(0.5);
    
//     // Create uniform buffer
//     let uniforms = Uniforms {
//         view_proj: (projection * camera_controller.get_view_matrix()).into(),
//         model: model_transform.into(),
//     };
    
//     let uniform_buffer = device.create_buffer_init(
//         &wgpu::util::BufferInitDescriptor {
//             label: Some("Uniform Buffer"),
//             contents: bytemuck::cast_slice(&[uniforms]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         }
//     );

//     // Create depth texture
//     let mut depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

//     // Create bind group layouts
//     let _texture_bind_group_layout = device.create_bind_group_layout(
//         &wgpu::BindGroupLayoutDescriptor {
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                         multisampled: false,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                     count: None,
//                 },
//             ],
//             label: Some("texture_bind_group_layout"),
//         }
//     );
    
//     // Load the OBJ model with materials
//     let obj_path = Path::new("models/bugatti/bugatti.obj");
//     let model = load_model(
//         obj_path, 
//         &device, 
//         &queue, 
//         &_texture_bind_group_layout, 
//         &uniform_buffer,
//         &sun_bind_group_layout,
//         &sun_light_buffer
//     ).await.expect("Failed to load model");

//     // Create shader for 3D rendering
//     let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: None,
//         source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders/shader3d.wgsl"))),
//     });

//     // Create a shader for the grid
//     let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: Some("Grid Shader"),
//         source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(r#"
//             struct Uniforms {
//                 view_proj: mat4x4<f32>,
//                 model: mat4x4<f32>,
//             };

//             @group(0) @binding(0)
//             var<uniform> uniforms: Uniforms;

//             struct VertexInput {
//                 @location(0) position: vec3<f32>,
//                 @location(1) color: vec3<f32>,
//             };

//             struct VertexOutput {
//                 @builtin(position) clip_position: vec4<f32>,
//                 @location(0) color: vec3<f32>,
//             };

//             @vertex
//             fn vs_main(
//                 in: VertexInput,
//             ) -> VertexOutput {
//                 var out: VertexOutput;
//                 out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(in.position, 1.0);
//                 out.color = in.color;
//                 return out;
//             }

//             @fragment
//             fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//                 return vec4<f32>(in.color, 1.0);
//             }
//         "#)),
//     });

//     // Create the pipeline layout for the model
//     let sun_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: Some("Sun Pipeline Layout"),
//         bind_group_layouts: &[&sun_bind_group_layout],
//         push_constant_ranges: &[],
//     });

//     // Create the pipeline layout for the grid
//     let grid_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: Some("Grid Pipeline Layout"),
//         bind_group_layouts: &[&_texture_bind_group_layout], // We can reuse the same layout since we only need uniforms
//         push_constant_ranges: &[],
//     });

//     // Create the render pipeline for the model
//     let sun_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some("Sun Pipeline"),
//         layout: Some(&sun_pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: &sun_shader,
//             entry_point: "vs_main",
//             buffers: &[Vertex::desc()],
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &sun_shader,
//             entry_point: "fs_main",
//             targets: &[Some(wgpu::ColorTargetState {
//                 format: config.format,
//                 blend: Some(wgpu::BlendState::REPLACE),
//                 write_mask: wgpu::ColorWrites::ALL,
//             })],
//         }),
//         primitive: wgpu::PrimitiveState {
//             topology: wgpu::PrimitiveTopology::TriangleList,
//             strip_index_format: None,
//             front_face: wgpu::FrontFace::Ccw,
//             cull_mode: Some(wgpu::Face::Back),
//             polygon_mode: wgpu::PolygonMode::Fill,
//             unclipped_depth: false,
//             conservative: false,
//         },
//         depth_stencil: Some(wgpu::DepthStencilState {
//             format: wgpu::TextureFormat::Depth32Float,
//             depth_write_enabled: true,
//             depth_compare: wgpu::CompareFunction::Less,
//             stencil: wgpu::StencilState::default(),
//             bias: wgpu::DepthBiasState::default(),
//         }),
//         multisample: wgpu::MultisampleState {
//             count: 1,
//             mask: !0,
//             alpha_to_coverage_enabled: false,
//         },
//         multiview: None,
//     });

//     // Create the render pipeline for the grid
//     let grid_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some("Grid Render Pipeline"),
//         layout: Some(&grid_pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: &grid_shader,
//             entry_point: "vs_main",
//             buffers: &[GridVertex::desc()],
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &grid_shader,
//             entry_point: "fs_main",
//             targets: &[Some(wgpu::ColorTargetState {
//                 format: config.format,
//                 blend: Some(wgpu::BlendState {
//                     color: wgpu::BlendComponent {
//                         src_factor: wgpu::BlendFactor::SrcAlpha,
//                         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
//                         operation: wgpu::BlendOperation::Add,
//                     },
//                     alpha: wgpu::BlendComponent {
//                         src_factor: wgpu::BlendFactor::One,
//                         dst_factor: wgpu::BlendFactor::One,
//                         operation: wgpu::BlendOperation::Add,
//                     },
//                 }),
//                 write_mask: wgpu::ColorWrites::ALL,
//             })],
//         }),
//         primitive: wgpu::PrimitiveState {
//             topology: wgpu::PrimitiveTopology::LineList,
//             strip_index_format: None,
//             front_face: wgpu::FrontFace::Ccw,
//             cull_mode: None,
//             polygon_mode: wgpu::PolygonMode::Fill,
//             unclipped_depth: false,
//             conservative: false,
//         },
//         depth_stencil: Some(wgpu::DepthStencilState {
//             format: wgpu::TextureFormat::Depth32Float,
//             depth_write_enabled: true,
//             depth_compare: wgpu::CompareFunction::Less,
//             stencil: wgpu::StencilState::default(),
//             bias: wgpu::DepthBiasState::default(),
//         }),
//         multisample: wgpu::MultisampleState {
//             count: 1,
//             mask: !0,
//             alpha_to_coverage_enabled: false,
//         },
//         multiview: None,
//     });

//     // Create the grid
//     let grid = Grid::new(&device, 4000.0, 4000);

//     // Create a bind group for the grid
//     let grid_default_texture = Texture::create_default_texture(&device, &queue);
//     let grid_bind_group = device.create_bind_group(
//         &wgpu::BindGroupDescriptor {
//             layout: &_texture_bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: uniform_buffer.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::TextureView(&grid_default_texture.view),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource: wgpu::BindingResource::Sampler(&grid_default_texture.sampler),
//                 },
//             ],
//             label: Some("grid_bind_group"),
//         }
//     );

//     // Camera state
//     let mut mouse_right_pressed = false;

//     // Create Sun visualization (a simple sphere)
//     let sun_sphere_data = create_sun_sphere(&device, 5.0);
//     let _sun_sphere = Mesh {
//         vertex_buffer: device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some("Sun Vertex Buffer"),
//                 contents: bytemuck::cast_slice(&sun_sphere_data.vertices),
//                 usage: wgpu::BufferUsages::VERTEX,
//             }
//         ),
//         index_buffer: device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some("Sun Index Buffer"),
//                 contents: bytemuck::cast_slice(&sun_sphere_data.indices),
//                 usage: wgpu::BufferUsages::INDEX,
//             }
//         ),
//         num_indices: sun_sphere_data.indices.len() as u32,
//         material_id: sun_sphere_data.material_id,
//     };

//     event_loop.run(move |event, _, control_flow| {
//         *control_flow = ControlFlow::Poll;

//         match event {
//             // Handle window events
//             Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
//                 match event {
//                     WindowEvent::KeyboardInput { .. } => {
//                         let mut redraw_needed = false;
                        
//                         if camera_controller.process_keyboard(event, &mut sun_controller) {
//                             redraw_needed = true;
//                         }
                        
//                         if sun_controller.process_keyboard(event) {
//                             // Update the sun light when direction changes
//                             let new_light = sun_controller.get_light();
//                             queue.write_buffer(&sun_light_buffer, 0, bytemuck::cast_slice(&[new_light]));
//                             redraw_needed = true;
//                         }
                        
//                         if redraw_needed {
//                             window.request_redraw();
//                         }
//                     },
//                     WindowEvent::MouseInput { 
//                         button: MouseButton::Right,
//                         state,
//                         ..
//                     } => {
//                         // Toggle cursor capture when right mouse button is pressed/released
//                         match state {
//                             ElementState::Pressed => {
//                                 mouse_right_pressed = true;
//                                 // Store the initial mouse position when capturing
//                                 camera_controller.initial_mouse_pos = camera_controller.last_mouse_pos;
//                                 // Try to grab the cursor - ignore errors if it fails
//                                 let _ = window.set_cursor_grab(CursorGrabMode::Confined);
//                                 window.set_cursor_visible(false);
//                             },
//                             ElementState::Released => {
//                                 mouse_right_pressed = false;
//                                 camera_controller.initial_mouse_pos = None;
//                                 let _ = window.set_cursor_grab(CursorGrabMode::None);
//                                 window.set_cursor_visible(true);
//                             },
//                         }
//                         window.request_redraw();
//                     },
//                     WindowEvent::CursorMoved { position, .. } => {
//                         if mouse_right_pressed {
//                             if camera_controller.process_mouse_move(*position, mouse_right_pressed, &window) {
//                                 // Update view matrix if camera rotated
//                                 let aspect = config.width as f32 / config.height as f32;
//                                 let projection = cgmath::perspective(Deg(60.0), aspect, 0.1, 100.0);
                                
//                                 let uniforms = Uniforms {
//                                     view_proj: (projection * camera_controller.get_view_matrix()).into(),
//                                     model: model_transform.into(),
//                                 };
                                
//                                 queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
//                                 window.request_redraw();
//                             }
//                         } else {
//                             // Just update the last position when not pressed
//                             camera_controller.last_mouse_pos = Some(*position);
//                         }
//                     },
//                     WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                     WindowEvent::Resized(physical_size) => {
//                         if physical_size.width > 0 && physical_size.height > 0 {
//                             config.width = physical_size.width;
//                             config.height = physical_size.height;
//                             surface.configure(&device, &config);
                            
//                             // Recreate depth texture
//                             depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");
                            
//                             let aspect = config.width as f32 / config.height as f32;
//                             let projection = cgmath::perspective(Deg(60.0), aspect, 0.1, 100.0);
                            
//                             let uniforms = Uniforms {
//                                 view_proj: (projection * camera_controller.get_view_matrix()).into(),
//                                 model: model_transform.into(),
//                             };
                            
//                             queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
//                         }
//                         window.request_redraw();
//                     },
//                     WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                         // Handle high DPI changes
//                         if new_inner_size.width > 0 && new_inner_size.height > 0 {
//                             config.width = new_inner_size.width;
//                             config.height = new_inner_size.height;
//                             surface.configure(&device, &config);
                            
//                             // Recreate depth texture
//                             depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");
                            
//                             let aspect = config.width as f32 / config.height as f32;
//                             let projection = cgmath::perspective(Deg(60.0), aspect, 0.1, 100.0);
                            
//                             let uniforms = Uniforms {
//                                 view_proj: (projection * camera_controller.get_view_matrix()).into(),
//                                 model: model_transform.into(),
//                             };
                            
//                             queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
//                         }
//                         window.request_redraw();
//                     },
//                     _ => {}
//                 }
//             },
//             Event::MainEventsCleared => {
//                 // Update camera position based on input
//                 if camera_controller.update_camera() {
//                     // Only if the camera moved, update the uniform buffer
//                     let aspect = config.width as f32 / config.height as f32;
//                     let projection = cgmath::perspective(Deg(60.0), aspect, 0.1, 100.0);
                    
//                     let uniforms = Uniforms {
//                         view_proj: (projection * camera_controller.get_view_matrix()).into(),
//                         model: model_transform.into(),
//                     };
                    
//                     queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
//                 }
                
//                 // Request a redraw continuously for smoother controls
//                 window.request_redraw();
//             },
//             Event::RedrawRequested(_) => {
//                 // Ensure current surface dimensions match the config before getting a texture
//                 let current_size = window.inner_size();
//                 if current_size.width != config.width || current_size.height != config.height {
//                     config.width = current_size.width.max(1);
//                     config.height = current_size.height.max(1);
//                     surface.configure(&device, &config);
//                     depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");
//                 }
                
//                 // Render the scene
//                 let frame = match surface.get_current_texture() {
//                     Ok(frame) => frame,
//                     Err(_) => {
//                         // If we get an error getting the texture, reconfigure the surface and try again
//                         surface.configure(&device, &config);
//                         match surface.get_current_texture() {
//                             Ok(frame) => frame,
//                             Err(e) => {
//                                 println!("Failed to acquire texture: {:?}", e);
//                                 return;
//                             }
//                         }
//                     }
//                 };
                
//                 let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
//                 let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//                 {
//                     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                         label: None,
//                         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                             view: &view,
//                             resolve_target: None,
//                             ops: wgpu::Operations {
//                                 load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
//                                 store: true,
//                             },
//                         })],
//                         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
//                             view: &depth_texture.view,
//                             depth_ops: Some(wgpu::Operations {
//                                 load: wgpu::LoadOp::Clear(1.0),
//                                 store: true,
//                             }),
//                             stencil_ops: None,
//                         }),
//                     });
                    
//                     // Draw the model
//                     render_pass.set_pipeline(&sun_pipeline);
                    
//                     // Draw each mesh with its material
//                     for mesh in &model.meshes {
//                         let material = &model.materials[mesh.material_id];
                        
//                         render_pass.set_bind_group(0, &material.diffuse_bind_group, &[]);
//                         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//                         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//                         render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
//                     }

//                     // Draw the grid
//                     render_pass.set_pipeline(&grid_render_pipeline);
//                     render_pass.set_bind_group(0, &grid_bind_group, &[]);
//                     render_pass.set_vertex_buffer(0, grid.vertex_buffer.slice(..));
//                     render_pass.draw(0..grid.num_vertices, 0..1);
//                 }
                
//                 queue.submit(std::iter::once(encoder.finish()));
//                 frame.present();
//             },
//             _ => {}
//         }

//         if camera_controller.should_exit_render_loop {
//             *control_flow = ControlFlow::Exit;
//         }
//     })
// }

// // Load a model from an OBJ file with materials
// async fn load_model(
//     obj_path: &Path, 
//     device: &wgpu::Device, 
//     queue: &wgpu::Queue,
//     _texture_bind_group_layout: &wgpu::BindGroupLayout,
//     uniform_buffer: &wgpu::Buffer,
//     sun_bind_group_layout: &wgpu::BindGroupLayout,
//     sun_light_buffer: &wgpu::Buffer
// ) -> Result<Model> {
//     let obj_dir = obj_path.parent().unwrap_or_else(|| Path::new(""));
    
//     // Load the obj file
//     let obj_result = tobj::load_obj(
//         obj_path,
//         &tobj::LoadOptions {
//             triangulate: true,
//             single_index: true,
//             ..Default::default()
//         },
//     );

//     let (obj_models, obj_materials) = match obj_result {
//         Ok((models, materials)) => (models, materials),
//         Err(e) => {
//             println!("Failed to load OBJ file: {:?}", e);
//             return Err(anyhow!("Failed to load OBJ file"));
//         }
//     };

//     // Load the MTL file
//     let mtl_materials = match obj_materials {
//         Ok(materials) => materials,
//         Err(e) => {
//             println!("No MTL file found or error loading it: {:?}", e);
//             vec![]
//         }
//     };

//     let mut materials = Vec::new();
//     let mut mesh_data = Vec::new();

//     // Process materials from MTL file
//     if !mtl_materials.is_empty() {
//         for mat in mtl_materials {
//             let diffuse_path = if !mat.diffuse_texture.is_empty() {
//                 Some(obj_dir.join(&mat.diffuse_texture))
//             } else {
//                 None
//             };
            
//             // Try to find PBR textures based on naming conventions
//             let normal_texture = if let Some(diff_path) = &diffuse_path {
//                 find_pbr_texture(diff_path, "normal", &mat.name)
//             } else {
//                 None
//             };
            
//             let metallic_texture = if let Some(diff_path) = &diffuse_path {
//                 find_pbr_texture(diff_path, "metallic", &mat.name)
//             } else {
//                 None
//             };
            
//             let roughness_texture = if let Some(diff_path) = &diffuse_path {
//                 find_pbr_texture(diff_path, "roughness", &mat.name)
//             } else {
//                 None
//             };
            
//             // Debug output
//             if normal_texture.is_some() || metallic_texture.is_some() || roughness_texture.is_some() {
//                 println!("Found PBR textures for {}: Normal: {:?}, Metallic: {:?}, Roughness: {:?}",
//                          mat.name, normal_texture, metallic_texture, roughness_texture);
//             }
    
//             // Guess PBR values from MTL if available
//             let metallic = if mat.specular[0] > 0.9 && mat.specular[1] > 0.9 && mat.specular[2] > 0.9 {
//                 0.9 // Likely metallic if specular is high and uniform
//             } else {
//                 0.0 // Non-metallic by default
//             };
            
//             let roughness = 1.0 - (mat.shininess / 1000.0).min(1.0);
    
//             materials.push(Material {
//                 name: mat.name,
//                 diffuse_texture: diffuse_path,
//                 normal_texture,
//                 metallic_texture,
//                 roughness_texture,
//                 ambient: mat.ambient,
//                 diffuse: mat.diffuse,
//                 specular: mat.specular,
//                 metallic,
//                 roughness,
//                 shininess: mat.shininess,
//             });
//         }
//     }
    
//     // Add a default material if none was loaded
//     if materials.is_empty() {
//         println!("No materials found, adding default material");
//         materials.push(Material {
//             name: "Default".to_string(),
//             diffuse_texture: None,
//             normal_texture: None,
//             metallic_texture: None,
//             roughness_texture: None,
//             ambient: [0.1, 0.1, 0.1],
//             diffuse: [0.7, 0.7, 0.7],
//             specular: [1.0, 1.0, 1.0],
//             metallic: 0.0,
//             roughness: 0.5,
//             shininess: 32.0,
//         });
//     }

//     // Process each mesh in the obj file
//     #[allow(unused, dead_code)]
//     for (i, model) in obj_models.iter().enumerate() {
//         let mesh = &model.mesh;

//         // Get material id, or use default material
//         let material_id = mesh.material_id.unwrap_or(0).min(materials.len() - 1);

//         let mut vertices = Vec::new();
        
//         // Combine position, normal, and texture coordinates into our custom Vertex type
//         for i in 0..mesh.positions.len() / 3 {
//             // Ensure we don't exceed array bounds
//             if i * 3 + 2 >= mesh.positions.len() {
//                 break;
//             }
            
//             let pos = [
//                 mesh.positions[i * 3],
//                 mesh.positions[i * 3 + 1],
//                 mesh.positions[i * 3 + 2],
//             ];
            
//             let normal = if !mesh.normals.is_empty() && i * 3 + 2 < mesh.normals.len() {
//                 [
//                     mesh.normals[i * 3],
//                     mesh.normals[i * 3 + 1],
//                     mesh.normals[i * 3 + 2],
//                 ]
//             } else {
//                 [0.0, 1.0, 0.0]
//             };
            
//             let tex_coords = if !mesh.texcoords.is_empty() && i * 2 + 1 < mesh.texcoords.len() {
//                 [
//                     mesh.texcoords[i * 2],
//                     1.0 - mesh.texcoords[i * 2 + 1], // Flip Y axis for texture coords
//                 ]
//             } else {
//                 [0.0, 0.0]
//             };
            
//             vertices.push(Vertex {
//                 position: pos,
//                 normal,
//                 tex_coords,
//             });
//         }

//         // println!("Vertices: {:?}", vertices);

//         mesh_data.push(MeshData {
//             vertices,
//             indices: mesh.indices.clone(),
//             material_id,
//         });
//     }

//     // Create textures for each material
//     let mut material_instances = Vec::new();
    
//     for (i, material) in materials.iter().enumerate() {
//         // Load diffuse texture
//         let (diffuse_view, diffuse_sampler) = if let Some(path) = &material.diffuse_texture {
//             println!("Loading diffuse texture: {:?}", path);
            
//             match std::fs::read(path) {
//                 Ok(bytes) => {
//                     match Texture::from_bytes(device, queue, &bytes, &format!("Texture {}", i)) {
//                         Ok(texture) => {
//                             println!("Successfully loaded texture: {:?}", path);
//                             (texture.view, texture.sampler)
//                         },
//                         Err(e) => {
//                             println!("Failed to load texture {:?}: {:?}", path, e);
//                             // Create a colored texture from the material's diffuse color
//                             let texture = Texture::create_colored_texture(
//                                 device, 
//                                 queue, 
//                                 material.diffuse,
//                                 &format!("Color Texture {}", i)
//                             );
//                             (texture.view, texture.sampler)
//                         }
//                     }
//                 },
//                 Err(e) => {
//                     println!("Failed to read texture file {:?}: {:?}", path, e);
//                     // Create a colored texture from the material's diffuse color
//                     let texture = Texture::create_colored_texture(
//                         device, 
//                         queue, 
//                         material.diffuse,
//                         &format!("Color Texture {}", i)
//                     );
//                     (texture.view, texture.sampler)
//                 }
//             }
//         } else {
//             println!("Creating color texture for material {}: {:?}", material.name, material.diffuse);
//             // Create a colored texture from the material's diffuse color
//             let texture = Texture::create_colored_texture(
//                 device, 
//                 queue, 
//                 material.diffuse,
//                 &format!("Color Texture {}", i)
//             );
//             (texture.view, texture.sampler)
//         };
        
//         // Create the PBR settings uniform based on material properties
//         let pbr_settings = SunLight {
//             direction: [0.0, 1.0, 0.0], // Default direction (will be overridden)
//             _padding1: 0,
//             color: [1.0, 1.0, 1.0],    // Default white light
//             intensity: 1.0,
//             use_pbr: 1,                // Enable PBR by default
//             metallic_factor: material.metallic,
//             roughness_factor: material.roughness,
//             _padding2: 0.0,
//         };
        
//         // Create a buffer for material-specific PBR settings
//         let _pbr_buffer = device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("Material PBR Buffer {}", i)),
//                 contents: bytemuck::cast_slice(&[pbr_settings]),
//                 usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//             }
//         );
        
//         // Check if we have a normal map
//         let _has_normal_map = material.normal_texture.is_some();
        
//         // Create the bind group
//         let bind_group = device.create_bind_group(
//             &wgpu::BindGroupDescriptor {
//                 layout: &sun_bind_group_layout,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: 0,
//                         resource: uniform_buffer.as_entire_binding(),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 1,
//                         resource: wgpu::BindingResource::TextureView(&diffuse_view),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 2,
//                         resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 3,
//                         resource: sun_light_buffer.as_entire_binding(),
//                     },
//                 ],
//                 label: Some(&format!("sun_bind_group_{}", i)),
//             }
//         );
        
//         material_instances.push(MaterialInstance {
//             material_id: i,
//             diffuse_bind_group: bind_group,
//         });
//     }

//     // Create meshes with vertex and index buffers
//     let mut meshes = Vec::new();
    
//     for (i, mesh) in mesh_data.iter().enumerate() {
//         let vertex_buffer = device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("Vertex Buffer {}", i)),
//                 contents: cast_slice(&mesh.vertices),
//                 usage: wgpu::BufferUsages::VERTEX,
//             }
//         );
        
//         let index_buffer = device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("Index Buffer {}", i)),
//                 contents: cast_slice(&mesh.indices),
//                 usage: wgpu::BufferUsages::INDEX,
//             }
//         );
        
//         meshes.push(Mesh {
//             // name: format!("Mesh {}", i),
//             vertex_buffer,
//             index_buffer,
//             num_indices: mesh.indices.len() as u32,
//             material_id: mesh.material_id,
//         });
//     }
    
//     Ok(Model {
//         meshes,
//         materials: material_instances,
//     })
// }