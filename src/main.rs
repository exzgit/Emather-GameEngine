use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use cgmath::Vector3;

// Import the engine module
mod engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Set up window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Mather Engine - Custom Model Demo")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();
    
    // Initialize the application
    let mut app = engine::core::application::Application::new(window).await;
    
    // Add a custom model as a game object (example usage)
    app.add_game_object_with_model(
        "Cube Model", 
        "models/cube/cube.obj", 
        Vector3::new(0.0, 2.0, 0.0)
    ).await?;
    
    // Run the application
    app.run(event_loop);
    
    Ok(())
}