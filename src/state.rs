// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendChoice {
    Auto,
    Vulkan,
    Metal,
    Dx12,
    OpenGl,
    WebGpu,
}

impl BackendChoice {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto   => "Auto (best available)",
            Self::Vulkan => "Vulkan",
            Self::Metal  => "Metal",
            Self::Dx12   => "DirectX 12",
            Self::OpenGl => "OpenGL",
            Self::WebGpu => "WebGPU",
        }
    }
    pub fn to_wgpu_backends(self) -> wgpu::Backends {
        match self {
            Self::Auto   => wgpu::Backends::all(),
            Self::Vulkan => wgpu::Backends::VULKAN,
            Self::Metal  => wgpu::Backends::METAL,
            Self::Dx12   => wgpu::Backends::DX12,
            Self::OpenGl => wgpu::Backends::GL,
            Self::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Solid,
    Wireframe,
    Points,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Cube,
    Sphere,
    Pyramid,
    Torus,
}

impl ShapeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cube    => "Cube",
            Self::Sphere  => "Sphere",
            Self::Pyramid => "Pyramid",
            Self::Torus   => "Torus (outline)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}



#[derive(Debug, Clone)]
pub struct Object3D {
    pub name:       String,
    pub shape:      ShapeKind,
    pub position:   [f32; 3],
    pub rotation:   [f32; 3],   
    pub scale:      f32,
    pub face_colors: [[f32; 4]; 6],
    pub color:      [f32; 4],   
    pub alpha:      f32,
    pub selected:   bool,
}

impl Default for Object3D {
    fn default() -> Self {
        Self {
            name:     "Object".into(),
            shape:    ShapeKind::Cube,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale:    1.0,
            face_colors: [
                [0.9, 0.2, 0.2, 1.0], 
                [0.2, 0.3, 0.9, 1.0], 
                [0.2, 0.8, 0.2, 1.0], 
                [0.9, 0.8, 0.1, 1.0], 
                [0.9, 0.5, 0.1, 1.0], 
                [0.8, 0.2, 0.8, 1.0], 
            ],
            color:    [0.6, 0.7, 0.9, 1.0],
            alpha:    1.0,
            selected: false,
        }
    }
}

pub struct AppState {
    
    pub startup_phase:      bool,
    pub backend_choice:     BackendChoice,
    pub pending_backend:    Option<BackendChoice>,  

    
    pub render_mode:        RenderMode,
    pub projection_mode:    ProjectionMode,
    pub fov_deg:            f32,
    pub cam_distance:       f32,
    pub msaa_enabled:       bool,
    pub vsync_enabled:      bool,
    pub face_culling:       bool,
    pub depth_display:      bool,
    pub wireframe_color:    [f32; 4],
    pub bg_color:           [f32; 4],

    
    pub lighting_enabled:   bool,
    pub ambient:            f32,
    pub diffuse:            f32,
    pub specular:           f32,
    pub shininess:          f32,
    pub light_pos:          [f32; 3],
    pub light_color:        [f32; 3],

    
    pub objects:            Vec<Object3D>,
    pub selected_obj:       usize,

    
    pub auto_rotate:        bool,
    pub paused:             bool,
    pub rot_speed:          [f32; 3],

    
    pub show_debug:         bool,
    pub show_axes:          bool,
    pub show_grid:          bool,
    pub show_normals:       bool,
    pub show_settings:      bool,
    pub show_keyboard_help: bool,
    pub show_info_dialog:   bool,
    pub show_perf_graph:    bool,
    pub rotation_trail:     bool,
    pub trail_len:          usize,

    
    pub fps:                u32,
    pub fps_min:            f32,
    pub fps_max:            f32,
    pub fps_avg:            f32,
    pub fps_history:        Vec<f32>,
    pub frame_time_ms:      f32,

    
    pub mouse_drag:         bool,
    pub mouse_last:         [f32; 2],

    
    pub dark_theme:         bool,

    
    pub screenshot_requested:    bool,
    pub fullscreen_requested:    bool,
    pub fullscreen_active:       bool,
    pub reset_rotation_requested:bool,
    pub export_rotation_text:    Option<String>,
    pub pipeline_dirty:          bool,   
    pub mesh_dirty:              bool,   

    
    pub adapter_info:       String,
    pub active_backend:     String,
    pub resolution:         [u32; 2],

    
    pub trail_rotations:    Vec<[f32; 3]>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut objects = Vec::new();
        let mut obj = Object3D::default();
        obj.name = "Cube 0".into();
        obj.selected = true;
        objects.push(obj);

        Self {
            startup_phase:          true,
            backend_choice:         BackendChoice::Auto,
            pending_backend:        None,

            render_mode:            RenderMode::Solid,
            projection_mode:        ProjectionMode::Perspective,
            fov_deg:                75.0,
            cam_distance:           5.0,
            msaa_enabled:           true,
            vsync_enabled:          true,
            face_culling:           true,
            depth_display:          true,
            wireframe_color:        [1.0, 1.0, 1.0, 1.0],
            bg_color:               [0.05, 0.05, 0.08, 1.0],

            lighting_enabled:       true,
            ambient:                0.15,
            diffuse:                0.8,
            specular:               0.5,
            shininess:              32.0,
            light_pos:              [3.0, 4.0, 3.0],
            light_color:            [1.0, 1.0, 1.0],

            objects,
            selected_obj:           0,

            auto_rotate:            true,
            paused:                 false,
            rot_speed:              [0.4, 0.7, 0.2],

            show_debug:             false,
            show_axes:              true,
            show_grid:              true,
            show_normals:           false,
            show_settings:          true,
            show_keyboard_help:     false,
            show_info_dialog:       false,
            show_perf_graph:        false,
            rotation_trail:         false,
            trail_len:              8,

            fps:                    0,
            fps_min:                f32::MAX,
            fps_max:                0.0,
            fps_avg:                0.0,
            fps_history:            Vec::with_capacity(120),
            frame_time_ms:          0.0,

            mouse_drag:             false,
            mouse_last:             [0.0, 0.0],

            dark_theme:             true,

            screenshot_requested:    false,
            fullscreen_requested:    false,
            fullscreen_active:       false,
            reset_rotation_requested:false,
            export_rotation_text:    None,
            pipeline_dirty:          false,
            mesh_dirty:              false,

            adapter_info:           "—".into(),
            active_backend:         "—".into(),
            resolution:             [1280, 720],

            trail_rotations:        Vec::new(),
        }
    }
}

impl AppState {
    pub fn selected_object(&self) -> Option<&Object3D> {
        self.objects.get(self.selected_obj)
    }
    pub fn selected_object_mut(&mut self) -> Option<&mut Object3D> {
        self.objects.get_mut(self.selected_obj)
    }
    pub fn reset_all(&mut self) {
        let fresh = AppState::default();
        self.render_mode         = fresh.render_mode;
        self.projection_mode     = fresh.projection_mode;
        self.fov_deg             = fresh.fov_deg;
        self.cam_distance        = fresh.cam_distance;
        self.lighting_enabled    = fresh.lighting_enabled;
        self.ambient             = fresh.ambient;
        self.diffuse             = fresh.diffuse;
        self.specular            = fresh.specular;
        self.shininess           = fresh.shininess;
        self.light_pos           = fresh.light_pos;
        self.light_color         = fresh.light_color;
        self.auto_rotate         = fresh.auto_rotate;
        self.paused              = fresh.paused;
        self.rot_speed           = fresh.rot_speed;
        self.bg_color            = fresh.bg_color;
        self.wireframe_color     = fresh.wireframe_color;
        self.show_axes           = fresh.show_axes;
        self.show_grid           = fresh.show_grid;
        self.show_normals        = fresh.show_normals;
        if let Some(obj) = self.objects.get_mut(self.selected_obj) {
            obj.rotation = [0.0; 3];
            obj.position = [0.0; 3];
            obj.scale    = 1.0;
        }
        self.pipeline_dirty = true;
        self.mesh_dirty     = true;
    }
}
