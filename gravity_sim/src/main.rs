//! 2D N-body Gravity Simulation
//!
//! A real-time gravitational simulation using Newtonian physics,
//! rendered with wgpu. Features include:
//! - N-body gravitational interactions
//! - Multiple preset configurations (solar system, disk, galaxy collision)
//! - Interactive camera controls
//!
//! Controls:
//! - Scroll: Zoom in/out
//! - Arrow keys / WASD: Pan camera
//! - Space: Pause/resume simulation
//! - 1/2/3: Load different presets
//! - R: Reset current simulation

mod physics;
mod renderer;
mod equations_ui;

use common::{Camera2D, GraphicsContext};
use glam::Vec3;
use physics::Simulation;
use renderer::Renderer;
use equations_ui::{draw_equations_sidebar, GRAVITY_EQUATIONS, GRAVITY_VARIABLES};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
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
    renderer: Renderer,
    simulation: Simulation,
    camera: Camera2D,
    paused: bool,
    current_preset: u8,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer::new(&ctx, MAX_PARTICLES);
        let camera = Camera2D::new(ctx.aspect_ratio());

        let mut simulation = Simulation::new();
        simulation.init_solar_system();

        let mut camera = camera;
        camera.zoom = 15.0;

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
    }

    fn update(&mut self, _dt: f32) {
        if !self.paused {
            // Substep for stability
            let substeps = 4;
            let sub_dt = _dt / substeps as f32;
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

        // Update GPU buffers
        self.renderer.update_camera(&self.ctx.queue, &self.camera);
        self.renderer
            .update_instances(&self.ctx.queue, &self.simulation.bodies);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "Gravity Simulation",
                GRAVITY_EQUATIONS,
                GRAVITY_VARIABLES,
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
                    if self.paused {
                        ui.label(egui::RichText::new("PAUSED").color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("RUNNING").color(egui::Color32::GREEN));
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

        self.renderer
            .render(&mut encoder, &view, self.simulation.bodies.len() as u32);

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
            KeyCode::KeyR => self.load_preset(self.current_preset),
            KeyCode::Digit1 => self.load_preset(1),
            KeyCode::Digit2 => self.load_preset(2),
            KeyCode::Digit3 => self.load_preset(3),
            KeyCode::ArrowUp | KeyCode::KeyW => self.camera.position.y += self.camera.zoom * 0.1,
            KeyCode::ArrowDown | KeyCode::KeyS => self.camera.position.y -= self.camera.zoom * 0.1,
            KeyCode::ArrowLeft | KeyCode::KeyA => self.camera.position.x -= self.camera.zoom * 0.1,
            KeyCode::ArrowRight | KeyCode::KeyD => self.camera.position.x += self.camera.zoom * 0.1,
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera.zoom *= 1.0 - delta * 0.1;
        self.camera.zoom = self.camera.zoom.clamp(1.0, 100.0);
    }

    fn load_preset(&mut self, preset: u8) {
        self.current_preset = preset;
        match preset {
            1 => {
                self.simulation.init_solar_system();
                self.camera.zoom = 15.0;
                self.camera.position = Vec3::ZERO;
            }
            2 => {
                self.simulation.init_disk(500);
                self.camera.zoom = 20.0;
                self.camera.position = Vec3::ZERO;
            }
            3 => {
                self.simulation.init_galaxy_collision(300);
                self.camera.zoom = 25.0;
                self.camera.position = Vec3::ZERO;
            }
            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Gravity Simulation - Rust/wgpu",
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
