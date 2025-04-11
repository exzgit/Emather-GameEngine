use winit::{
    window::{Window, WindowBuilder},
    event_loop::EventLoop,
    dpi::PhysicalSize,
};

/// Window size structure
#[allow(unused)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// Window configuration
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        // Get primary monitor size if available
        let event_loop = EventLoop::<()>::new();
        let size = if let Some(monitor) = event_loop.primary_monitor() {
            let monitor_size = monitor.size();
            (monitor_size.width * 2 / 3, monitor_size.height * 2 / 3)
        } else {
            (1280, 720) // Sensible default if monitor size can't be determined
        };
        
        Self {
            title: "Mather Engine".to_string(),
            width: size.0,
            height: size.1,
            resizable: true,
            maximized: false, // Don't force maximized, use appropriate size instead
        }
    }
}

#[allow(dead_code)]
/// Create a window with the given configuration
pub fn create_window(config: &WindowConfig, event_loop: &EventLoop<()>) -> Window {
    let mut builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(PhysicalSize::new(config.width, config.height))
        .with_resizable(config.resizable);
    
    if config.maximized {
        builder = builder.with_maximized(true);
    }
    
    builder.build(event_loop).expect("Failed to create window")
}

#[allow(dead_code)]
/// Create a window with default configuration
pub fn create_default_window(event_loop: &EventLoop<()>) -> Window {
    create_window(&WindowConfig::default(), event_loop)
}

/// Create a window with a title
#[allow(dead_code)]
pub fn create_titled_window(title: &str) -> (Window, winit::event_loop::EventLoop<()>) {
    let event_loop = winit::event_loop::EventLoop::new();
    
    // Get the primary monitor's dimensions
    let primary_monitor = event_loop.primary_monitor();
    let physical_size = if let Some(monitor) = primary_monitor {
        let size = monitor.size();
        winit::dpi::PhysicalSize::new(size.width * 2 / 3, size.height * 2 / 3)
    } else {
        // Fallback to a reasonable size if can't detect monitor
        winit::dpi::PhysicalSize::new(1280, 720)
    };
    
    let wb = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(physical_size)
        .with_resizable(true);
        
    let window = wb.build(&event_loop).unwrap();
    
    (window, event_loop)
} 