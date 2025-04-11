use cgmath::{Point3, Deg, Rad};
use bytemuck::{Pod, Zeroable};
use winit::event::{VirtualKeyCode, ElementState};

/// Light trait for common light operations
#[allow(dead_code)]
pub trait Light {
    fn get_color(&self) -> [f32; 3];
    fn set_color(&mut self, color: [f32; 3]);
    fn get_intensity(&self) -> f32;
    fn set_intensity(&mut self, intensity: f32);
}

/// Represents a directional light source (sun)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunLight {
    pub direction: [f32; 3],
    _padding1: u32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub use_pbr: u32,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    _padding2: f32,
}

impl SunLight {
    /// Create a new sun light
    pub fn new() -> Self {
        Self {
            direction: [-0.5, -1.0, -0.3], // Default direction pointing down and slightly forward
            _padding1: 0,
            color: [1.0, 1.0, 1.0], // White light
            intensity: 1.0,
            use_pbr: 0, // Not using PBR by default
            metallic_factor: 0.5,
            roughness_factor: 0.5,
            _padding2: 0.0,
        }
    }
    
    /// Create a default sun light
    pub fn default() -> Self {
        Self {
            direction: [0.0, -1.0, 0.0], // Top-down light
            _padding1: 0,
            color: [1.0, 1.0, 1.0],     // White light
            intensity: 1.0,
            use_pbr: 1,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            _padding2: 0.0,
        }
    }
}

impl Light for SunLight {
    fn get_color(&self) -> [f32; 3] {
        self.color
    }
    
    fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
    
    fn get_intensity(&self) -> f32 {
        self.intensity
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
}

/// Sun light controller for interactive control
pub struct SunController {
    pub sun_light: SunLight,
    elevation: f32, // vertical angle (in degrees)
    azimuth: f32,   // horizontal angle (in degrees)
}

impl SunController {
    /// Create a new sun controller
    pub fn new() -> Self {
        // Initial angles that correspond to the default direction
        let elevation = 120.0; // Pointing down at 60 degrees
        let azimuth = 210.0;  // Pointing slightly forward

        let mut controller = Self {
            sun_light: SunLight::new(),
            elevation,
            azimuth,
        };

        // Update direction to match the initial angles
        controller.update_direction();
        controller
    }
    
    /// Update the sun's direction based on elevation and azimuth angles
    pub fn update_direction(&mut self) {
        // Convert angles to radians
        let elevation_rad = Rad::from(Deg(self.elevation));
        let azimuth_rad = Rad::from(Deg(self.azimuth));

        // Calculate direction vector from angles
        let x = azimuth_rad.0.sin() * elevation_rad.0.cos();
        let y = -elevation_rad.0.sin(); // Negative for downward direction
        let z = azimuth_rad.0.cos() * elevation_rad.0.cos();

        self.sun_light.direction = [x, y, z];
    }

    /// Process keyboard input to control the sun
    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        if state != ElementState::Pressed {
            return false;
        }

        let mut handled = true;
        const ANGLE_DELTA: f32 = 5.0;
        const INTENSITY_DELTA: f32 = 0.1;

        match key {
            // Sun elevation (up/down)
            VirtualKeyCode::I => {
                self.elevation = (self.elevation + ANGLE_DELTA).min(90.0);
                self.update_direction();
            }
            VirtualKeyCode::K => {
                self.elevation = (self.elevation - ANGLE_DELTA).max(0.0);
                self.update_direction();
            }
            // Sun azimuth (left/right)
            VirtualKeyCode::J => {
                self.azimuth = (self.azimuth + ANGLE_DELTA) % 360.0;
                self.update_direction();
            }
            VirtualKeyCode::L => {
                self.azimuth = (self.azimuth - ANGLE_DELTA + 360.0) % 360.0;
                self.update_direction();
            }
            // Sun intensity
            VirtualKeyCode::U => {
                self.sun_light.intensity = (self.sun_light.intensity + INTENSITY_DELTA).min(2.0);
            }
            VirtualKeyCode::O => {
                self.sun_light.intensity = (self.sun_light.intensity - INTENSITY_DELTA).max(0.0);
            }
            _ => handled = false,
        }

        handled
    }
}

/// Point light with position
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PointLight {
    pub position: [f32; 3],
    _padding: u32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    _padding2: [f32; 2],
}

#[allow(dead_code)]
impl PointLight {
    /// Create a new point light
    pub fn new(position: Point3<f32>, color: [f32; 3], intensity: f32, radius: f32) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            _padding: 0,
            color,
            intensity,
            radius,
            falloff: 1.0,
            _padding2: [0.0, 0.0],
        }
    }
}

impl Light for PointLight {
    fn get_color(&self) -> [f32; 3] {
        self.color
    }
    
    fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
    
    fn get_intensity(&self) -> f32 {
        self.intensity
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
} 