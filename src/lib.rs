// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
pub mod state;
pub mod renderer;
pub mod mesh;
pub mod ui;
use std::sync::Arc;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(not(target_arch = "wasm32"))]
use winit::window::Fullscreen;

use state::{AppState, BackendChoice};
use renderer::{BareGpu, Renderer};
pub fn show_error(title: &str, msg: &str) {
    log::error!("{title}: {msg}");
    eprintln!("ERROR — {title}: {msg}");

    #[cfg(not(target_arch = "wasm32"))]
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(title)
        .set_description(msg)
        .show();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let _ = win.alert_with_message(&format!("{title}: {msg}"));
        }
    }
}

fn choice_matches_bare(choice: BackendChoice, bare: &BareGpu) -> bool {
    match choice {
        BackendChoice::Auto   => true,
        BackendChoice::Vulkan => bare.adapter_info.backend == wgpu::Backend::Vulkan,
        BackendChoice::Metal  => bare.adapter_info.backend == wgpu::Backend::Metal,
        BackendChoice::Dx12   => bare.adapter_info.backend == wgpu::Backend::Dx12,
        BackendChoice::OpenGl => bare.adapter_info.backend == wgpu::Backend::Gl,
        BackendChoice::WebGpu => bare.adapter_info.backend == wgpu::Backend::BrowserWebGpu,
    }
}

fn backend_from_env() -> Option<BackendChoice> {
    #[cfg(not(target_arch = "wasm32"))]
    match std::env::var("THREEDTEST_BACKEND").as_deref() {
        Ok("auto")   => return Some(BackendChoice::Auto),
        Ok("vulkan") => return Some(BackendChoice::Vulkan),
        Ok("dx12")   => return Some(BackendChoice::Dx12),
        Ok("gl")     => return Some(BackendChoice::OpenGl),
        Ok("metal")  => return Some(BackendChoice::Metal),
        Ok("webgpu") => return Some(BackendChoice::WebGpu),
        _            => {}
    }
    None
}

fn restart_with_backend(backend: BackendChoice) -> ! {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let key = match backend {
            BackendChoice::Auto   => "auto",
            BackendChoice::Vulkan => "vulkan",
            BackendChoice::Dx12   => "dx12",
            BackendChoice::OpenGl => "gl",
            BackendChoice::Metal  => "metal",
            BackendChoice::WebGpu => "webgpu",
        };
        let exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("threedtest"));
        let _ = std::process::Command::new(exe)
            .env("THREEDTEST_BACKEND", key)
            .spawn();
        std::process::exit(0);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = backend; 
        
        if let Some(win) = web_sys::window() {
            let _ = win.location().reload();
        }
        panic!("Reloading page for backend switch");
    }
}

enum Phase {
    Startup(BareGpu),
    Running(Renderer),
}

pub async fn run() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    
    #[cfg(target_arch = "wasm32")]
    let initial_size = {
        let win = web_sys::window().unwrap();
        let w = win.inner_width().unwrap().as_f64().unwrap() as u32;
        let h = win.inner_height().unwrap().as_f64().unwrap() as u32;
        winit::dpi::PhysicalSize::new(w.max(1), h.max(1))
    };
    #[cfg(not(target_arch = "wasm32"))]
    let initial_size = winit::dpi::PhysicalSize::new(1280u32, 720u32);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("3DTest")
            .with_inner_size(initial_size)
            .with_min_inner_size(winit::dpi::PhysicalSize::new(400u32, 300u32))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let canvas = window.canvas().expect("winit canvas");
        
        canvas.style().set_property("width",  "100%").ok();
        canvas.style().set_property("height", "100%").ok();
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
            .map(|body| body.append_child(&canvas).ok());
    }

    let mut app_state = AppState::default();
    let forced_backend = backend_from_env();
    let mut phase: Option<Phase>;

    if let Some(backend) = forced_backend {
        app_state.backend_choice = backend;
        app_state.startup_phase  = false;
        let renderer = match Renderer::new(window.clone(), &app_state).await {
            Ok(r) => r,
            Err(e) => { show_error("Failed to initialize GPU", &e); return; }
        };
        app_state.active_backend = format!("{:?}", renderer.adapter_info.backend);
        app_state.adapter_info   = format!(
            "{} ({})", renderer.adapter_info.name, renderer.adapter_info.driver
        );
        phase = Some(Phase::Running(renderer));
    } else {
        let bare = match BareGpu::new(window.clone(), BackendChoice::Auto).await {
            Ok(b) => b,
            Err(e) => { show_error("Failed to initialize GPU", &e); return; }
        };
        app_state.active_backend = format!("{:?}", bare.adapter_info.backend);
        app_state.adapter_info   = format!(
            "{} ({})", bare.adapter_info.name, bare.adapter_info.driver
        );

        
        #[cfg(target_arch = "wasm32")]
        {
            app_state.startup_phase = false;
            app_state.backend_choice = BackendChoice::Auto;
            app_state.pending_backend = Some(BackendChoice::Auto);
        }

        phase = Some(Phase::Startup(bare));
    }

    let egui_ctx = egui::Context::default();
    let initial_scale = window.scale_factor() as f32;
    let mut egui_winit_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &event_loop,
        Some(initial_scale),
        None,
    );

    
    #[cfg(not(target_arch = "wasm32"))]
    let mut last_time  = std::time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let mut fps_timer  = std::time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let mut trail_last = std::time::Instant::now();

    #[cfg(target_arch = "wasm32")]
    let perf = web_sys::window()
        .and_then(|w| w.performance())
        .expect("window.performance");
    #[cfg(target_arch = "wasm32")]
    let mut last_time_ms  = perf.now();
    #[cfg(target_arch = "wasm32")]
    let mut fps_timer_ms  = perf.now();
    #[cfg(target_arch = "wasm32")]
    let mut trail_last_ms = perf.now();

    let mut fps_frames = 0u32;

    
    let event_handler = move |event: Event<()>,
                               elwt: &winit::event_loop::EventLoopWindowTarget<()>| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: ref win_event, .. } => {
                let resp = egui_winit_state.on_window_event(&window, win_event);

                match win_event {
                    WindowEvent::CloseRequested => { elwt.exit(); }

                    WindowEvent::Resized(sz) => {
                        app_state.resolution = [sz.width, sz.height];
                        match phase.as_mut().unwrap() {
                            Phase::Startup(bare) => bare.resize(*sz),
                            Phase::Running(r)    => r.resize(*sz),
                        }
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        let sz = window.inner_size();
                        app_state.resolution = [sz.width, sz.height];
                        match phase.as_mut().unwrap() {
                            Phase::Startup(bare) => bare.resize(sz),
                            Phase::Running(r)    => r.resize(sz),
                        }
                    }

                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left, ..
                    } => {
                        if !resp.consumed { app_state.mouse_drag = true; }
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left, ..
                    } => {
                        app_state.mouse_drag = false;
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let x = position.x as f32;
                        let y = position.y as f32;
                        if app_state.mouse_drag && !resp.consumed {
                            let dx = x - app_state.mouse_last[0];
                            let dy = y - app_state.mouse_last[1];
                            if let Some(obj) = app_state.selected_object_mut() {
                                obj.rotation[1] += dx * 0.008;
                                obj.rotation[0] += dy * 0.008;
                            }
                        }
                        app_state.mouse_last = [x, y];
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        if !resp.consumed {
                            let s = match delta {
                                MouseScrollDelta::LineDelta(_, y)  => *y,
                                MouseScrollDelta::PixelDelta(p)    => p.y as f32 / 50.0,
                            };
                            app_state.cam_distance =
                                (app_state.cam_distance - s * 0.4).clamp(0.5, 50.0);
                        }
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        if !resp.consumed && event.state == ElementState::Pressed {
                            if let PhysicalKey::Code(key) = event.physical_key {
                                handle_key(key, &mut app_state);
                            }
                        }
                    }

                    WindowEvent::RedrawRequested => {
                        
                        #[cfg(not(target_arch = "wasm32"))]
                        let dt = {
                            let now = std::time::Instant::now();
                            let dt = now.duration_since(last_time).as_secs_f32();
                            last_time = now;
                            dt
                        };
                        #[cfg(target_arch = "wasm32")]
                        let dt = {
                            let now = perf.now();
                            let dt  = ((now - last_time_ms) / 1000.0) as f32;
                            last_time_ms = now;
                            dt
                        };

                        app_state.frame_time_ms = dt * 1000.0;
                        fps_frames += 1;

                        
                        #[cfg(not(target_arch = "wasm32"))]
                        let fps_elapsed = fps_timer.elapsed().as_secs_f32();
                        #[cfg(target_arch = "wasm32")]
                        let fps_elapsed = ((perf.now() - fps_timer_ms) / 1000.0) as f32;

                        if fps_elapsed >= 1.0 {
                            #[cfg(not(target_arch = "wasm32"))]
                            { fps_timer = std::time::Instant::now(); }
                            #[cfg(target_arch = "wasm32")]
                            { fps_timer_ms = perf.now(); }

                            let f = fps_frames as f32;
                            app_state.fps = fps_frames;
                            if f < app_state.fps_min { app_state.fps_min = f; }
                            if f > app_state.fps_max { app_state.fps_max = f; }
                            let n = app_state.fps_history.len().min(60) as f32;
                            app_state.fps_avg = (app_state.fps_avg * n + f) / (n + 1.0);
                            app_state.fps_history.push(f);
                            if app_state.fps_history.len() > 120 {
                                app_state.fps_history.remove(0);
                            }
                            fps_frames = 0;
                        }

                        
                        if let Some(new_backend) = app_state.pending_backend.take() {
                            if let Some(Phase::Startup(_)) = &phase {
                                app_state.backend_choice = new_backend;

                                let bare = match phase.take().unwrap() {
                                    Phase::Startup(b) => b,
                                    _ => unreachable!(),
                                };

                                if choice_matches_bare(new_backend, &bare) {
                                    log::info!("Upgrading BareGpu → Renderer (surface reused)");
                                    match Renderer::from_bare_gpu(bare, &app_state) {
                                        Ok(r) => {
                                            app_state.active_backend =
                                                format!("{:?}", r.adapter_info.backend);
                                            app_state.adapter_info = format!(
                                                "{} ({})",
                                                r.adapter_info.name,
                                                r.adapter_info.driver
                                            );
                                            phase = Some(Phase::Running(r));
                                        }
                                        Err(e) => {
                                            show_error("Failed to start renderer", &e);
                                            elwt.exit();
                                        }
                                    }
                                } else {
                                    log::info!("Different backend — restarting");
                                    drop(bare);
                                    restart_with_backend(new_backend);
                                }
                            }
                        }

                        
                        if let Some(Phase::Running(r)) = phase.as_mut() {
                            if app_state.pipeline_dirty {
                                app_state.pipeline_dirty = false;
                                r.rebuild_pipelines(&app_state);
                                r.set_vsync(app_state.vsync_enabled);
                            }
                            if app_state.mesh_dirty {
                                app_state.mesh_dirty = false;
                                r.rebuild_meshes(&app_state);
                            }
                        }

                        
                        if matches!(phase, Some(Phase::Running(_))) {
                            if !app_state.paused && app_state.auto_rotate {
                                let spd = app_state.rot_speed;
                                if let Some(obj) = app_state.selected_object_mut() {
                                    obj.rotation[0] += spd[0] * dt;
                                    obj.rotation[1] += spd[1] * dt;
                                    obj.rotation[2] += spd[2] * dt;
                                }
                            }

                            if app_state.reset_rotation_requested {
                                app_state.reset_rotation_requested = false;
                                if let Some(obj) = app_state.selected_object_mut() {
                                    obj.rotation = [0.0; 3];
                                }
                                app_state.trail_rotations.clear();
                            }

                            
                            #[cfg(not(target_arch = "wasm32"))]
                            let trail_elapsed = trail_last.elapsed().as_secs_f32();
                            #[cfg(target_arch = "wasm32")]
                            let trail_elapsed = ((perf.now() - trail_last_ms) / 1000.0) as f32;

                            if app_state.rotation_trail && trail_elapsed > 0.12 {
                                #[cfg(not(target_arch = "wasm32"))]
                                { trail_last = std::time::Instant::now(); }
                                #[cfg(target_arch = "wasm32")]
                                { trail_last_ms = perf.now(); }

                                if let Some(obj) = app_state.selected_object() {
                                    app_state.trail_rotations.push(obj.rotation);
                                }
                                while app_state.trail_rotations.len() > app_state.trail_len {
                                    app_state.trail_rotations.remove(0);
                                }
                            }

                            
                            if app_state.fullscreen_requested {
                                app_state.fullscreen_requested = false;
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if window.fullscreen().is_some() {
                                        window.set_fullscreen(None);
                                        app_state.fullscreen_active = false;
                                    } else {
                                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                        app_state.fullscreen_active = true;
                                    }
                                }
                            }
                        }

                        
                        let raw_input   = egui_winit_state.take_egui_input(&window);
                        let full_output = egui_ctx.run(raw_input, |ctx| {
                            ui::draw(ctx, &mut app_state)
                        });
                        egui_winit_state.handle_platform_output(
                            &window,
                            full_output.platform_output.clone(),
                        );

                        
                        match phase.as_mut().unwrap() {
                            Phase::Startup(bare) => {
                                bare.render_startup(&egui_ctx, &full_output, &window);
                            }
                            Phase::Running(r) => {
                                r.render(&app_state, &egui_ctx, &full_output, &window);

                                #[cfg(not(target_arch = "wasm32"))]
                                if app_state.screenshot_requested {
                                    app_state.screenshot_requested = false;
                                    r.try_finalize_screenshot(&mut app_state);
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { .. }, ..
            } => {}
            Event::AboutToWait => { window.request_redraw(); }
            _ => {}
        }
    };

    
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(event_handler).expect("Event loop error");

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn(event_handler);
        
    }
}

fn handle_key(key: KeyCode, state: &mut AppState) {
    match key {
        KeyCode::KeyS     => state.show_settings           = !state.show_settings,
        KeyCode::KeyD     => state.show_debug               = !state.show_debug,
        KeyCode::KeyH     => state.show_keyboard_help       = !state.show_keyboard_help,
        KeyCode::KeyP | KeyCode::Space => state.paused      = !state.paused,
        KeyCode::KeyR     => state.reset_rotation_requested = true,
        #[cfg(not(target_arch = "wasm32"))]
        KeyCode::F11      => state.fullscreen_requested     = true,
        #[cfg(not(target_arch = "wasm32"))]
        KeyCode::F5       => state.screenshot_requested     = true,
        KeyCode::Escape   => {
            state.show_keyboard_help = false;
            state.show_info_dialog   = false;
        }
        _ => {}
    }
}



#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();
    log::info!("3DTest — wasm entry");
    wasm_bindgen_futures::spawn_local(run());
}
