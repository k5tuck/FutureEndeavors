//! Quantum Teleportation Visualization
//!
//! Interactive demonstration of the quantum teleportation protocol.
//!
//! Controls:
//! - Enter/Space: Advance to next stage
//! - R: Reset simulation
//! - 1-4: Set different initial states to teleport

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
use teleportation::TeleportationSimulation;
use renderer::{QuantumRenderer, PointInstance};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};
use std::f32::consts::PI;

struct App {
    ctx: GraphicsContext,
    renderer: QuantumRenderer,
    simulation: TeleportationSimulation,
    camera: Camera3D,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 100, 50);
        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 12.0;
        camera.pitch = 0.3;

        let simulation = TeleportationSimulation::new();

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

        // Create qubit visualizations
        let mut points: Vec<PointInstance> = Vec::new();

        for qubit in &self.simulation.qubits {
            // Main qubit sphere
            points.push(PointInstance {
                position: [qubit.position.x, qubit.position.y, qubit.position.z],
                size: 0.5,
                color: qubit.color,
            });

            // Bloch vector tip
            let tip = qubit.position + qubit.bloch_vector * 0.6;
            points.push(PointInstance {
                position: [tip.x, tip.y, tip.z],
                size: 0.15,
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }

        self.renderer.update_points(&self.ctx.queue, &points);

        // Entanglement lines
        let lines: Vec<(Vec3, Vec3, [f32; 4])> = self.simulation.entanglement_links
            .iter()
            .map(|link| {
                let p1 = self.simulation.qubits[link.qubit_a].position;
                let p2 = self.simulation.qubits[link.qubit_b].position;
                (p1, p2, link.color)
            })
            .collect();

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

        // Print stage info
        println!("\r{} (Fidelity: {:.2})",
                 self.simulation.stage_description(),
                 self.simulation.fidelity);

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match key {
            KeyCode::Space | KeyCode::Enter => self.simulation.next_stage(),
            KeyCode::KeyR => self.simulation.reset(),
            KeyCode::Digit1 => self.simulation.set_state_to_teleport(0.0, 0.0),
            KeyCode::Digit2 => self.simulation.set_state_to_teleport(PI, 0.0),
            KeyCode::Digit3 => self.simulation.set_state_to_teleport(PI / 2.0, 0.0),
            KeyCode::Digit4 => self.simulation.set_state_to_teleport(PI / 3.0, PI / 4.0),
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
    println!("Quantum Teleportation Protocol");
    println!("==============================");
    println!("Press Space/Enter to advance through stages");
    println!("Press 1-4 to set different initial states");
    println!("Press R to reset\n");

    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Quantum Teleportation - Bell State Protocol",
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
