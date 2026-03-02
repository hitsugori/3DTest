# 3DTest v1.3.1

A high-performance 3D graphics renderer built with Rust, wgpu, and egui. Features a multi-backend architecture supporting Vulkan, DirectX 12, Metal, OpenGL, and WebGPU.

## Features

### Rendering
- **Multi-backend support**: Automatically selects the best graphics API for your platform
  - Vulkan (Linux, Windows)
  - DirectX 12 (Windows)
  - OpenGL (Cross-platform fallback)
  - WebGPU (Browser, experimental)
- **Multiple rendering modes**: Solid, Wireframe, and Point rendering
- **3D Primitives**: Cube, Sphere, Pyramid, Torus
- **Debugging visualization**: Grid, Axes, and Normal line rendering
- **Lighting system**: Ambient, diffuse, blahblahblah
- **Normal visualization**: View surface normals as colors
- **Transparency support**: Configurable alpha blending

### Camera & View
- **Multiple projection modes**: Perspective and Orthographic
- **Interactive camera controls**: Rotation, pan, and zoom
- **Keyboard shortcuts**: Full keyboard navigation support
- **Fullscreen mode**: Toggle fullscreen with F11

### Performance
- **Performance monitoring**: Real-time FPS display
- **Performance graph**: Visualize frame time trends
- **Debug overlay**: Display internal rendering information

### User Interface
- **Dark/Light theme**: Switchable UI themes
- **Settings panel**: Configure rendering parameters in real-time
- **Info dialog**: About and version information
- **Export functionality**: Save rotation data and other state

## Building

### Requirements
- Rust 1.70 or later
- Cargo
- For Linux: X11 development libraries

### Build Command
```bash
cargo build --release
```

The binary will be located at `target/release/3DTest`

## Controls

### Keyboard
- **R**: Reset camera to default position
- **F11**: Toggle fullscreen
- **1-4**: Switch between shapes (Cube, Sphere, Pyramid, Torus)
- **W**: Wireframe mode
- **S**: Solid mode
- **P**: Points mode
- **G**: Toggle grid overlay
- **A**: Toggle axes overlay
- **H**: Show keyboard shortcuts help
- **I**: Show info dialog
- **D**: Toggle debug overlay
- **E**: Export rotation to clipboard

### Mouse
- **Left-click + drag**: Rotate object
- **Right-click + drag**: Pan camera
- **Scroll wheel**: Zoom in/out

## Architecture

### Module Structure

#### `main.rs`
- Application entry point
- Event loop management
- Backend selection and initialization
- Error handling and panic hooks

#### `renderer.rs`
- Core rendering pipeline using wgpu
- GPU resource management (buffers, textures, pipelines)
- Shader compilation and management
- Matrix transformations and lighting calculations
- Supports multiple render modes (solid, wireframe, points)

#### `mesh.rs`
- Mesh generation algorithms
- Primitive shapes: Cube, Sphere, Pyramid, Torus
- Helper geometries: Grid, Axes
- Normal visualization meshes
- Vertex structure and buffer layout definitions

#### `state.rs`
- Application state management
- Configuration enums (RenderMode, ProjectionMode, ShapeKind, BackendChoice)
- Input handling state
- Camera and lighting parameters
- UI state management

#### `ui.rs`
- ImGUI interface using egui
- Settings panel for runtime configuration
- Performance graph and debug overlay
- Keyboard help dialog
- Theme management (light/dark)
- Toast notifications

## Dependencies

Key dependencies:
- **wgpu** (0.19): Low-level graphics abstraction
- **winit** (0.29): Window creation and event handling
- **egui** (0.27): Immediate-mode GUI framework
- **glam** (0.25): Linear algebra and transforms
- **bytemuck** (1.14): GPU-safe type conversions
- **image** (0.24): Image loading (PNG support)
- **pollster** (0.3): Async runtime
- **rfd** (0.14): Native file dialogs and UI
- **log** (0.4): Logging framework

## License

GPL-3.0-or-later - See [LICENSE](LICENSE) or https://www.gnu.org/licenses/ for details

## Author

**mikedev_** (Hitsugori) - mike@mikeden.site
