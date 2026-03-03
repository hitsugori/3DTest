# 3DTest v1.4.0

A high-performance 3D graphics renderer built with Rust, wgpu, and egui. Features a multi-backend architecture supporting Vulkan, DirectX 12, OpenGL, and now WebGPU (WASM).

## Features

### Rendering

* **Multi-backend support**: Automatically selects the best graphics API for your platform

  * Vulkan (Linux, Windows)
  * DirectX 12 (Windows)
  * OpenGL (Cross-platform fallback)
  * WebGPU (Browser via WASM)
* **Web Build Support**: Native WebGPU backend compiled to WebAssembly
* **Multiple rendering modes**: Solid, Wireframe, and Point rendering
* **3D Primitives**: Cube, Sphere, Pyramid, Torus
* **Debugging visualization**: Grid, Axes, and Normal line rendering
* **Lighting system**: Ambient and diffuse lighting
* **Normal visualization**: View surface normals as colors
* **Transparency support**: Configurable alpha blending

### Camera & View

* **Multiple projection modes**: Perspective and Orthographic
* **Interactive camera controls**: Rotation, pan, and zoom
* **Keyboard shortcuts**: Full keyboard navigation support
* **Fullscreen mode**: Toggle fullscreen with F11

### Performance

* **Performance monitoring**: Real-time FPS display
* **Performance graph**: Visualize frame time trends
* **Debug overlay**: Display internal rendering information

### User Interface

* **Dark/Light theme**: Switchable UI themes
* **Settings panel**: Configure rendering parameters in real-time
* **Info dialog**: About and version information
* **Export functionality**: Save rotation data and other state

---

## Web Version (WebGPU + WASM)

3DTest now supports **WebGPU via WebAssembly**.

### Build for Web

Run:

```bash
build-web.cmd
```

After the build completes, the web distribution files will be located in:

```
dist/
```

### Prebuilt Web Version

A prebuilt online version is available at:

[https://3dtest.mikeden.site/](https://3dtest.mikeden.site/)

---

## Building (Native)

### Requirements

* Rust 1.70 or later
* Cargo
* For Linux: X11 development libraries

### Build Command

```bash
cargo build --release
```

The binary will be located at:

```
target/release/3DTest
```

---

## Controls

### Keyboard

* **S**: Settings toggle
* **D**: Toggle debug overlay
* **H**: Show keyboard shortcuts help
* **P**: Pause / Resume
* **R**: Reset camera to default position
* **F11**: Toggle fullscreen
* **F5**: Screenshot
* **Space**: Pause / Resume
* **ESC**: Close dialogs

### Mouse

* **Left-click + drag**: Rotate object
* **Right-click + drag**: Pan camera
* **Scroll wheel**: Zoom in/out

---

## Dependencies

Key dependencies:

* **wgpu** (0.19): Low-level graphics abstraction (Vulkan, DX12, OpenGL, WebGPU)
* **winit** (0.29): Window creation and event handling
* **egui** (0.27): Immediate-mode GUI framework
* **glam** (0.25): Linear algebra and transforms
* **bytemuck** (1.14): GPU-safe type conversions
* **image** (0.24): Image loading (PNG support)
* **pollster** (0.3): Async runtime
* **rfd** (0.14): Native file dialogs and UI
* **log** (0.4): Logging framework

---

## License

GPL-3.0-or-later
See [LICENSE](LICENSE) or [https://www.gnu.org/licenses/](https://www.gnu.org/licenses/) for details.

---

## Author

**mikedev_** (Hitsugori)
[mike@mikeden.site](mailto:mike@mikeden.site)
