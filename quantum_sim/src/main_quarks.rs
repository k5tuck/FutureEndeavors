//! Quark and Hadron Visualization
//!
//! Simulates quarks, color charge, and QCD confinement.
//!
//! Controls:
//! - 1: Proton (uud)
//! - 2: Neutron (udd)
//! - 3: Pion+ (ud̄)
//! - 4: J/ψ (cc̄)
//! - Space: Pause/resume
//! - Arrow keys: Rotate view

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
use quarks::QuarkSimulation;
use renderer::{QuantumRenderer, PointInstance, quarks_to_points};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

struct App {
    ctx: GraphicsContext,
    renderer: QuantumRenderer,
    simulation: QuarkSimulation,
    camera: Camera3D,
    paused: bool,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 100, 200);
        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 5.0;

        let mut simulation = QuarkSimulation::new();
        simulation.init_proton();

        Self {
            ctx,
            renderer,
            simulation,
            camera,
            paused: false,
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

        self.renderer.update_camera_3d(&self.ctx.queue, &self.camera);

        // Render quarks
        let quark_data = self.simulation.get_quark_data();
        let points = quarks_to_points(&quark_data);
        self.renderer.update_points(&self.ctx.queue, &points);

        // Render flux tubes
        let mut lines: Vec<(Vec3, Vec3, [f32; 4])> = Vec::new();
        for tube in &self.simulation.flux_tubes {
            let p1 = self.simulation.quarks[tube.quark_a].position;
            let p2 = self.simulation.quarks[tube.quark_b].position;
            lines.push((p1, p2, tube.color_flow));
        }

        // Render gluons as small connecting lines
        for gluon in &self.simulation.gluons {
            let color = [
                (gluon.color.render_color()[0] + gluon.anticolor.render_color()[0]) / 2.0,
                (gluon.color.render_color()[1] + gluon.anticolor.render_color()[1]) / 2.0,
                (gluon.color.render_color()[2] + gluon.anticolor.render_color()[2]) / 2.0,
                0.6,
            ];
            lines.push((
                gluon.position - gluon.velocity.normalize() * 0.1,
                gluon.position + gluon.velocity.normalize() * 0.1,
                color,
            ));
        }

        self.renderer.update_lines(&self.ctx.queue, &lines);

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
            KeyCode::Digit1 => self.simulation.init_proton(),
            KeyCode::Digit2 => self.simulation.init_neutron(),
            KeyCode::Digit3 => self.simulation.init_pion_plus(),
            KeyCode::Digit4 => self.simulation.init_jpsi(),
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
    println!("Quark Confinement Simulation");
    println!("============================");
    println!("Press 1: Proton (uud)");
    println!("Press 2: Neutron (udd)");
    println!("Press 3: Pion+ (ud̄)");
    println!("Press 4: J/ψ (cc̄)\n");

    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Quarks & Hadrons - QCD Visualization",
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
