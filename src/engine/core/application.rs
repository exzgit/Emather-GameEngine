#[allow(unused)]
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    dpi::PhysicalSize,
    dpi::PhysicalPosition,
};
use crate::engine::renderer::Renderer;
use crate::engine::scene::camera::Camera;
use crate::engine::scene::light::SunController;
use crate::engine::input::controller::InputController;
use std::time::{Instant, Duration};
use cgmath::{Point3, Rad, SquareMatrix};
use wgpu::util::DeviceExt;

/// Uniform buffer data structure for shader
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

/// Main application for the game engine
pub struct Application {
    window: Window,
    renderer: Renderer,
    camera: Camera,
    input: InputController,
    last_update: Instant,
    running: bool,
    sun_controller: SunController,
    sun_light_buffer: wgpu::Buffer,
    cube_uniform_buffer: wgpu::Buffer,  // Store the cube's uniform buffer
    game_objects: Vec<crate::engine::scene::GameObject>, // Game objects container
}

impl Application {
    /// Create a new application
    pub async fn new(window: Window) -> Self {
        // Set up window properties
        window.set_title("Mather Engine");
        window.set_maximized(true);
        window.set_resizable(true);
        // Create renderer
        let renderer = Renderer::new(&window).await;
        
        // Create camera
        let size = window.inner_size();
        #[allow(unused_variables)]
        let aspect_ratio = size.width as f32 / size.height as f32;
        let mut camera = Camera::new(
            Point3::new(0.0, 10.0, 10.0),  // Position higher up to see the grid better
            Rad(3.0 * std::f32::consts::PI / 2.0),
            Rad(-std::f32::consts::PI / 6.0),  // Look down more to view the grid
        );
        camera.set_aspect_ratio(size.width, size.height);
        
        // Create input controller
        let input = InputController::new();
        
        // Create sun controller and buffer
        let sun_controller = SunController::new();
        
        let sun_light_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sun Light Buffer"),
                contents: bytemuck::cast_slice(&[sun_controller.sun_light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        // Add a test cube model
        let uniform_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Uniform Buffer"),
                contents: &[0u8; 128], // Initialize with zeros
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        // Create a cube mesh with the mesh utility function
        let cube_mesh = crate::engine::renderer::mesh::Mesh::create_cube(renderer.device(), 2.0);
        
        // Create a material for the cube
        use crate::engine::renderer::texture::Texture;
        let texture = Texture::create_colored_texture(
            renderer.device(),
            renderer.queue(),
            [1.0, 1.0, 1.0], // White
            "Cube Texture"
        );
        
        // Create a bind group for the cube
        let bind_group = renderer.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cube Bind Group"),
            layout: renderer.bind_group_layout(),
            entries: &[
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: sun_light_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create material instance
        let material_instance = crate::engine::scene::model::MaterialInstance {
            material_id: 0,
            diffuse_bind_group: bind_group,
        };
        
        // Create model
        let cube_model = crate::engine::scene::Model {
            meshes: vec![cube_mesh],
            materials: vec![material_instance],
        };
        
        // Add the model to the renderer
        let mut renderer_mut = renderer;
        renderer_mut.add_model(cube_model);
        
        // Create the application
        let app = Self {
            window,
            renderer: renderer_mut,
            camera,
            input,
            last_update: Instant::now(),
            running: true,
            sun_controller,
            sun_light_buffer,
            cube_uniform_buffer: uniform_buffer,
            game_objects: Vec::new(),
        };
        
        // Initialize the cube's uniform buffer
        app.update_cube_transform();
        
        app
    }
    
    /// Update the cube's transform
    fn update_cube_transform(&self) {
        // Create uniform data with view-projection and model matrices
        #[repr(C)]
        #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Uniforms {
            view_proj: [[f32; 4]; 4],
            model: [[f32; 4]; 4],
        }
        
        // Get the camera's view-projection matrix
        let view_proj = self.camera.get_view_projection_matrix();
        
        // Create model matrix for the cube (positioned above the grid)
        let model = cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 1.0, 0.0));
        
        // Create uniforms struct
        let uniforms = Uniforms {
            view_proj: view_proj.into(),
            model: model.into(),
        };
        
        // Write to the uniform buffer
        self.renderer.queue().write_buffer(
            &self.cube_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms])
        );
    }
    
    /// Update the grid's transform
    fn update_grid_transform(&self) {
        // Create uniform data with view-projection and model matrices
        let uniforms = Uniforms {
            view_proj: self.camera.get_view_projection_matrix().into(),
            model: cgmath::Matrix4::<f32>::identity().into(),
        };
        
        // Write to the uniform buffer only once instead of in a loop
        if let Some(_) = self.renderer.get_model(0) {
            self.renderer.queue().write_buffer(
                &self.cube_uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniforms])
            );
        }
    }
    
    #[allow(dead_code)]
    /// Load a 3D model
    pub async fn load_model(&mut self, path: &str) -> anyhow::Result<()> {
        use std::path::Path;
        
        // Create uniform buffer for camera and model matrices
        let uniform_buffer = self.renderer.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 2 * 64, // Two 4x4 matrices (view-proj and model)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create model loader
        let model_loader = crate::engine::resources::ModelLoader::new(
            self.renderer.device(),
            self.renderer.queue(),
            self.renderer.bind_group_layout(),
        );
        
        // Load the model
        let model = model_loader.load_obj(
            &Path::new(path),
            &uniform_buffer,
            Some(&self.sun_light_buffer),
        ).await?;
        
        self.renderer.add_model(model);
        
        Ok(())
    }
    
    /// Run the application using the event loop
    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = if self.running {
                ControlFlow::Poll
            } else {
                ControlFlow::Exit
            };
            
            self.process_event(event, control_flow);
        });
    }
    
    /// Process events from winit
    #[allow(unused_variables)]
    fn process_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent { event, window_id } if window_id == self.window.id() => {
                // Process input
                self.input.process_event(&event);
                
                // Process camera controls
                if self.camera.process_keyboard(&event) {
                    self.window.request_redraw();
                }
                
                // Process mouse events for camera rotation
                if let WindowEvent::MouseInput { button, state, .. } = event {
                    if button == winit::event::MouseButton::Right {
                        let is_pressed = state == winit::event::ElementState::Pressed;
                        if is_pressed {
                            // Record starting position for cursor reset
                            let current_position = self.input.mouse_position();
                            // Store the initial position in the camera
                            self.camera.set_rotation_origin(current_position);
                            
                            // Hide cursor but keep it in its original position
                            let _ = self.window.set_cursor_visible(false);
                            // Use Confined mode to keep cursor within window
                            let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                            
                            // Start rotation with original cursor position
                            self.camera.start_rotation(current_position);
                        } else {
                            // End rotation
                            self.camera.end_rotation();
                            
                            // Return cursor to normal
                            let _ = self.window.set_cursor_visible(true);
                            let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                        }
                        self.window.request_redraw();
                    }
                }
                
                if let WindowEvent::CursorMoved { position, .. } = event {
                    // Check if right mouse button is pressed
                    if self.input.is_mouse_button_pressed(winit::event::MouseButton::Right) {
                        // Process mouse movement first
                        let rotation_changed = self.camera.process_mouse_move(position, true, &self.window);
                        
                        // Then reset cursor position to original, regardless of whether camera rotated
                        if let Some(origin) = self.camera.get_rotation_origin() {
                            // Set cursor back to original position
                            let _ = self.window.set_cursor_position(origin);
                        }
                        
                        // Only update transforms if camera actually rotated
                        if rotation_changed {
                            // Update transforms immediately when camera rotates
                            self.update_cube_transform();
                            self.update_grid_transform();
                            self.window.request_redraw();
                        }
                    }
                }
                
                // Process sun light controls
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    if let Some(keycode) = input.virtual_keycode {
                        if self.sun_controller.process_keyboard(keycode, input.state) {
                            self.renderer.queue().write_buffer(
                                &self.sun_light_buffer,
                                0,
                                bytemuck::cast_slice(&[self.sun_controller.sun_light])
                            );
                            self.window.request_redraw();
                        }
                    }
                }
                
                match event {
                    WindowEvent::CloseRequested => {
                        self.running = false;
                    },
                    WindowEvent::Resized(physical_size) => {
                        // Ensure minimum dimensions to prevent Vulkan errors
                        let width = physical_size.width.max(1);
                        let height = physical_size.height.max(1);
                        
                        // On some platforms like Vulkan, certain dimensions might be problematic
                        // We'll resize and let the renderer handle any fallbacks
                        self.renderer.resize(width, height);
                        self.camera.set_aspect_ratio(width, height);
                        
                        // Update transforms after resize
                        self.update_cube_transform();
                        self.update_grid_transform();
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // Ensure minimum dimensions to prevent Vulkan errors
                        let width = new_inner_size.width.max(1);
                        let height = new_inner_size.height.max(1);
                        
                        self.renderer.resize(width, height);
                        self.camera.set_aspect_ratio(width, height);
                        
                        // Update transforms after resize
                        self.update_cube_transform();
                        self.update_grid_transform();
                    },
                    _ => {}
                }
            },
            Event::MainEventsCleared => {
                // Process camera movement every frame
                if self.camera.update() {
                    // If camera moved, update all dependent objects
                    self.update_cube_transform();
                    self.update_grid_transform();
                    self.window.request_redraw();
                }
                
                // Calculate delta time
                let now = Instant::now();
                let dt = now.duration_since(self.last_update);
                self.last_update = now;
                
                // Update game state
                self.update(dt);
                
                // Request a redraw
                self.window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                // Render frame
                self.render();
            },
            _ => {}
        }
    }
    
    /// Update the application state
    #[allow(unused_variables)]
    fn update(&mut self, dt: Duration) {
        // Update camera
        if self.camera.update() {
            // Camera moved, update view matrix
            self.update_cube_transform();
            self.update_grid_transform();
        }
        
        // Update sun light buffer with current controller values
        self.renderer.queue().write_buffer(
            &self.sun_light_buffer,
            0,
            bytemuck::cast_slice(&[self.sun_controller.sun_light])
        );
        
        // Reset input state for next frame
        self.input.update();
        
        // Update game objects
        self.update_game_objects(dt.as_secs_f32());
    }
    
    /// Render the frame
    fn render(&mut self) {
        // Create a command encoder 
        let mut encoder = self.renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Add a timestamp query at the start of the frame (if available)
        // This is a placeholder for future performance metrics
        
        // Render the scene
        if let Err(e) = self.renderer.render_scene() {
            eprintln!("Failed to render scene: {:?}", e);
        }
        
        // Add a timestamp query at the end of the frame (if available)
        // This is a placeholder for future performance metrics
        
        // Submit the command buffer and make sure it contains at least one command
        // to avoid the "No work has been submitted for this frame" error
        encoder.push_debug_group("End of frame marker");
        encoder.pop_debug_group();
        
        self.renderer.queue().submit(Some(encoder.finish()));
    }
    
    /// Add a game object to the scene
    pub async fn add_game_object_with_model(&mut self, name: &str, model_path: &str, position: cgmath::Vector3<f32>) -> anyhow::Result<()> {
        // Create game object with model
        let game_object = crate::engine::scene::GameObject::with_model(
            name,
            model_path,
            position,
            self.renderer.device(),
            self.renderer.queue(),
            self.renderer.bind_group_layout(),
            Some(&self.sun_light_buffer),
        ).await?;
        
        // Add the model to the renderer
        if let Some(model_component) = game_object.get_component::<crate::engine::scene::ModelComponent>() {
            #[allow(unused)]
            if let Some(ref model) = model_component.model {
                // We can't clone the model, so we load it again for the renderer
                #[allow(unused)]
                let model_loader = crate::engine::resources::ModelLoader::new(
                    self.renderer.device(),
                    self.renderer.queue(),
                    self.renderer.bind_group_layout(),
                );
                
                // Load the model directly for the renderer
                let model_for_renderer = model_loader.load_obj(
                    std::path::Path::new(model_component.model_path.as_ref().unwrap()),
                    &model_component.uniform_buffer.as_ref().unwrap(),
                    Some(&self.sun_light_buffer),
                ).await?;
                
                self.renderer.add_model(model_for_renderer);
            }
        }
        
        // Store game object
        self.game_objects.push(game_object);
        
        Ok(())
    }
    
    /// Update all game objects
    fn update_game_objects(&mut self, dt: f32) {
        for game_object in &mut self.game_objects {
            // Create a copy of the transform by implementing Clone or manually copying values
            #[allow(unused)]
            let transform = game_object.transform.clone();
            game_object.update(dt);
            
            // Update model uniforms if has a model component
            if let Some(model_component) = game_object.get_component::<crate::engine::scene::ModelComponent>() {
                let view_proj = self.camera.get_view_projection_matrix();
                let model_matrix = game_object.transform.get_model_matrix();
                
                model_component.update_transform(self.renderer.queue(), view_proj, model_matrix);
            }
        }
    }
} 