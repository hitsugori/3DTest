// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
use std::sync::Arc;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use chrono;

use crate::state::{AppState, RenderMode, ProjectionMode, ShapeKind};
use crate::mesh::{Vertex, MeshData, generate_cube, generate_sphere,
                  generate_pyramid, generate_torus, generate_axes,
                  generate_grid, generate_normal_lines};
const SHADER_SRC: &str = r#"
struct Uniforms {
    mvp        : mat4x4<f32>,
    model      : mat4x4<f32>,
    normal_mat : mat4x4<f32>,
    light_pos  : vec3<f32>,
    ambient    : f32,
    light_color: vec3<f32>,
    diffuse_s  : f32,
    cam_pos    : vec3<f32>,
    specular_s : f32,
    shininess  : f32,
    alpha      : f32,
    use_light  : u32,
    show_normals: u32,
}

@group(0) @binding(0) var<uniform> u : Uniforms;

struct VIn {
    @location(0) pos    : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) color  : vec4<f32>,
}

struct VOut {
    @builtin(position) clip : vec4<f32>,
    @location(0) wpos  : vec3<f32>,
    @location(1) wnorm : vec3<f32>,
    @location(2) color : vec4<f32>,
}

@vertex
fn vs_main(v: VIn) -> VOut {
    var o : VOut;
    let wp   = (u.model * vec4<f32>(v.pos, 1.0)).xyz;
    o.clip   = u.mvp * vec4<f32>(v.pos, 1.0);
    o.wpos   = wp;
    o.wnorm  = normalize((u.normal_mat * vec4<f32>(v.normal, 0.0)).xyz);
    o.color  = v.color;
    return o;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    if u.show_normals != 0u {
        return vec4<f32>(v.wnorm * 0.5 + 0.5, 1.0);
    }

    let base = v.color.rgb;

    if u.use_light == 0u {
        return vec4<f32>(base, v.color.a * u.alpha);
    }

    let N   = normalize(v.wnorm);
    let L   = normalize(u.light_pos - v.wpos);
    let V   = normalize(u.cam_pos   - v.wpos);
    let R   = reflect(-L, N);

    let amb  = u.ambient   * u.light_color * base;
    let diff = max(dot(N, L), 0.0) * u.diffuse_s  * u.light_color * base;
    let sf   = pow(max(dot(V, R), 0.0), u.shininess);
    let spec = u.specular_s * sf * u.light_color;

    let col  = clamp(amb + diff + spec, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(col, v.color.a * u.alpha);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Default)]
struct Uniforms {
    mvp:          [[f32; 4]; 4],
    model:        [[f32; 4]; 4],
    normal_mat:   [[f32; 4]; 4],
    light_pos:    [f32; 3],
    ambient:      f32,
    light_color:  [f32; 3],
    diffuse_s:    f32,
    cam_pos:      [f32; 3],
    specular_s:   f32,
    shininess:    f32,
    alpha:        f32,
    use_light:    u32,
    show_normals: u32,
}

struct GpuMesh {
    vbuf:       wgpu::Buffer,
    ibuf:       wgpu::Buffer,
    idx_count:  u32,
    line_ibuf:  Option<(wgpu::Buffer, u32)>,  
}

impl GpuMesh {
    fn from_mesh(device: &wgpu::Device, mesh: &MeshData) -> Self {
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("vbuf"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage:    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("ibuf"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage:    wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        GpuMesh {
            vbuf,
            ibuf,
            idx_count: mesh.indices.len() as u32,
            line_ibuf: None,
        }
    }
    fn from_line_mesh(device: &wgpu::Device, mesh: &MeshData) -> Self {
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("line_vbuf"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage:    wgpu::BufferUsages::VERTEX,
        });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("line_ibuf"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage:    wgpu::BufferUsages::INDEX,
        });
        GpuMesh {
            vbuf,
            ibuf,
            idx_count: mesh.indices.len() as u32,
            line_ibuf: None,
        }
    }
}

pub struct BareGpu {
    pub device:                 wgpu::Device,
    pub queue:                  wgpu::Queue,
    pub surface:                wgpu::Surface<'static>,
    pub surface_config:         wgpu::SurfaceConfiguration,
    pub surface_format:         wgpu::TextureFormat,
    pub egui_renderer:          egui_wgpu::Renderer,
    pub adapter_info:           wgpu::AdapterInfo,
    pub supports_polygon_line:  bool,
    pub supports_polygon_point: bool,
    pub max_msaa_samples:       u32,
}

impl BareGpu {
    pub fn new(
        window: Arc<winit::window::Window>,
        choice: crate::state::BackendChoice,
    ) -> Result<Self, String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends:             choice.to_wgpu_backends(),
            dx12_shader_compiler: Default::default(),
            flags:                wgpu::InstanceFlags::default(),
            gles_minor_version:   wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Failed to create window surface: {e}"))?;
        let adapter = instance
            .enumerate_adapters(choice.to_wgpu_backends())
            .into_iter()
            .filter(|a| a.is_surface_supported(&surface))
            .max_by_key(|a| match a.get_info().device_type {
                wgpu::DeviceType::DiscreteGpu   => 2,
                wgpu::DeviceType::IntegratedGpu => 1,
                _                               => 0,
            })
            .ok_or_else(|| format!(
                "No GPU adapter found for backend '{}'.                  Try a different backend or check your GPU drivers.",
                choice.label()
            ))?;

        let adapter_info = adapter.get_info();
        log::info!("Selected adapter: {} ({:?})", adapter_info.name, adapter_info.backend);

        let supports_polygon_line  = adapter.features().contains(wgpu::Features::POLYGON_MODE_LINE);
        let supports_polygon_point = adapter.features().contains(wgpu::Features::POLYGON_MODE_POINT);

        let mut required_features = wgpu::Features::empty();
        if supports_polygon_line  { required_features |= wgpu::Features::POLYGON_MODE_LINE; }
        if supports_polygon_point { required_features |= wgpu::Features::POLYGON_MODE_POINT; }

        
        
        let desc = wgpu::DeviceDescriptor {
            label:             Some("bare_device"),
            required_features,
            required_limits:   wgpu::Limits::default(),
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&desc, None))
            .map_err(|e| format!("Failed to create GPU device: {e}"))?;

        let surface_caps   = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        
        let copy_src_supported = surface_caps.usages.contains(wgpu::TextureUsages::COPY_SRC);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | if copy_src_supported { wgpu::TextureUsages::COPY_SRC } else { wgpu::TextureUsages::empty() },
            format:                        surface_format,
            width:                         size.width.max(1),
            height:                        size.height.max(1),
            present_mode:                  wgpu::PresentMode::AutoVsync,
            alpha_mode:                    surface_caps.alpha_modes[0],
            view_formats:                  vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);

        
        let fmt_flags = adapter.get_texture_format_features(surface_format).flags;
        let max_msaa_samples: u32 = if fmt_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) { 4 } else { 1 };
        log::info!("Max MSAA samples for {:?}: {}", surface_format, max_msaa_samples);

        Ok(Self {
            device, queue, surface, surface_config, surface_format,
            egui_renderer, adapter_info,
            supports_polygon_line, supports_polygon_point,
            max_msaa_samples,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
        self.surface_config.width  = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render_startup(
        &mut self,
        egui_ctx:    &egui::Context,
        full_output: &egui::FullOutput,
        window:      &winit::window::Window,
    ) {
        let surface_tex = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(e) => { log::error!("Startup surface error: {:?}", e); return; }
        };

        let view = surface_tex.texture.create_view(&Default::default());
        let pixels_per_point = window.scale_factor() as f32;
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels:  [self.surface_config.width, self.surface_config.height],
            pixels_per_point,
        };
        let primitives = egui_ctx.tessellate(
            full_output.shapes.clone(),
            full_output.pixels_per_point,
        );

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("startup_enc") }
        );
        self.egui_renderer.update_buffers(
            &self.device, &self.queue, &mut encoder, &primitives, &screen_desc,
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("startup_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.egui_renderer.render(&mut pass, &primitives, &screen_desc);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_tex.present();
    }
}



pub struct Renderer {
    pub device:         wgpu::Device,
    pub queue:          wgpu::Queue,
    surface:            wgpu::Surface<'static>,
    surface_config:     wgpu::SurfaceConfiguration,

    
    solid_pipeline:     wgpu::RenderPipeline,
    line_pipeline:      wgpu::RenderPipeline,
    point_pipeline:     Option<wgpu::RenderPipeline>,

    
    uniform_buf:        wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    bind_group_layout:  wgpu::BindGroupLayout,

    
    depth_view:         wgpu::TextureView,
    ms_view:            Option<wgpu::TextureView>,
    pub msaa_samples:       u32,
    max_msaa_samples:       u32,  

    
    cube_mesh:          GpuMesh,
    sphere_mesh:        GpuMesh,
    pyramid_mesh:       GpuMesh,
    torus_mesh:         GpuMesh,
    axes_mesh:          GpuMesh,
    grid_mesh:          GpuMesh,
    normals_mesh:       GpuMesh,    

    surface_format:     wgpu::TextureFormat,

    
    pub egui_renderer:  egui_wgpu::Renderer,

    
    screenshot_buf:     Option<(wgpu::Buffer, u32, u32)>,

    pub adapter_info:   wgpu::AdapterInfo,
    pub supports_polygon_line:  bool,
    pub supports_polygon_point: bool,
}

impl Renderer {
    
    
    pub fn new(window: Arc<winit::window::Window>, state: &AppState) -> Result<Self, String> {
        let bare = BareGpu::new(window, state.backend_choice)?;
        Self::from_bare_gpu(bare, state)
    }

    
    
    pub fn from_bare_gpu(bare: BareGpu, state: &AppState) -> Result<Self, String> {
        let BareGpu {
            device,
            queue,
            surface,
            mut surface_config,
            surface_format,
            adapter_info,
            supports_polygon_line,
            supports_polygon_point,
            max_msaa_samples,
            egui_renderer,    
        } = bare;

        let msaa_samples: u32 = if state.msaa_enabled && max_msaa_samples >= 4 { 4 } else { 1 };

        
        surface_config.present_mode = if state.vsync_enabled {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        
        let copy_src_supported = surface_config.usage.contains(wgpu::TextureUsages::COPY_SRC);
        surface_config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | if copy_src_supported { wgpu::TextureUsages::COPY_SRC } else { wgpu::TextureUsages::empty() };
        surface.configure(&device, &surface_config);

        let w = surface_config.width;
        let h = surface_config.height;

        
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ubl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding:    0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty:         wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                }],
            }
        );

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("uniform_buf"),
            size:               std::mem::size_of::<Uniforms>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("ubg"),
            layout:  &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label:                Some("pl"),
                bind_group_layouts:   &[&bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let depth_stencil = wgpu::DepthStencilState {
            format:              wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare:       wgpu::CompareFunction::Less,
            stencil:             wgpu::StencilState::default(),
            bias:                wgpu::DepthBiasState::default(),
        };

        let make_pipeline = |topology: wgpu::PrimitiveTopology,
                              poly_mode: wgpu::PolygonMode,
                              cull: Option<wgpu::Face>|
                             -> wgpu::RenderPipeline
        {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label:  Some("pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module:      &shader,
                    entry_point: "vs_main",
                    buffers:     &[Vertex::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology,
                    strip_index_format: None,
                    front_face:         wgpu::FrontFace::Ccw,
                    cull_mode:          cull,
                    polygon_mode:       poly_mode,
                    unclipped_depth:    false,
                    conservative:       false,
                },
                depth_stencil: Some(depth_stencil.clone()),
                multisample:   wgpu::MultisampleState {
                    count:                  msaa_samples,
                    mask:                   !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module:      &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend:  Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };

        let cull = if state.face_culling { Some(wgpu::Face::Back) } else { None };
        let solid_pipeline = make_pipeline(
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
            cull,
        );
        let line_pipeline = make_pipeline(
            wgpu::PrimitiveTopology::LineList,
            wgpu::PolygonMode::Fill,
            None,
        );
        let point_pipeline = if supports_polygon_point {
            Some(make_pipeline(
                wgpu::PrimitiveTopology::TriangleList,
                wgpu::PolygonMode::Point,
                None,
            ))
        } else {
            None
        };

        
        let depth_view = create_depth_texture(&device, w.max(1), h.max(1), msaa_samples);
        let ms_view    = if msaa_samples > 1 {
            Some(create_ms_texture(&device, surface_format, w.max(1), h.max(1), msaa_samples))
        } else { None };

        
        let obj = state.selected_object().unwrap();
        let cube_data    = generate_cube(&obj.face_colors);
        let sphere_data  = generate_sphere(obj.color, 24, 32);
        let pyramid_data = generate_pyramid(obj.color);
        let torus_data   = generate_torus(obj.color);
        let axes_data    = generate_axes();
        let grid_data    = generate_grid(5);
        let normals_data = generate_normal_lines(&cube_data, 0.3);

        let cube_mesh    = GpuMesh::from_mesh(&device, &cube_data);
        let sphere_mesh  = GpuMesh::from_mesh(&device, &sphere_data);
        let pyramid_mesh = GpuMesh::from_mesh(&device, &pyramid_data);
        let torus_mesh   = GpuMesh::from_line_mesh(&device, &torus_data);
        let axes_mesh    = GpuMesh::from_line_mesh(&device, &axes_data);
        let grid_mesh    = GpuMesh::from_line_mesh(&device, &grid_data);
        let normals_mesh = GpuMesh::from_line_mesh(&device, &normals_data);

        
        
        
        let egui_renderer = egui_renderer;

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,

            solid_pipeline,
            line_pipeline,
            point_pipeline,

            uniform_buf,
            uniform_bind_group,
            bind_group_layout,

            depth_view,
            ms_view,
            msaa_samples,
            max_msaa_samples,

            cube_mesh,
            sphere_mesh,
            pyramid_mesh,
            torus_mesh,
            axes_mesh,
            grid_mesh,
            normals_mesh,

            surface_format,

            egui_renderer,

            screenshot_buf: None,

            adapter_info,
            supports_polygon_line,
            supports_polygon_point,
        })
    }

    

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
        self.surface_config.width  = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_view = create_depth_texture(&self.device, new_size.width, new_size.height, self.msaa_samples);
        if self.msaa_samples > 1 {
            self.ms_view = Some(create_ms_texture(
                &self.device, self.surface_format, new_size.width, new_size.height, self.msaa_samples
            ));
        }
    }

    

    pub fn rebuild_pipelines(&mut self, state: &AppState) {
        let msaa_samples: u32 = if state.msaa_enabled && self.max_msaa_samples >= 4 { 4 } else { 1 };
        if msaa_samples != self.msaa_samples {
            self.msaa_samples = msaa_samples;
            let w = self.surface_config.width;
            let h = self.surface_config.height;
            self.depth_view = create_depth_texture(&self.device, w, h, msaa_samples);
            self.ms_view = if msaa_samples > 1 {
                Some(create_ms_texture(&self.device, self.surface_format, w, h, msaa_samples))
            } else { None };

            
        }

        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label:                Some("pl"),
                bind_group_layouts:   &[&self.bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let depth_stencil = wgpu::DepthStencilState {
            format:              wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare:       wgpu::CompareFunction::Less,
            stencil:             wgpu::StencilState::default(),
            bias:                wgpu::DepthBiasState::default(),
        };

        let sf      = self.surface_format;
        let ms      = self.msaa_samples;
        let cull    = if state.face_culling { Some(wgpu::Face::Back) } else { None };
        let make    = |topology: wgpu::PrimitiveTopology, poly_mode: wgpu::PolygonMode,
                       cull_m: Option<wgpu::Face>| -> wgpu::RenderPipeline {
            self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label:  Some("pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology,
                    front_face:  wgpu::FrontFace::Ccw,
                    cull_mode:   cull_m,
                    polygon_mode:poly_mode,
                    ..Default::default()
                },
                depth_stencil: Some(depth_stencil.clone()),
                multisample:   wgpu::MultisampleState { count: ms, ..Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &shader, entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: sf,
                        blend:  Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };

        self.solid_pipeline  = make(wgpu::PrimitiveTopology::TriangleList,  wgpu::PolygonMode::Fill, cull);
        self.line_pipeline   = make(wgpu::PrimitiveTopology::LineList,      wgpu::PolygonMode::Fill, None);
        self.point_pipeline  = if self.supports_polygon_point {
            Some(make(wgpu::PrimitiveTopology::TriangleList, wgpu::PolygonMode::Point, None))
        } else { None };
    }

    

    pub fn rebuild_meshes(&mut self, state: &AppState) {
        if let Some(obj) = state.selected_object() {
            let cube_data    = generate_cube(&obj.face_colors);
            let sphere_data  = generate_sphere(obj.color, 24, 32);
            let pyramid_data = generate_pyramid(obj.color);
            let torus_data   = generate_torus(obj.color);

            let normals_src = match obj.shape {
                ShapeKind::Cube    => &cube_data,
                ShapeKind::Sphere  => &sphere_data,
                ShapeKind::Pyramid => &pyramid_data,
                ShapeKind::Torus   => &torus_data,
            };
            let normals_data = generate_normal_lines(normals_src, 0.3);

            self.cube_mesh    = GpuMesh::from_mesh(&self.device, &cube_data);
            self.sphere_mesh  = GpuMesh::from_mesh(&self.device, &sphere_data);
            self.pyramid_mesh = GpuMesh::from_mesh(&self.device, &pyramid_data);
            self.torus_mesh   = GpuMesh::from_line_mesh(&self.device, &torus_data);
            self.normals_mesh = GpuMesh::from_line_mesh(&self.device, &normals_data);
        }
    }

    

    pub fn render(
        &mut self,
        state:       &AppState,
        egui_ctx:    &egui::Context,
        full_output: &egui::FullOutput,
        window:      &winit::window::Window,
    ) {
        let surface_tex = match self.surface.get_current_texture() {
            Ok(t)  => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(e) => { log::error!("Surface error: {:?}", e); return; }
        };

        let surface_view = surface_tex.texture.create_view(&Default::default());

        
        let (color_attach, resolve) = if let Some(ms) = &self.ms_view {
            (ms as &wgpu::TextureView, Some(&surface_view as &wgpu::TextureView))
        } else {
            (&surface_view as &wgpu::TextureView, None)
        };

        let bg = state.bg_color;
        let clear_color = wgpu::Color { r: bg[0] as f64, g: bg[1] as f64, b: bg[2] as f64, a: 1.0 };

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("enc") }
        );

        let w = self.surface_config.width  as f32;
        let h = self.surface_config.height as f32;
        let aspect = w / h;

        
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3d_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           color_attach,
                    resolve_target: resolve,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            
            if state.show_grid {
                let grid_unif = self.build_uniforms(
                    state, Mat4::IDENTITY, false, false,
                );
                self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&grid_unif));
                pass.set_pipeline(&self.line_pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_vertex_buffer(0, self.grid_mesh.vbuf.slice(..));
                pass.set_index_buffer(self.grid_mesh.ibuf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.grid_mesh.idx_count, 0, 0..1);
            }

            
            if state.show_axes {
                let axes_unif = self.build_uniforms(
                    state, Mat4::IDENTITY, false, false,
                );
                self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&axes_unif));
                pass.set_pipeline(&self.line_pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_vertex_buffer(0, self.axes_mesh.vbuf.slice(..));
                pass.set_index_buffer(self.axes_mesh.ibuf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.axes_mesh.idx_count, 0, 0..1);
            }

            
            for (oi, obj) in state.objects.iter().enumerate() {
                
                let rx    = Mat4::from_rotation_x(obj.rotation[0]);
                let ry    = Mat4::from_rotation_y(obj.rotation[1]);
                let rz    = Mat4::from_rotation_z(obj.rotation[2]);
                let t     = Mat4::from_translation(Vec3::from(obj.position));
                let s     = Mat4::from_scale(Vec3::splat(obj.scale));
                let model = t * rz * ry * rx * s;

                
                if state.rotation_trail && oi == state.selected_obj {
                    for (ti, tr) in state.trail_rotations.iter().enumerate() {
                        let fade = (ti as f32 + 1.0) / (state.trail_rotations.len() as f32 + 1.0);
                        let trx  = Mat4::from_rotation_x(tr[0]);
                        let try_ = Mat4::from_rotation_y(tr[1]);
                        let trz  = Mat4::from_rotation_z(tr[2]);
                        let tmodel = t * trz * try_ * trx * s;
                        let mut trail_unif = self.build_uniforms(state, tmodel, false, false);
                        trail_unif.alpha   = fade * 0.35;
                        self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&trail_unif));
                        
                        let trail_pipe = if obj.shape == ShapeKind::Torus {
                            &self.line_pipeline
                        } else {
                            &self.solid_pipeline
                        };
                        pass.set_pipeline(trail_pipe);
                        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                        self.draw_object_mesh(&mut pass, obj.shape, state, false);
                    }
                }

                let is_line_shape = obj.shape == ShapeKind::Torus;
                let use_lighting  = state.lighting_enabled
                    && state.render_mode == crate::state::RenderMode::Solid
                    && !is_line_shape;
                let show_normals  = state.show_normals && oi == state.selected_obj && !is_line_shape;
                let unif = self.build_uniforms_full(state, model, aspect, use_lighting, show_normals, obj.alpha);
                self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&unif));

                if is_line_shape {
                    
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    self.draw_object_mesh(&mut pass, obj.shape, state, false);
                } else {
                    match state.render_mode {
                        RenderMode::Solid => {
                            pass.set_pipeline(&self.solid_pipeline);
                            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                            self.draw_object_mesh(&mut pass, obj.shape, state, false);
                        }
                        RenderMode::Wireframe => {
                            let mut wf_unif = unif;
                            wf_unif.use_light = 0;
                            self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&wf_unif));
                            pass.set_pipeline(&self.line_pipeline);
                            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                            self.draw_object_mesh(&mut pass, obj.shape, state, true);
                        }
                        RenderMode::Points => {
                            let pp = self.point_pipeline.as_ref().unwrap_or(&self.solid_pipeline);
                            pass.set_pipeline(pp);
                            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                            self.draw_object_mesh(&mut pass, obj.shape, state, false);
                        }
                    }
                }

                
                if show_normals {
                    let mut n_unif = unif;
                    n_unif.use_light    = 0;
                    n_unif.show_normals = 0;
                    self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&n_unif));
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    pass.set_vertex_buffer(0, self.normals_mesh.vbuf.slice(..));
                    pass.set_index_buffer(self.normals_mesh.ibuf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..self.normals_mesh.idx_count, 0, 0..1);
                }
            }
        } 

        
        let pixels_per_point = window.scale_factor() as f32;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels:  [self.surface_config.width, self.surface_config.height],
            pixels_per_point,
        };

        let primitives = egui_ctx.tessellate(
            full_output.shapes.clone(),
            full_output.pixels_per_point,
        );

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }

        
        
        let _extra_cmds = self.egui_renderer.update_buffers(
            &self.device, &self.queue, &mut encoder,
            &primitives, &screen_descriptor,
        );

        {
            let mut egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    
                    
                    view:           &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.egui_renderer.render(&mut egui_pass, &primitives, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        
        let screenshot_this_frame = state.screenshot_requested;
        if screenshot_this_frame {
            let bpp = 4u32; 
            let align    = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let unpadded = self.surface_config.width * bpp;
            let padded   = (unpadded + align - 1) / align * align;
            let buf_size = (padded * self.surface_config.height) as u64;
            let ss_buf   = self.device.create_buffer(&wgpu::BufferDescriptor {
                label:              Some("ss_buf"),
                size:               buf_size,
                usage:              wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                surface_tex.texture.as_image_copy(),
                wgpu::ImageCopyBuffer {
                    buffer: &ss_buf,
                    layout: wgpu::ImageDataLayout {
                        offset:         0,
                        bytes_per_row:  Some(padded),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width:                 self.surface_config.width,
                    height:                self.surface_config.height,
                    depth_or_array_layers: 1,
                },
            );
            self.screenshot_buf = Some((ss_buf, padded, self.surface_config.height));
        }

        
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_tex.present();
    }

    

    fn draw_object_mesh<'a>(
        &'a self,
        pass:  &mut wgpu::RenderPass<'a>,
        shape: ShapeKind,
        _state: &AppState,
        _wireframe: bool,
    ) {
        let mesh = match shape {
            ShapeKind::Cube    => &self.cube_mesh,
            ShapeKind::Sphere  => &self.sphere_mesh,
            ShapeKind::Pyramid => &self.pyramid_mesh,
            ShapeKind::Torus   => &self.torus_mesh,
        };
        pass.set_vertex_buffer(0, mesh.vbuf.slice(..));
        pass.set_index_buffer(mesh.ibuf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.idx_count, 0, 0..1);
    }


    fn build_uniforms(&self, state: &AppState, model: Mat4, use_light: bool, show_normals: bool) -> Uniforms {
        let w = self.surface_config.width  as f32;
        let h = self.surface_config.height as f32;
        let aspect = w / h;
        self.build_uniforms_full(state, model, aspect, use_light, show_normals, 1.0)
    }

    fn build_uniforms_full(
        &self,
        state:       &AppState,
        model:       Mat4,
        aspect:      f32,
        use_light:   bool,
        show_normals:bool,
        alpha:       f32,
    ) -> Uniforms {
        let proj = match state.projection_mode {
            ProjectionMode::Perspective => {
                let fov = state.fov_deg.to_radians();
                Mat4::perspective_rh(fov, aspect, 0.1, 1000.0)
            }
            ProjectionMode::Orthographic => {
                let half = state.cam_distance * 0.7;
                Mat4::orthographic_rh(-half * aspect, half * aspect, -half, half, 0.1, 1000.0)
            }
        };

        let cam_pos = Vec3::new(0.0, 0.0, state.cam_distance);
        let view    = Mat4::look_at_rh(cam_pos, Vec3::ZERO, Vec3::Y);
        let mvp     = proj * view * model;
        let norm_m  = model.inverse().transpose();

        Uniforms {
            mvp:          mvp.to_cols_array_2d(),
            model:        model.to_cols_array_2d(),
            normal_mat:   norm_m.to_cols_array_2d(),
            light_pos:    state.light_pos,
            ambient:      state.ambient,
            light_color:  state.light_color,
            diffuse_s:    state.diffuse,
            cam_pos:      cam_pos.into(),
            specular_s:   state.specular,
            shininess:    state.shininess,
            alpha,
            use_light:    use_light as u32,
            show_normals: show_normals as u32,
        }
    }

    

    pub fn try_finalize_screenshot(&mut self, state: &mut AppState) {
        if let Some((buf, padded_row, height)) = self.screenshot_buf.take() {
            let width = self.surface_config.width;
            let slice = buf.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
            self.device.poll(wgpu::Maintain::Wait);
            if rx.recv().unwrap().is_ok() {
                let data   = slice.get_mapped_range();
                let bpp    = 4u32;
                let unpadded = width * bpp;
                let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
                for row in 0..height {
                    let start = (row * padded_row) as usize;
                    let end   = start + unpadded as usize;
                    pixels.extend_from_slice(&data[start..end]);
                }
                drop(data);
                buf.unmap();

                let ts     = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let fname  = format!("screenshot_{}.png", ts);
                if let Some(img) = image::RgbaImage::from_raw(width, height, pixels) {
                    let _ = img.save(&fname);
                    state.export_rotation_text = Some(format!("Screenshot saved: {}", fname));
                }
            }
        }
    }

    

    pub fn set_vsync(&mut self, vsync: bool) {
        self.surface_config.present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.surface_config);
    }
}



fn create_depth_texture(device: &wgpu::Device, w: u32, h: u32, samples: u32) -> wgpu::TextureView {
    
    let usage = if samples > 1 {
        wgpu::TextureUsages::RENDER_ATTACHMENT
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
    };
    device.create_texture(&wgpu::TextureDescriptor {
        label:           Some("depth_tex"),
        size:            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count:    samples,
        dimension:       wgpu::TextureDimension::D2,
        format:          wgpu::TextureFormat::Depth32Float,
        usage,
        view_formats:    &[],
    }).create_view(&Default::default())
}

fn create_ms_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    w: u32, h: u32,
    samples: u32,
) -> wgpu::TextureView {
    device.create_texture(&wgpu::TextureDescriptor {
        label:           Some("ms_tex"),
        size:            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count:    samples,
        dimension:       wgpu::TextureDimension::D2,
        format,
        usage:           wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats:    &[],
    }).create_view(&Default::default())
}