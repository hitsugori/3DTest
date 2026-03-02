// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
use webbrowser;
use egui::{Context, RichText, Color32, Slider, Ui, Vec2, Stroke, FontId};
use crate::state::{AppState, BackendChoice, RenderMode, ProjectionMode, ShapeKind, Object3D};
pub fn draw(ctx: &Context, state: &mut AppState) {
    apply_theme(ctx, state.dark_theme);

    if state.startup_phase {
        draw_startup_dialog(ctx, state);
        return;
    }

    
    draw_fps_overlay(ctx, state);

    
    if state.show_debug {
        draw_debug_overlay(ctx, state);
    }

    
    if state.show_settings {
        draw_settings_panel(ctx, state);
    }

    
    if state.show_perf_graph {
        draw_perf_graph(ctx, state);
    }

    
    if state.show_keyboard_help {
        draw_keyboard_shortcuts(ctx, state);
    }

    
    if state.show_info_dialog {
        draw_info_dialog(ctx, state);
    }

    
    if let Some(msg) = state.export_rotation_text.clone() {
        draw_toast(ctx, &msg, state);
    }
}

fn draw_startup_dialog(ctx: &Context, state: &mut AppState) {
    
    egui::Area::new("backdrop".into())
        .fixed_pos([0.0, 0.0])
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(220));
        });

    egui::Window::new("3DTest - Renderer Selector")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .min_width(440.0)
        .show(ctx, |ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new("Select Rendering Backend").size(20.0).strong());
            ui.add_space(4.0);
            ui.label("Choose the GPU API to use. 'Auto' selects the best available.");
            ui.add_space(8.0);

            
            
            #[cfg(not(target_arch = "wasm32"))]
            let choices: &[BackendChoice] = &[
                BackendChoice::Auto,
                BackendChoice::Vulkan,
                #[cfg(target_os = "windows")]
                BackendChoice::Dx12,
                BackendChoice::OpenGl,
            ];
            #[cfg(target_arch = "wasm32")]
            let choices: &[BackendChoice] = &[
                BackendChoice::Auto,
                BackendChoice::WebGpu,
            ];
            egui::Grid::new("bg_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                for &choice in choices {
                    let selected = state.backend_choice == choice;
                    if ui.selectable_label(selected, choice.label()).clicked() {
                        state.backend_choice = choice;
                    }
                    ui.end_row();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("  🚀  Launch  ").size(16.0)).clicked() {
                    state.pending_backend = Some(state.backend_choice);
                    state.startup_phase = false;
                }
                ui.add_space(8.0);
                ui.label(RichText::new("Press Enter to launch").color(Color32::GRAY));
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(RichText::new("3DTest  •  mikedev_  •  GPL-3.0-or-later  •  Version 1.3.1").small().color(Color32::GRAY));

            
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                state.pending_backend = Some(state.backend_choice);
                state.startup_phase = false;
            }
        });
}



fn draw_settings_panel(ctx: &Context, state: &mut AppState) {
    egui::SidePanel::right("settings_panel")
        .resizable(true)
        .min_width(260.0)
        .max_width(380.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⚙ Settings");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() { state.show_settings = false; }
                    if ui.button("ℹ").clicked() { state.show_info_dialog = true; }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                section(ui, "🖥  Display", |ui| {
                    ui.label(format!("Resolution: {}×{}", state.resolution[0], state.resolution[1]));
                    ui.label(format!("Backend: {}", state.active_backend));
                    ui.add_space(4.0);
                    if ui.checkbox(&mut state.vsync_enabled, "VSync").changed() {
                        state.pipeline_dirty = true;
                    }
                    if ui.checkbox(&mut state.face_culling, "Face Culling").changed() {
                        state.pipeline_dirty = true;
                    }
                    if ui.checkbox(&mut state.msaa_enabled, "MSAA 4×").changed() {
                        state.pipeline_dirty = true;
                    }
                    if ui.button(if state.fullscreen_active { "⬜ Windowed (F11)" } else { "⛶ Fullscreen (F11)" }).clicked() {
                        state.fullscreen_requested = true;
                    }
                });

                section(ui, "🎨  Rendering", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        for (mode, label) in [
                            (RenderMode::Solid,     "Solid"),
                            (RenderMode::Wireframe, "Wire"),
                            (RenderMode::Points,    "Points"),
                        ] {
                            if ui.selectable_label(state.render_mode == mode, label).clicked() {
                                state.render_mode = mode;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Projection:");
                        for (mode, label) in [
                            (ProjectionMode::Perspective,  "Persp"),
                            (ProjectionMode::Orthographic, "Ortho"),
                        ] {
                            if ui.selectable_label(state.projection_mode == mode, label).clicked() {
                                state.projection_mode = mode;
                            }
                        }
                    });
                    ui.add(Slider::new(&mut state.fov_deg, 15.0..=150.0).text("FOV°"));
                    ui.add(Slider::new(&mut state.cam_distance, 1.0..=30.0).text("Distance"));

                    
                    ui.horizontal(|ui| {
                        ui.label("Background:");
                        let mut c32 = arr_to_color32(state.bg_color);
                        if ui.color_edit_button_srgba(&mut c32).changed() {
                            state.bg_color = color32_to_arr(c32);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Wireframe:");
                        let mut c32 = arr_to_color32(state.wireframe_color);
                        if ui.color_edit_button_srgba(&mut c32).changed() {
                            state.wireframe_color = color32_to_arr(c32);
                        }
                    });
                });

                section(ui, "💡  Lighting", |ui| {
                    ui.checkbox(&mut state.lighting_enabled, "Enable Phong Lighting");
                    ui.add_space(2.0);
                    ui.add(Slider::new(&mut state.ambient,  0.0..=1.0).text("Ambient"));
                    ui.add(Slider::new(&mut state.diffuse,  0.0..=1.0).text("Diffuse"));
                    ui.add(Slider::new(&mut state.specular, 0.0..=1.0).text("Specular"));
                    ui.add(Slider::new(&mut state.shininess, 1.0..=256.0).text("Shininess"));
                    ui.add_space(4.0);
                    ui.label("Light Position:");
                    ui.add(Slider::new(&mut state.light_pos[0], -10.0..=10.0).text("X"));
                    ui.add(Slider::new(&mut state.light_pos[1], -10.0..=10.0).text("Y"));
                    ui.add(Slider::new(&mut state.light_pos[2], -10.0..=10.0).text("Z"));
                    ui.horizontal(|ui| {
                        ui.label("Light Color:");
                        let mut c = light_arr_to_color32(state.light_color);
                        if ui.color_edit_button_srgba(&mut c).changed() {
                            state.light_color = color32_to_arr3(c);
                        }
                    });
                });

                section(ui, "🔄  Animation", |ui| {
                    ui.checkbox(&mut state.auto_rotate, "Auto-Rotate");
                    ui.horizontal(|ui| {
                        if ui.button(if state.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                            state.paused = !state.paused;
                        }
                        if ui.button("↺ Reset").clicked() {
                            state.reset_rotation_requested = true;
                        }
                    });
                    ui.add(Slider::new(&mut state.rot_speed[0], -5.0..=5.0).text("Speed X"));
                    ui.add(Slider::new(&mut state.rot_speed[1], -5.0..=5.0).text("Speed Y"));
                    ui.add(Slider::new(&mut state.rot_speed[2], -5.0..=5.0).text("Speed Z"));
                    ui.checkbox(&mut state.rotation_trail, "Rotation Trail");
                    if state.rotation_trail {
                        ui.add(Slider::new(&mut state.trail_len, 2..=20).text("Trail Length"));
                    }
                });

                section(ui, "👁  Overlays", |ui| {
                    ui.checkbox(&mut state.show_axes,    "Show Axes (X/Y/Z)");
                    ui.checkbox(&mut state.show_grid,    "Show Grid");
                    ui.checkbox(&mut state.show_normals, "Show Normals");
                    ui.checkbox(&mut state.show_debug,   "Debug Overlay (D)");
                    ui.checkbox(&mut state.show_perf_graph, "Performance Graph");
                    ui.checkbox(&mut state.dark_theme,   "Dark Theme");
                });

                section(ui, "📦  Objects", |ui| {
                    let len = state.objects.len();
                    let mut to_remove: Option<usize> = None;
                    let mut to_select: Option<usize> = None;

                    for i in 0..len {
                        let selected = state.selected_obj == i;
                        ui.horizontal(|ui| {
                            if ui.selectable_label(selected, format!("▸ {}", state.objects[i].name)).clicked() {
                                to_select = Some(i);
                            }
                            if len > 1 {
                                if ui.small_button("🗑").clicked() {
                                    to_remove = Some(i);
                                }
                            }
                        });
                    }
                    if let Some(i) = to_remove {
                        state.objects.remove(i);
                        state.selected_obj = state.selected_obj.min(state.objects.len().saturating_sub(1));
                    }
                    if let Some(i) = to_select {
                        state.objects.iter_mut().for_each(|o| o.selected = false);
                        state.selected_obj = i;
                        if let Some(o) = state.objects.get_mut(i) { o.selected = true; }
                    }

                    if ui.button("➕ Add Object").clicked() {
                        let n = state.objects.len();
                        let mut new_obj = Object3D::default();
                        new_obj.name     = format!("Object {}", n);
                        new_obj.position = [(n as f32) * 2.5 - 2.5, 0.0, 0.0];
                        state.objects.push(new_obj);
                    }
                });

                
                if let Some(idx) = Some(state.selected_obj) {
                    if idx < state.objects.len() {
                        let obj_name = state.objects[idx].name.clone();
                        section(ui, &format!("✏  {}", obj_name), |ui| {
                            let obj = &mut state.objects[idx];

                            
                            ui.horizontal(|ui| {
                                ui.label("Shape:");
                                for shape in [ShapeKind::Cube, ShapeKind::Sphere, ShapeKind::Pyramid, ShapeKind::Torus] {
                                    if ui.selectable_label(obj.shape == shape, shape.label()).clicked() {
                                        obj.shape = shape;
                                        state.mesh_dirty = true;
                                    }
                                }
                            });

                            ui.add(Slider::new(&mut obj.scale, 0.1..=5.0).text("Scale"));
                            ui.add(Slider::new(&mut obj.alpha,  0.0..=1.0).text("Alpha"));

                            ui.label("Position:");
                            ui.add(Slider::new(&mut obj.position[0], -10.0..=10.0).text("X"));
                            ui.add(Slider::new(&mut obj.position[1], -10.0..=10.0).text("Y"));
                            ui.add(Slider::new(&mut obj.position[2], -10.0..=10.0).text("Z"));

                            ui.label("Rotation (rad):");
                            ui.add(Slider::new(&mut obj.rotation[0], -std::f32::consts::TAU..=std::f32::consts::TAU).text("X"));
                            ui.add(Slider::new(&mut obj.rotation[1], -std::f32::consts::TAU..=std::f32::consts::TAU).text("Y"));
                            ui.add(Slider::new(&mut obj.rotation[2], -std::f32::consts::TAU..=std::f32::consts::TAU).text("Z"));

                            
                            if obj.shape == ShapeKind::Cube {
                                ui.collapsing("Face Colors", |ui| {
                                    let names = ["Front", "Back", "Top", "Bottom", "Right", "Left"];
                                    let mut dirty = false;
                                    for (fi, name) in names.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(*name);
                                            let mut c = arr_to_color32(obj.face_colors[fi]);
                                            if ui.color_edit_button_srgba(&mut c).changed() {
                                                obj.face_colors[fi] = color32_to_arr(c);
                                                dirty = true;
                                            }
                                        });
                                    }
                                    if dirty { state.mesh_dirty = true; }
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    let mut c = arr_to_color32(obj.color);
                                    if ui.color_edit_button_srgba(&mut c).changed() {
                                        obj.color = color32_to_arr(c);
                                        state.mesh_dirty = true;
                                    }
                                });
                            }
                        });
                    }
                }

                section(ui, "💾  Actions", |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📸 Screenshot").clicked() {
                            state.screenshot_requested = true;
                        }
                        if ui.button("📋 Export Rot").clicked() {
                            if let Some(obj) = state.selected_object() {
                                state.export_rotation_text = Some(format!(
                                    "Rot: X={:.3} Y={:.3} Z={:.3}",
                                    obj.rotation[0], obj.rotation[1], obj.rotation[2]
                                ));
                            }
                        }
                    });
                    if ui.button("🔄 Reset All Settings").clicked() {
                        state.reset_all();
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("Press 'H' for shortcuts  •  'D' for debug").small().color(Color32::GRAY));
            });
        });
}



fn draw_fps_overlay(ctx: &Context, state: &AppState) {
    egui::Area::new("fps_overlay".into())
        .fixed_pos([10.0, 10.0])
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Color32::from_black_alpha(140))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    let fps_color = if state.fps >= 60 {
                        Color32::from_rgb(100, 220, 100)
                    } else if state.fps >= 30 {
                        Color32::from_rgb(220, 200, 80)
                    } else {
                        Color32::from_rgb(220, 80, 80)
                    };
                    ui.label(RichText::new(format!("FPS: {}", state.fps))
                        .color(fps_color)
                        .font(FontId::monospace(14.0)));
                    ui.label(RichText::new(format!("{:.2} ms", state.frame_time_ms))
                        .color(Color32::LIGHT_GRAY)
                        .font(FontId::monospace(12.0)));

                    if !state.show_settings {
                        ui.label(RichText::new("S: settings").small().color(Color32::GRAY));
                    }
                });
        });
}



fn draw_debug_overlay(ctx: &Context, state: &AppState) {
    egui::Window::new("🐛 Debug")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .resizable(false)
        .collapsible(true)
        .show(ctx, |ui| {
            let obj = state.selected_object();
            if let Some(o) = obj {
                ui.monospace(format!("Shape:    {:?}", o.shape));
                ui.monospace(format!("Rot:      X={:.2} Y={:.2} Z={:.2}", o.rotation[0], o.rotation[1], o.rotation[2]));
                ui.monospace(format!("Pos:      X={:.2} Y={:.2} Z={:.2}", o.position[0], o.position[1], o.position[2]));
                ui.monospace(format!("Scale:    {:.2}", o.scale));
            }
            ui.separator();
            ui.monospace(format!("FPS:      {}", state.fps));
            ui.monospace(format!("FPS min:  {:.1}", if state.fps_min == f32::MAX { 0.0 } else { state.fps_min }));
            ui.monospace(format!("FPS max:  {:.1}", state.fps_max));
            ui.monospace(format!("FPS avg:  {:.1}", state.fps_avg));
            ui.monospace(format!("Frame:    {:.2}ms", state.frame_time_ms));
            ui.separator();
            ui.monospace(format!("Backend:  {}", state.active_backend));
            ui.monospace(format!("GPU:      {}", state.adapter_info));
            ui.monospace(format!("Res:      {}×{}", state.resolution[0], state.resolution[1]));
            ui.monospace(format!("MSAA:     {}", if state.msaa_enabled { "4×" } else { "off" }));
            ui.monospace(format!("VSync:    {}", state.vsync_enabled));
            ui.monospace(format!("Depth:    {}", state.depth_display));
            ui.monospace(format!("Objects:  {}", state.objects.len()));
        });
}



fn draw_perf_graph(ctx: &Context, state: &AppState) {
    egui::Window::new("📈 Performance")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 80.0])
        .resizable(false)
        .collapsible(true)
        .default_width(280.0)
        .show(ctx, |ui| {
            let hist = &state.fps_history;
            if hist.len() < 2 {
                ui.label("Collecting data…");
                return;
            }
            let max_fps  = hist.iter().copied().fold(0.0_f32, f32::max).max(1.0);
            let w = ui.available_width();
            let h = 60.0_f32;
            let (resp, painter) = ui.allocate_painter(Vec2::new(w, h), egui::Sense::hover());
            let rect = resp.rect;

            painter.rect_filled(rect, 4.0, Color32::from_black_alpha(160));

            let n = hist.len();
            for i in 1..n {
                let x0 = rect.left() + (i - 1) as f32 / (n - 1) as f32 * rect.width();
                let x1 = rect.left() + i as f32 / (n - 1) as f32 * rect.width();
                let y0 = rect.bottom() - hist[i - 1] / max_fps * rect.height();
                let y1 = rect.bottom() - hist[i]     / max_fps * rect.height();
                let frac   = hist[i] / 144.0;
                let r      = ((1.0 - frac) * 220.0) as u8;
                let g      = (frac * 220.0) as u8;
                painter.line_segment(
                    [[x0, y0].into(), [x1, y1].into()],
                    Stroke::new(1.5, Color32::from_rgb(r, g, 80)),
                );
            }

            
            painter.text(
                rect.left_top() + Vec2::new(4.0, 2.0),
                egui::Align2::LEFT_TOP,
                format!("{:.0}", max_fps),
                FontId::monospace(10.0),
                Color32::LIGHT_GRAY,
            );
            painter.text(
                rect.left_bottom() + Vec2::new(4.0, -12.0),
                egui::Align2::LEFT_TOP,
                "0",
                FontId::monospace(10.0),
                Color32::GRAY,
            );

            ui.label(format!(
                "min:{:.0}  max:{:.0}  avg:{:.0}",
                if state.fps_min == f32::MAX { 0.0 } else { state.fps_min },
                state.fps_max,
                state.fps_avg
            ));
        });
}



fn draw_keyboard_shortcuts(ctx: &Context, state: &mut AppState) {
    egui::Window::new("⌨  Keyboard Shortcuts")
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            let shortcuts = [
                ("S",          "Toggle Settings panel"),
                ("D",          "Toggle Debug overlay"),
                ("H",          "Toggle this help"),
                ("P",          "Pause / Resume"),
                ("R",          "Reset rotation"),
                ("F11",        "Toggle Fullscreen"),
                ("F5",         "Screenshot"),
                ("Space",      "Pause"),
                ("LMB drag",   "Rotate object"),
                ("Scroll",     "Zoom (camera distance)"),
                ("Esc",        "Close dialogs"),
            ];
            egui::Grid::new("ksg").num_columns(2).striped(true).spacing([16.0, 4.0]).show(ui, |ui| {
                for (key, desc) in &shortcuts {
                    ui.monospace(RichText::new(*key).color(Color32::YELLOW));
                    ui.label(*desc);
                    ui.end_row();
                }
            });
            ui.add_space(6.0);
            if ui.button("Close").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.show_keyboard_help = false;
            }
        });
}



fn draw_info_dialog(ctx: &Context, state: &mut AppState) {
    egui::Window::new("ℹ  About 3DTest")
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .min_width(320.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.heading(RichText::new("3DTest").size(24.0).strong());
                ui.label(RichText::new("wgpu multi-backend 3D renderer").color(Color32::LIGHT_GRAY));
                ui.add_space(6.0);
                ui.label(RichText::new("v1.3.1").color(Color32::LIGHT_GRAY));
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                ui.label(RichText::new("Created by  mikedev_").size(15.0).strong());
                ui.add_space(4.0);

                if ui.link("Discord: mikedev_  (click to open)").clicked() {
                    let _ = webbrowser::open("https://discord.com");
                }
                ui.add_space(2.0);
                if ui.link("Github: https://github.com/hitsugori/3DTest").clicked() {
                    let _ = webbrowser::open("https://github.com/hitsugori/3DTest");
                }
                ui.add_space(2.0);
                ui.label(RichText::new("Contact: mike@mikeden.site ").color(Color32::LIGHT_GRAY));

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(RichText::new("Licensed under GPL-3.0-or-later ").small().color(Color32::GRAY));
                ui.label(RichText::new("Built with: wgpu - egui - winit - glam").small().color(Color32::GRAY));
                ui.add_space(8.0);
            });

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        state.show_info_dialog = false;
                    }
                });
            });
        });
}



fn draw_toast(ctx: &Context, msg: &str, state: &mut AppState) {
    egui::Area::new("toast".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0])
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Color32::from_black_alpha(180))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(msg).color(Color32::WHITE));
                        if ui.small_button("X").clicked() {
                            state.export_rotation_text = None;
                        }
                    });
                });
        });
}



fn section(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui)) {
    ui.collapsing(label, |ui| {
        ui.add_space(4.0);
        content(ui);
        ui.add_space(4.0);
    });
}



fn arr_to_color32(c: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
        (c[3] * 255.0) as u8,
    )
}

fn color32_to_arr(c: Color32) -> [f32; 4] {
    let [r, g, b, a] = c.to_array();
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
}

fn light_arr_to_color32(c: [f32; 3]) -> Color32 {
    arr_to_color32([c[0], c[1], c[2], 1.0])
}

fn color32_to_arr3(c: Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}



fn apply_theme(ctx: &Context, dark: bool) {
    if dark {
        ctx.set_visuals(egui::Visuals::dark());
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }
}