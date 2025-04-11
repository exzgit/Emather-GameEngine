# Mather Engine

Mather Engine adalah sebuah game engine 3D modular yang dibangun dengan Rust, WebGPU, dan wgpu.

## Arsitektur

Engine ini didesain dengan pendekatan modular untuk fleksibilitas dan pemeliharaan yang mudah:

```
src/
├── engine/                  # Engine core module
│   ├── core/                # Core functionality
│   │   ├── application.rs   # Main application loop and state
│   │   └── window.rs        # Window management
│   ├── renderer/            # Rendering system
│   │   ├── texture.rs       # Texture loading and management
│   │   ├── shader.rs        # Shader compilation and management
│   │   ├── pipeline.rs      # Render pipeline configuration
│   │   └── mesh.rs          # Mesh data structures and creation
│   ├── scene/               # Scene management
│   │   ├── camera.rs        # Camera controller and viewpoint
│   │   ├── model.rs         # 3D model representation
│   │   └── light.rs         # Lighting system (sun, point lights)
│   ├── resources/           # Resource loading
│   │   └── loader.rs        # Loading models, textures, etc.
│   └── input/               # Input handling
│       └── controller.rs    # Keyboard and mouse input
└── main.rs                  # Application entry point
```

## Fitur

- Renderer 3D berbasis wgpu/WebGPU
- Sistem kamera dengan kontrol mouse dan keyboard
- Sistem lighting (directional dan point lights)
- Dukungan material PBR (Physically Based Rendering)
- Loader untuk model OBJ/MTL
- Shader manager untuk pembuatan dan pengelolaan shader
- Input handling untuk keyboard, mouse, dan scrolling

## Penggunaan Dasar

```rust
use engine::core::{application::Application, window};
use winit::event_loop::EventLoop;

fn main() {
    // Create event loop and window
    let event_loop = EventLoop::new();
    let window = window::create_default_window(&event_loop);
    
    // Create and run application
    let application = pollster::block_on(Application::new(window));
    application.run(event_loop);
}
```

## Rendering Pipeline

Engine ini mendukung:

1. PBR (Physically Based Rendering)
2. Pencahayaan dinamis
3. Tekstur normal, metallic, dan roughness
4. Ambient dan diffuse lighting

## Kontrol

- WASD: Gerakan kamera
- Mouse: Rotasi kamera
- Space/Shift: Gerakan vertikal
- IJKL: Kontrol posisi matahari
- P: Toggle mode PBR
- M/N: Tingkatkan/kurangi nilai metallic
- R/F: Tingkatkan/kurangi nilai roughness 