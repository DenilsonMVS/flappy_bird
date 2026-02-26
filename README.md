# Flappy Bird - High-Performance Rust & OpenGL 4.6 Engine

This is a high-performance Flappy Bird clone built from the ground up in **Rust** using **Modern OpenGL (4.6 Core Profile)**. Rather than relying on high-level game engines, this project serves as a deep dive into systems programming, custom rendering pipelines, and low-level GPU memory management.

## 🚀 Engineering Highlights

### Procedural Macros & Metaprogramming
To bridge the gap between Rust's type system and OpenGL's state machine, this project utilizes custom **Procedural Macros**:
* **`#[derive(GlVertex)]`**: Automatically generates static vertex layouts and calculates attribute offsets at compile-time.
* **`#[program_interface]`**: Automates the linking of GLSL uniforms to Rust struct fields and generates type-safe setters.
* **`#[atlas_bundle]`**: Parses texture atlas JSON data during compilation to generate type-safe enums for animation frames.

### Modern Rendering Pipeline
The engine is built on **OpenGL 4.6** principles to minimize driver overhead and maximize throughput:
* **Persistent Mapped Buffers**: Uses `glMapNamedBufferRange` with persistent and coherent flags to allow the CPU to write directly to GPU memory without costly re-allocations or synchronization blocks.
* **Direct State Access (DSA)**: Manipulates textures and buffers without the traditional "bind-to-edit" bottleneck.
* **Instanced Rendering**: Draws complex scenes, including the pipe system and UI elements, using `glDrawArraysInstanced` to keep draw calls to a minimum.

### MSDF Font System
Instead of traditional bitmap fonts, the engine implements **Multi-channel Signed Distance Fields (MSDF)**:
* Text remains perfectly sharp at any scale or resolution.
* Custom fragment shaders perform median filtering on the MSDF texture to reconstruct crisp edges with hardware-accelerated anti-aliasing.

## 🎮 Game Architecture
* **State-Driven Scenes**: Decoupled game states using a `Scene` trait for `MainMenu` and `Playing` logic.
* **Procedural Obstacles**: Infinite pipe generation using a ring-buffer approach for memory efficiency.
* **Physics & Collision**: Real-time circle-to-rectangle collision detection.
* **Spatial Audio**: Integrated audio system using the `rodio` crate for low-latency sound effects.

## 🛠️ Tech Stack
* **Language**: Rust.
* **Graphics**: OpenGL 4.6 (via `gl` and `glfw` crates).
* **Math**: `nalgebra-glm` for matrix transformations and linear algebra.
* **Assets**: `msdfgen` for font generation and `image` for texture loading.

## 🚀 Getting Started

### Prerequisites
* Rust compiler.
* GPU drivers supporting OpenGL 4.6.

### Build and Run
```bash
# Clone the repository
git clone [https://github.com/DenilsonMVS/flappy_bird](https://github.com/DenilsonMVS/flappy_bird)

# Run in release mode for optimal performance and persistent mapping efficiency
cargo run --release
