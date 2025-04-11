#[allow(unused_imports)]
use cgmath::{Point3, Vector3, Matrix4, Rad, Deg, perspective, InnerSpace, Zero};
#[allow(unused_imports)]
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, VirtualKeyCode, MouseButton, WindowEvent},
    window::Window,
};
#[allow(unused_imports)]
use std::time::{Instant, Duration};
use std::f32::consts::PI;

#[allow(dead_code)]
/// Camera Controller for handling camera movement
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub roll: Rad<f32>,
    
    speed: f32,
    sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_roll_left_pressed: bool,
    is_roll_right_pressed: bool,
    last_mouse_pos: Option<PhysicalPosition<f64>>,
    initial_mouse_pos: Option<PhysicalPosition<f64>>,
    rotation_origin: Option<PhysicalPosition<f64>>, // Store the origin point for cursor reset
    last_update: Instant,
    last_rotation_update: Instant, // Separate timestamp for rotation
    
    pub aspect_ratio: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

#[allow(dead_code)]
impl Camera {
    /// Create a new camera
    pub fn new(position: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Self {
        let now = Instant::now();
        Self {
            position,
            yaw,
            pitch,
            roll: Rad(0.0),
            speed: 6.0,
            sensitivity: 0.005,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_roll_left_pressed: false,
            is_roll_right_pressed: false,
            last_mouse_pos: None,
            initial_mouse_pos: None,
            rotation_origin: None,
            last_update: now,
            last_rotation_update: now,
            aspect_ratio: 1.0,
            fovy: Rad(70.0 * PI / 180.0),
            znear: 0.001,
            zfar: 300.0,
        }
    }
    
    /// Process keyboard input
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    virtual_keycode: Some(key_code),
                    state,
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match key_code {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::E => {
                        self.is_up_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::Scroll => {
                        self.is_roll_right_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::Q => {
                        self.is_down_pressed = is_pressed;
                        true
                    },
                    // VirtualKeyCode::Q => {
                    //     self.is_roll_left_pressed = is_pressed;
                    //     true
                    // },
                    // VirtualKeyCode::E => {
                    //     self.is_roll_right_pressed = is_pressed;
                    //     true
                    // },
                    _ => false,
                }
            },
            _ => false,
        }
    }
    
    /// Process mouse movement with improved smoothness
    pub fn process_mouse_move(&mut self, position: PhysicalPosition<f64>, mouse_pressed: bool, _window: &Window) -> bool {
        if mouse_pressed {
            // Get delta movement from last position
            if let Some(last_position) = self.last_mouse_pos {
                // Calculate delta from last position (before it's reset)
                let dx = position.x - last_position.x;
                let dy = position.y - last_position.y;
                
                // Only update if there's actual movement
                if dx.abs() > 0.0 || dy.abs() > 0.0 {
                    // Update time tracking
                    let now = Instant::now();
                    #[allow(unused)]
                    let dt = now.duration_since(self.last_rotation_update).as_secs_f32();
                    self.last_rotation_update = now;
                    
                    // Apply sensitivity - increased for better response
                    let rotation_scale = self.sensitivity;
                    
                    // Update camera angles based on deltas
                    self.yaw += Rad((dx as f32) * rotation_scale);
                    self.pitch -= Rad((dy as f32) * rotation_scale);
                    
                    // Set stricter limits for up/down rotation (pitch)
                    // Using 70 degrees (in radians) as the maximum up/down angle
                    self.pitch = Rad(self.pitch.0.clamp(-self.fovy.0, self.fovy.0));
                    
                    // When cursor is reset to origin by the application,
                    // we need to update our last_mouse_pos to that origin
                    if let Some(origin) = self.rotation_origin {
                        self.last_mouse_pos = Some(origin);
                    }
                    
                    return true;
                }
            } else {
                // If no last position (first movement), use the current as reference
                self.last_mouse_pos = Some(position);
            }
        } else {
            // Reset tracking when not pressed
            self.last_mouse_pos = None;
        }
        
        false
    }
    
    /// Set the rotation origin point for cursor reset
    pub fn set_rotation_origin(&mut self, position: PhysicalPosition<f64>) {
        self.rotation_origin = Some(position);
    }
    
    /// Get the rotation origin point
    pub fn get_rotation_origin(&self) -> Option<PhysicalPosition<f64>> {
        self.rotation_origin
    }
    
    /// Start camera rotation
    pub fn start_rotation(&mut self, position: PhysicalPosition<f64>) -> bool {
        // Set initial position for first frame
        self.last_mouse_pos = Some(position);
        // Also set as rotation origin if not already set
        if self.rotation_origin.is_none() {
            self.rotation_origin = Some(position);
        }
        true
    }
    
    /// End camera rotation
    pub fn end_rotation(&mut self) -> bool {
        // Clear mouse position when ending rotation
        self.last_mouse_pos = None;
        self.rotation_origin = None;
        true
    }
    
    /// Update camera position based on keyboard input
    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        // Cap delta time to prevent large jumps after lag spikes
        let dt = dt.min(0.05);
        
        let mut changed = false;
        
        // Handle roll rotation with smooth interpolation
        // MARK: HANDLE ROLL ROTATION
        if self.is_roll_left_pressed {
            self.roll += Rad(2.0 * dt);
            changed = true;
        }
        if self.is_roll_right_pressed {
            self.roll -= Rad(2.0 * dt);
            changed = true;
        }
        
        // Ensure pitch stays within limits (same as in mouse movement)
        self.pitch = Rad(self.pitch.0.clamp(-self.fovy.0, self.fovy.0));
        
        // Pre-calculate view vectors only once
        let forward = Vector3::new(
            self.yaw.0.cos() * self.pitch.0.cos(),
            self.pitch.0.sin(),
            self.yaw.0.sin() * self.pitch.0.cos(),
        ).normalize();
        
        let right = forward.cross(Vector3::unit_y()).normalize();
        let up = right.cross(forward).normalize();
        
        // Apply roll rotation to right and up vectors
        let cos_roll = self.roll.0.cos();
        let sin_roll = self.roll.0.sin();
        let rolled_right = right * cos_roll + up * sin_roll;
        let rolled_up = -right * sin_roll + up * cos_roll;
        
        // Apply movement based on pressed keys with smooth acceleration
        let mut movement = Vector3::zero();
        
        if self.is_forward_pressed {
            movement += forward;
            changed = true;
        }
        if self.is_backward_pressed {
            movement -= forward;
            changed = true;
        }
        if self.is_right_pressed {
            movement += rolled_right;
            changed = true;
        }
        if self.is_left_pressed {
            movement -= rolled_right;
            changed = true;
        }
        if self.is_up_pressed {
            movement += rolled_up;
            changed = true;
        }
        if self.is_down_pressed {
            movement -= rolled_up;
            changed = true;
        }
        
        // Update position with smooth movement
        if movement.magnitude2() > 0.0 {
            // Normalize movement vector to ensure consistent speed in all directions
            movement = movement.normalize();
            
            // Apply movement with constant velocity regardless of framerate
            self.position += movement * self.speed * dt;
            changed = true;
        }
        
        changed
    }
    
    #[allow(dead_code, unused)]
    /// Get the view matrix for rendering
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let forward = Vector3::new(
            self.yaw.0.cos() * self.pitch.0.cos(),
            self.pitch.0.sin(),
            self.yaw.0.sin() * self.pitch.0.cos(),
        ).normalize();
        
        let right = forward.cross(Vector3::unit_y()).normalize();
        let up = right.cross(forward).normalize();
        
        // Apply roll rotation to right and up vectors
        let cos_roll = self.roll.0.cos();
        let sin_roll = self.roll.0.sin();
        let rolled_right = right * cos_roll + up * sin_roll;
        let rolled_up = -right * sin_roll + up * cos_roll;
        
        // Create view matrix with roll
        let mut view = Matrix4::look_at_rh(self.position, self.position + forward, rolled_up);
        view
    }
    
    /// Get the projection matrix for rendering
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect_ratio, self.znear, self.zfar)
    }
    
    /// Get the combined view-projection matrix
    pub fn get_view_projection_matrix(&self) -> Matrix4<f32> {
        self.get_projection_matrix() * self.get_view_matrix()
    }
    
    /// Set the aspect ratio (usually when window is resized)
    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }
} 