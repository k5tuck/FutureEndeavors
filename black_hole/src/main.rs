//! 3D Black Hole Visualization with Ray Marching
//!
//! Full 3D visualization of a Schwarzschild black hole featuring:
//! - Gravitational lensing of background stars
//! - Accretion disk with temperature-based coloring
//! - Doppler effects from disk rotation
//! - Photon sphere visualization
//!
//! Controls:
//! - Left mouse drag: Orbit camera
//! - Scroll: Zoom in/out
//! - +/-: Adjust black hole mass
//! - Space: Pause/resume disk animation
//! - R: Reset view

mod physics;
mod renderer;

use common::GraphicsContext;
use glam::Vec3;
use physics::BlackHole;
use renderer::Renderer3D;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

struct App {
    ctx: GraphicsContext,
    renderer: Renderer3D,
    black_hole: BlackHole,
    camera_distance: f32,
    camera_yaw: f32,
    camera_pitch: f32,
    time: f32,
    paused: bool,
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer3D::new(&ctx);
        let black_hole = BlackHole::new(Vec3::ZERO, 1.0);

        Self {
            ctx,
            renderer,
            black_hole,
            camera_distance: 20.0,
            camera_yaw: 0.0,
            camera_pitch: 0.3,
            time: 0.0,
            paused: false,
            mouse_pressed: false,
            last_mouse_pos: None,
        }
    }

    fn camera_position(&self) -> [f32; 3] {
        [
            self.camera_distance * self.camera_pitch.cos() * self.camera_yaw.sin(),
            self.camera_distance * self.camera_pitch.sin(),
            self.camera_distance * self.camera_pitch.cos() * self.camera_yaw.cos(),
        ]
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
    }

    fn update(&mut self, dt: f32) {
        if !self.paused {
            self.time += dt;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update(
            &self.ctx.queue,
            self.camera_position(),
            &self.black_hole,
            self.time,
        );

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer.render(&mut encoder, &view);

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
            KeyCode::KeyR => {
                self.camera_distance = 20.0;
                self.camera_yaw = 0.0;
                self.camera_pitch = 0.3;
            }
            KeyCode::Equal | KeyCode::NumpadAdd => {
                self.black_hole = BlackHole::new(Vec3::ZERO, self.black_hole.mass * 1.2);
            }
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                self.black_hole = BlackHole::new(Vec3::ZERO, (self.black_hole.mass / 1.2).max(0.1));
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.mouse_pressed {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                let dx = (x - last_x) as f32 * 0.01;
                let dy = (y - last_y) as f32 * 0.01;

                self.camera_yaw += dx;
                self.camera_pitch = (self.camera_pitch + dy).clamp(-1.5, 1.5);
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera_distance = (self.camera_distance - delta * 2.0).clamp(5.0, 100.0);
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Black Hole - 3D Visualization",
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
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            app.mouse_pressed = state == ElementState::Pressed;
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
