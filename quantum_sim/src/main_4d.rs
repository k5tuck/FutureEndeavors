//! 4D Hypercube (Tesseract) Visualization
//!
//! Projects 4-dimensional polytopes into 3D space with animated rotation.
//!
//! Controls:
//! - Space: Toggle auto-rotation
//! - 1: Tesseract (8-cell)
//! - 2: 16-cell
//! - 3: 24-cell
//! - 4: 5-cell (pentatope)
//! - Q/W: Rotate in XW plane
//! - E/R: Rotate in YW plane
//! - T/Y: Rotate in ZW plane
//! - Arrow keys: Rotate 3D view
//! - P: Toggle perspective/orthographic

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

use common::{Camera3D, GraphicsContext};
use glam::Vec3;
use hypercube::{Hypercube4DSimulation, Polytope4D};
use renderer::{QuantumRenderer, PointInstance, hypercube_to_points};
use equations_ui::{draw_equations_sidebar, HYPERCUBE_EQUATIONS, HYPERCUBE_VARIABLES};
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
    simulation: Hypercube4DSimulation,
    camera: Camera3D,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 50, 200);
        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 6.0;

        let simulation = Hypercube4DSimulation::preset_tesseract();

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
        self.simulation.step(dt);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update_camera_3d(&self.ctx.queue, &self.camera);

        // Render vertices
        let vertex_data = self.simulation.get_vertices_3d();
        let points = hypercube_to_points(&vertex_data);
        self.renderer.update_points(&self.ctx.queue, &points);

        // Render edges
        let edges = self.simulation.get_edges_3d();
        self.renderer.update_lines(&self.ctx.queue, &edges);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let polytope_name = self.simulation.current_polytope_name();
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "4D Geometry",
                HYPERCUBE_EQUATIONS,
                HYPERCUBE_VARIABLES,
            );

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Polytope: {}", polytope_name));
                    ui.separator();
                    ui.label(format!("Vertices: {}", self.simulation.polytope.vertices.len()));
                    ui.separator();
                    ui.label(format!("Edges: {}", self.simulation.polytope.edges.len()));
                    ui.separator();
                    if self.simulation.auto_rotate {
                        ui.label(egui::RichText::new("AUTO-ROTATE").color(egui::Color32::GREEN));
                    }
                    if self.simulation.use_perspective {
                        ui.label("Perspective");
                    } else {
                        ui.label("Orthographic");
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
            .render_lines(&mut encoder, &view, edges.len() as u32, true);
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

        match key {
            KeyCode::Space => self.simulation.auto_rotate = !self.simulation.auto_rotate,
            KeyCode::Digit1 => self.simulation.set_polytope(Polytope4D::tesseract(1.0)),
            KeyCode::Digit2 => self.simulation.set_polytope(Polytope4D::cell_16(1.0)),
            KeyCode::Digit3 => self.simulation.set_polytope(Polytope4D::cell_24(0.7)),
            KeyCode::Digit4 => self.simulation.set_polytope(Polytope4D::simplex_5(0.8)),
            KeyCode::KeyQ => self.simulation.rotate_xw(0.1),
            KeyCode::KeyW => self.simulation.rotate_xw(-0.1),
            KeyCode::KeyE => self.simulation.rotate_yw(0.1),
            KeyCode::KeyR => self.simulation.rotate_yw(-0.1),
            KeyCode::KeyT => self.simulation.rotate_zw(0.1),
            KeyCode::KeyY => self.simulation.rotate_zw(-0.1),
            KeyCode::KeyP => self.simulation.use_perspective = !self.simulation.use_perspective,
            KeyCode::ArrowLeft => self.camera.orbit(-0.1, 0.0),
            KeyCode::ArrowRight => self.camera.orbit(0.1, 0.0),
            KeyCode::ArrowUp => self.camera.orbit(0.0, 0.1),
            KeyCode::ArrowDown => self.camera.orbit(0.0, -0.1),
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "4D Visualization - Tesseract",
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
