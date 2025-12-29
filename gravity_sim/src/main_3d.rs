//! 3D N-body Gravity Simulation
//!
//! Full 3D gravitational simulation with:
//! - Billboard particle rendering with lighting
//! - Orbital trails
//! - Star field background
//! - Multiple camera modes
//!
//! Controls:
//! - Left mouse drag: Orbit camera
//! - Scroll: Zoom in/out
//! - 1/2/3: Load presets (Solar System, Accretion Disk, Galaxy Collision)
//! - Space: Pause/resume
//! - T: Toggle trails
//! - G: Toggle grid
//! - R: Reset view
//! - +/-: Adjust time scale

mod physics;
mod physics_3d;
mod renderer;
mod renderer_3d;
mod equations_ui;

use common::{Camera3D, GraphicsContext};
use physics_3d::Simulation3D;
use renderer_3d::Renderer3D;
use equations_ui::{draw_equations_sidebar, GRAVITY_3D_EQUATIONS, GRAVITY_3D_VARIABLES};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

const MAX_PARTICLES: usize = 2000;

struct EguiState {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

struct App {
    ctx: GraphicsContext,
    renderer: Renderer3D,
    simulation: Simulation3D,
    camera: Camera3D,
    paused: bool,
    show_grid: bool,
    show_trails: bool,
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,
    current_preset: u8,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer3D::new(&ctx, MAX_PARTICLES);
        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 40.0;
        camera.pitch = 0.4;
        camera.update_orbital();

        let mut simulation = Simulation3D::new();
        simulation.init_solar_system();

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &ctx.window,
            Some(ctx.window.scale_factor() as f32),
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &ctx.device,
            ctx.config.format,
            None,
            1,
        );

        Self {
            ctx,
            renderer,
            simulation,
            camera,
            paused: false,
            show_grid: true,
            show_trails: true,
            mouse_pressed: false,
            last_mouse_pos: None,
            current_preset: 1,
            egui: EguiState {
                ctx: egui_ctx,
                state: egui_state,
                renderer: egui_renderer,
            },
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.camera.update_aspect_ratio(self.ctx.aspect_ratio());
        self.renderer
            .resize(&self.ctx.device, new_size.width, new_size.height);
    }

    fn update(&mut self, dt: f32) {
        if !self.paused {
            let substeps = 4;
            let sub_dt = dt / substeps as f32;
            for _ in 0..substeps {
                self.simulation.step(sub_dt);
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update_camera(&self.ctx.queue, &self.camera);
        let (num_instances, trail_ranges) =
            self.renderer.update_simulation(&self.ctx.queue, &self.simulation);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "3D Orbital Mechanics",
                GRAVITY_3D_EQUATIONS,
                GRAVITY_3D_VARIABLES,
            );

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Bodies: {}", self.simulation.bodies.len()));
                    ui.separator();
                    let preset_name = match self.current_preset {
                        1 => "Solar System",
                        2 => "Accretion Disk",
                        3 => "Galaxy Collision",
                        _ => "Custom",
                    };
                    ui.label(format!("Preset: {}", preset_name));
                    ui.separator();
                    ui.label(format!("Time: {:.1}x", self.simulation.time_scale));
                    ui.separator();
                    if self.paused {
                        ui.label(egui::RichText::new("PAUSED").color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("RUNNING").color(egui::Color32::GREEN));
                    }
                    if self.show_trails {
                        ui.separator();
                        ui.label("Trails ON");
                    }
                });
            });
        });

        self.egui.state.handle_platform_output(&self.ctx.window, full_output.platform_output);
        let tris = self.egui.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui.renderer.update_texture(&self.ctx.device, &self.ctx.queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.ctx.size.width, self.ctx.size.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer.render(
            &mut encoder,
            &view,
            num_instances,
            &trail_ranges,
            self.show_grid,
            self.show_trails,
        );

        self.egui.renderer.update_buffers(
            &self.ctx.device,
            &self.ctx.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.egui.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui.renderer.free_texture(id);
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match key {
            KeyCode::Space => self.paused = !self.paused,
            KeyCode::KeyG => self.show_grid = !self.show_grid,
            KeyCode::KeyT => self.show_trails = !self.show_trails,
            KeyCode::KeyR => {
                self.camera.distance = 40.0;
                self.camera.pitch = 0.4;
                self.camera.yaw = 0.0;
                self.camera.target = glam::Vec3::ZERO;
                self.camera.update_orbital();
            }
            KeyCode::Digit1 => {
                self.current_preset = 1;
                self.simulation.init_solar_system();
                self.camera.distance = 40.0;
            }
            KeyCode::Digit2 => {
                self.current_preset = 2;
                self.simulation.init_accretion_disk(500);
                self.camera.distance = 50.0;
            }
            KeyCode::Digit3 => {
                self.current_preset = 3;
                self.simulation.init_galaxy_collision(300);
                self.camera.distance = 80.0;
            }
            KeyCode::Equal | KeyCode::NumpadAdd => {
                self.simulation.time_scale *= 1.5;
            }
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                self.simulation.time_scale /= 1.5;
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.mouse_pressed {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                let dx = (x - last_x) as f32 * 0.01;
                let dy = (y - last_y) as f32 * 0.01;
                self.camera.orbit(dx, dy);
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera.zoom(delta * 3.0);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "3D Gravity Simulation - Rust/wgpu",
        1280,
        720,
    ));

    let mut app = App::new(ctx);
    let mut last_time = std::time::Instant::now();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { ref event, .. } => {
                    let consumed = app.handle_window_event(event);

                    if !consumed {
                        match event {
                            WindowEvent::CloseRequested => elwt.exit(),
                            WindowEvent::Resized(size) => app.resize(*size),
                            WindowEvent::MouseInput { state, button, .. } => {
                                if *button == MouseButton::Left {
                                    app.mouse_pressed = *state == ElementState::Pressed;
                                    if !app.mouse_pressed {
                                        app.last_mouse_pos = None;
                                    }
                                }
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                app.handle_mouse_move(position.x, position.y);
                            }
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        physical_key: PhysicalKey::Code(key),
                                        state,
                                        ..
                                    },
                                ..
                            } => app.handle_key(*key, *state),
                            WindowEvent::MouseWheel { delta, .. } => {
                                let scroll = match delta {
                                    MouseScrollDelta::LineDelta(_, y) => *y,
                                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                                };
                                app.handle_scroll(scroll);
                            }
                            WindowEvent::RedrawRequested => {
                                let now = std::time::Instant::now();
                                let dt = (now - last_time).as_secs_f32().min(0.1);
                                last_time = now;

                                app.update(dt);
                                match app.render() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => app.resize(app.ctx.size),
                                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                    Err(e) => eprintln!("Render error: {:?}", e),
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::AboutToWait => {
                    app.ctx.window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
