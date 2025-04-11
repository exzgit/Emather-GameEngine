# Mather Engine

**Mather Engine** is a modular 3D game engine built with **Rust**, powered by **WebGPU** and the `wgpu` graphics API. This engine is currently under **active development**, and its current focus is on implementing core 3D rendering features such as **lighting**, **FPS-style camera control**, and **rendering static `.obj` models**.

---

## 🚧 Project Status

> ⚠️ **Work in Progress**  
> This engine is in early development. Core systems like lighting, camera movement, and rendering `.obj` models are functional, but many advanced features (physics, animation, UI, networking) are still under construction.

---

## 🔧 Architecture Overview

The engine is designed using a **modular architecture** to improve maintainability, scalability, and reusability:

```
src/
├── engine/                  # Main engine module
│   ├── core/                # Application loop & windowing
│   │   ├── application.rs   # Main game loop and state management
│   │   └── window.rs        # Cross-platform window creation
│   ├── renderer/            # Rendering system
│   │   ├── texture.rs       # Texture loading and management
│   │   ├── shader.rs        # Shader compilation and hot-reloading
│   │   ├── pipeline.rs      # Render pipeline setup (PBR, lighting)
│   │   └── mesh.rs          # Mesh loading and rendering logic
│   ├── scene/               # Scene graph and 3D objects
│   │   ├── camera.rs        # FPS-style camera controller
│   │   ├── model.rs         # 3D model abstraction (.obj loader)
│   │   └── light.rs         # Directional and point lights
│   ├── resources/           # Asset loader module
│   │   └── loader.rs        # Model and texture loading
│   └── input/               # Input and control mapping
│       └── controller.rs    # Mouse and keyboard event handler
└── main.rs                  # Application entry point
```

---

## ✨ Features

- ⚙️ Built entirely in **Rust** for safety and performance  
- 🔆 Basic **lighting system** (directional and point lights)  
- 🎥 **First-person camera** movement using mouse + keyboard  
- 🧱 Basic **PBR (Physically Based Rendering)** support  
- 📦 `.obj` and `.mtl` **model loading**  
- 🎮 Input handling via `winit` (keyboard, mouse, scroll)  
- 🧪 Simple shader system with live reloading

---

## 🚀 Getting Started

```rust
use engine::core::{application::Application, window};
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new();
    let window = window::create_default_window(&event_loop);

    let application = pollster::block_on(Application::new(window));
    application.run(event_loop);
}
```

---

## 🎮 Controls

- `W`, `A`, `S`, `D`: Move camera (FPS style)  
- Mouse Movement: Rotate camera view  
- `Space` / `Shift`: Move camera vertically  
- `I`, `J`, `K`, `L`: Adjust sunlight direction  
- `P`: Toggle PBR mode on/off  
- `M` / `N`: Increase / decrease metallic value  
- `R` / `F`: Increase / decrease roughness value

---

## 🔬 Rendering Pipeline

- ✅ **Physically Based Rendering (PBR)**  
- ✅ Support for **diffuse**, **metallic**, **roughness**, and **normal maps**  
- ✅ **Dynamic lighting** with directional and point light sources  
- ✅ Basic **camera and projection systems** (perspective)

---

## 📌 Goals for Future Development

- Shadow mapping and global illumination  
- Animation system (skeletal + keyframe)  
- Entity-Component-System (ECS) integration  
- In-editor scene graph and GUI  
- Audio, physics, and scripting support  

---

## 📖 Dependencies

- [`wgpu`](https://github.com/gfx-rs/wgpu) – WebGPU implementation in Rust  
- [`winit`](https://github.com/rust-windowing/winit) – Windowing and input  
- [`tobj`](https://crates.io/crates/tobj) – OBJ file loader  
- [`glam`](https://crates.io/crates/glam) – Math library for 3D graphics  
