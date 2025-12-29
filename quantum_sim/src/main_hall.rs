//! Quantum Hall Effect Visualization
//!
//! Simulates 2D electron gas in a magnetic field showing Landau levels and edge states.
//!
//! Controls:
//! - Space: Pause/resume
//! - Up/Down: Adjust magnetic field
//! - +/-: Add/remove electrons
//! - 1/2: Preset filling factors ν=1 and ν=2

mod wavefunction;
mod quantum_state;
mod tunneling;
mod orbitals;
mod teleportation;
mod quarks;
mod hall_effect;
mod hypercube;
mod renderer;
mod equations_ui;

use common::{Camera2D, GraphicsContext};
use glam::Vec3;
use hall_effect::HallSimulation;
use renderer::{QuantumRenderer, PointInstance};
use equations_ui::{draw_equations_sidebar, HALL_EQUATIONS, HALL_VARIABLES};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

struct EguiState {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

struct App {
    ctx: GraphicsContext,
    renderer: QuantumRenderer,
    simulation: HallSimulation,
    camera: Camera2D,
    paused: bool,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 500, 200);
        let mut camera = Camera2D::new(ctx.aspect_ratio());
        camera.zoom = 8.0;

        let simulation = HallSimulation::default();

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

    fn update(&mut self, dt: f32) {
        if !self.paused {
            self.simulation.step(dt);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update_camera_2d(&self.ctx.queue, &self.camera);

        let electron_data = self.simulation.get_electron_data();
        let points: Vec<PointInstance> = electron_data
            .iter()
            .map(|(pos, color, is_edge)| {
                let size = if *is_edge { 0.15 } else { 0.1 };
                PointInstance {
                    position: [pos.x, pos.y, 0.0],
                    size,
                    color: *color,
                }
            })
            .collect();

        self.renderer.update_points(&self.ctx.queue, &points);

        let orbits = self.simulation.get_orbits();
        let mut lines: Vec<(Vec3, Vec3, [f32; 4])> = Vec::new();

        for (center, radius, color) in orbits.iter().take(20) {
            let segments = 16;
            for i in 0..segments {
                let a1 = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
                let a2 = (i + 1) as f32 * 2.0 * std::f32::consts::PI / segments as f32;
                let p1 = Vec3::new(center.x + radius * a1.cos(), center.y + radius * a1.sin(), 0.0);
                let p2 = Vec3::new(center.x + radius * a2.cos(), center.y + radius * a2.sin(), 0.0);
                lines.push((p1, p2, [color[0], color[1], color[2], 0.3]));
            }
        }

        let hw = self.simulation.width / 2.0;
        let hh = self.simulation.height / 2.0;
        lines.push((Vec3::new(-hw, -hh, 0.0), Vec3::new(hw, -hh, 0.0), [0.5, 0.5, 0.5, 0.5]));
        lines.push((Vec3::new(hw, -hh, 0.0), Vec3::new(hw, hh, 0.0), [0.5, 0.5, 0.5, 0.5]));
        lines.push((Vec3::new(hw, hh, 0.0), Vec3::new(-hw, hh, 0.0), [0.5, 0.5, 0.5, 0.5]));
        lines.push((Vec3::new(-hw, hh, 0.0), Vec3::new(-hw, -hh, 0.0), [0.5, 0.5, 0.5, 0.5]));

        self.renderer.update_lines(&self.ctx.queue, &lines);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "Quantum Hall Effect",
                HALL_EQUATIONS,
                HALL_VARIABLES,
            );

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("B = {:.2} T", self.simulation.magnetic_field));
                    ui.separator();
                    ui.label(format!("ν = {:.2}", self.simulation.filling_factor));
                    ui.separator();
                    ui.label(format!("σ_xy = {:.0} e²/h", self.simulation.hall_conductance));
                    ui.separator();
                    ui.label(format!("Electrons: {}", self.simulation.electrons.len()));
                    if self.paused {
                        ui.label(egui::RichText::new("PAUSED").color(egui::Color32::YELLOW));
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
            .render_lines(&mut encoder, &view, lines.len() as u32, true);
        self.renderer
            .render_points(&mut encoder, &view, points.len() as u32, false);

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

        let n_electrons = self.simulation.electrons.len();

        match key {
            KeyCode::Space => self.paused = !self.paused,
            KeyCode::ArrowUp => {
                self.simulation.set_magnetic_field(self.simulation.magnetic_field + 0.2);
            }
            KeyCode::ArrowDown => {
                self.simulation.set_magnetic_field(self.simulation.magnetic_field - 0.2);
            }
            KeyCode::Equal => {
                self.simulation.fill_electrons(n_electrons + 10);
            }
            KeyCode::Minus => {
                if n_electrons > 10 {
                    self.simulation.fill_electrons(n_electrons - 10);
                }
            }
            KeyCode::Digit1 => {
                self.simulation = HallSimulation::preset_nu_1();
            }
            KeyCode::Digit2 => {
                self.simulation = HallSimulation::preset_nu_2();
            }
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera.zoom *= 1.0 - delta * 0.1;
        self.camera.zoom = self.camera.zoom.clamp(2.0, 20.0);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Quantum Hall Effect - Landau Levels",
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
