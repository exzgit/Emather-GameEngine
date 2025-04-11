#[allow(unused_imports)]
use winit::{
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    dpi::PhysicalPosition,
    window::Window,
};
use std::collections::HashMap;

/// Input event types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputEvent {
    KeyPressed(VirtualKeyCode),
    KeyReleased(VirtualKeyCode),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseMoved(f64, f64),
    MouseScrolled(f32, f32),
}

/// Key state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Input controller for handling input events
pub struct InputController {
    // Keyboard state
    keys: HashMap<VirtualKeyCode, KeyState>,
    
    // Mouse state
    mouse_buttons: HashMap<MouseButton, KeyState>,
    mouse_position: PhysicalPosition<f64>,
    mouse_previous_position: PhysicalPosition<f64>,
    mouse_delta: (f64, f64),
    
    // Scroll state
    scroll_delta: (f32, f32),
    
    // Event queue for this frame
    events: Vec<InputEvent>,
}
#[allow(dead_code)]
impl InputController {
    /// Create a new input controller
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            mouse_buttons: HashMap::new(),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_previous_position: PhysicalPosition::new(0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            events: Vec::new(),
        }
    }
    
    /// Process a window event
    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    virtual_keycode: Some(key_code),
                    state,
                    ..
                },
                ..
            } => {
                let key_state = match state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                };
                
                // Add to event queue
                let event = match key_state {
                    KeyState::Pressed => InputEvent::KeyPressed(*key_code),
                    KeyState::Released => InputEvent::KeyReleased(*key_code),
                };
                self.events.push(event);
                
                // Update key state
                self.keys.insert(*key_code, key_state);
                
                true
            },
            WindowEvent::MouseInput { 
                button, 
                state, 
                .. 
            } => {
                let button_state = match state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                };
                
                // Add to event queue
                let event = match button_state {
                    KeyState::Pressed => InputEvent::MousePressed(*button),
                    KeyState::Released => InputEvent::MouseReleased(*button),
                };
                self.events.push(event);
                
                // Update button state
                self.mouse_buttons.insert(*button, button_state);
                
                true
            },
            WindowEvent::CursorMoved { position, .. } => {
                // Calculate delta
                self.mouse_delta = (
                    position.x - self.mouse_position.x,
                    position.y - self.mouse_position.y
                );
                
                // Save previous position
                self.mouse_previous_position = self.mouse_position;
                
                // Update current position
                self.mouse_position = *position;
                
                // Add to event queue
                self.events.push(InputEvent::MouseMoved(position.x, position.y));
                
                true
            },
            WindowEvent::MouseWheel { delta, .. } => {
                // Convert delta to a consistent format
                let (x, y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                
                // Update scroll delta
                self.scroll_delta = (x, y);
                
                // Add to event queue
                self.events.push(InputEvent::MouseScrolled(x, y));
                
                true
            },
            _ => false,
        }
    }
    
    /// Update the input controller state
    pub fn update(&mut self) {
        // Clear events for next frame
        self.events.clear();
        
        // Reset deltas
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }
    
    /// Check if a key is pressed
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        matches!(self.keys.get(&key), Some(KeyState::Pressed))
    }
    
    /// Check if a mouse button is pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        matches!(self.mouse_buttons.get(&button), Some(KeyState::Pressed))
    }
    
    /// Get the current mouse position
    pub fn mouse_position(&self) -> PhysicalPosition<f64> {
        self.mouse_position
    }
    
    /// Get the mouse delta for this frame
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }
    
    /// Get the scroll delta for this frame
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }
    
    /// Get all events for this frame
    pub fn events(&self) -> &[InputEvent] {
        &self.events
    }
} 