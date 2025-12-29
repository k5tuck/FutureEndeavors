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

use common::{Camera3D, GraphicsContext};
use glam::Vec3;
use hypercube::{Hypercube4DSimulation, Polytope4D};
use renderer::{QuantumRenderer, PointInstance, hypercube_to_points};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

struct App {
    ctx: GraphicsContext,
    renderer: QuantumRenderer,
    simulation: Hypercube4DSimulation,
    camera: Camera3D,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 50, 200);
        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 6.0;

        let simulation = Hypercube4DSimulation::preset_tesseract();

        Self {
            ctx,
            renderer,
            simulation,
            camera,
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
}

fn main() {
    println!("4D Visualization - Tesseract & Polytopes");
    println!("========================================");
    println!("1: Tesseract (hypercube)");
    println!("2: 16-cell (cross-polytope)");
    println!("3: 24-cell");
    println!("4: 5-cell (pentatope)");
    println!("Q/W, E/R, T/Y: Rotate in 4D");
    println!("Space: Toggle auto-rotation");
    println!("P: Toggle perspective/orthographic\n");

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
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(size) => app.resize(size),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key),
                                state,
                                ..
                            },
                        ..
                    } => app.handle_key(key, state),
                    WindowEvent::MouseWheel { delta, .. } => {
                        let scroll = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
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
                },
                Event::AboutToWait => {
                    app.ctx.window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
