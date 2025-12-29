//! Quantum Tunneling Visualization
//!
//! Demonstrates wave packet transmission through potential barriers.
//!
//! Controls:
//! - Space: Pause/resume
//! - 1/2/3: Switch presets (single barrier, double barrier, step)
//! - R: Reset simulation
//! - +/-: Adjust wave packet momentum
//! - Up/Down: Adjust barrier height

mod wavefunction;
mod quantum_state;
mod tunneling;
mod orbitals;
mod teleportation;
mod quarks;
mod hall_effect;
mod hypercube;
mod renderer;

use common::{Camera2D, GraphicsContext};
use glam::Vec3;
use tunneling::{Barrier, TunnelingSimulation};
use renderer::{QuantumRenderer, PointInstance};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

struct App {
    ctx: GraphicsContext,
    renderer: QuantumRenderer,
    simulation: TunnelingSimulation,
    camera: Camera2D,
    paused: bool,
    current_preset: u8,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = QuantumRenderer::new(&ctx, 1024, 100);
        let mut camera = Camera2D::new(ctx.aspect_ratio());
        camera.zoom = 12.0;

        let simulation = TunnelingSimulation::preset_single_barrier();

        Self {
            ctx,
            renderer,
            simulation,
            camera,
            paused: false,
            current_preset: 1,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.camera.update_aspect_ratio(self.ctx.aspect_ratio());
    }

    fn update(&mut self, dt: f32) {
        if !self.paused {
            // Multiple substeps for stability
            let substeps = 20;
            for _ in 0..substeps {
                self.simulation.step();
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update_camera_2d(&self.ctx.queue, &self.camera);

        // Convert wavefunction data to points
        let render_data = self.simulation.get_render_data();
        let points: Vec<PointInstance> = render_data
            .iter()
            .map(|(x, prob, potential, color)| {
                // Show wavefunction as vertical displacement
                let y = prob.sqrt() * 3.0;
                PointInstance {
                    position: [*x, y, 0.0],
                    size: 0.08,
                    color: *color,
                }
            })
            .collect();

        // Add potential barrier visualization
        let potential = self.simulation.potential_profile();
        let mut barrier_points: Vec<PointInstance> = potential
            .iter()
            .enumerate()
            .filter(|(_, v)| **v > 0.1)
            .map(|(i, v)| {
                let x = self.simulation.wavefunction.x_at(i);
                PointInstance {
                    position: [x, v / 5.0, 0.0],
                    size: 0.1,
                    color: [0.5, 0.5, 0.5, 0.5],
                }
            })
            .collect();

        let mut all_points = points;
        all_points.append(&mut barrier_points);

        self.renderer.update_points(&self.ctx.queue, &all_points);

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer
            .render_points(&mut encoder, &view, all_points.len() as u32, true);

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
            _ => {}
        }
    }

    fn load_preset(&mut self, preset: u8) {
        self.current_preset = preset;
        self.simulation = match preset {
            1 => TunnelingSimulation::preset_single_barrier(),
            2 => TunnelingSimulation::preset_resonant_tunneling(),
            3 => TunnelingSimulation::preset_step_potential(),
            _ => TunnelingSimulation::preset_single_barrier(),
        };
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Quantum Tunneling - Wave Packet Simulation",
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
