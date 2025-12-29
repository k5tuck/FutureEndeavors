//! Solar Voyage - Interstellar Journey Simulation
//!
//! An integrated physics simulation featuring:
//! - Accurate solar system with real planetary data
//! - Black hole with gravitational lensing
//! - Controllable spaceship with relativistic effects
//! - Spacetime curvature visualization
//!
//! Controls:
//! - Mouse drag: Orbit camera
//! - Scroll: Zoom
//! - WASD: Move spaceship (thrust)
//! - Q/E: Roll spaceship
//! - Shift: Boost thrust
//! - Space: Pause/resume
//! - Tab: Toggle camera mode (orbit/follow ship)
//! - G: Toggle spacetime grid
//! - T: Toggle trails
//! - B: Add/remove black hole
//! - 1-9: Focus on planet
//! - +/-: Time scale

mod solar_system;
mod spaceship;
mod spacetime;
mod renderer;

use common::{Camera3D, GraphicsContext};
use glam::Vec3;
use solar_system::SolarSystem;
use spaceship::Spaceship;
use spacetime::SpacetimeGrid;
use renderer::Renderer;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CameraMode {
    Orbit,
    FollowShip,
    ShipView,
}

struct App {
    ctx: GraphicsContext,
    renderer: Renderer,
    solar_system: SolarSystem,
    spaceship: Spaceship,
    spacetime_grid: SpacetimeGrid,
    camera: Camera3D,
    camera_mode: CameraMode,

    // Input state
    keys_pressed: KeyState,
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,

    // Simulation state
    paused: bool,
    show_grid: bool,
    show_trails: bool,
    has_black_hole: bool,
    focused_body: Option<usize>,
}

#[derive(Default)]
struct KeyState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    roll_left: bool,
    roll_right: bool,
    boost: bool,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer::new(&ctx);

        let mut camera = Camera3D::new(ctx.aspect_ratio());
        camera.distance = 5.0;
        camera.pitch = 0.3;
        camera.update_orbital();

        let mut solar_system = SolarSystem::new();
        solar_system.init_accurate();
        solar_system.time_scale = 0.5; // Half a year per second

        let mut spaceship = Spaceship::new();
        // Start in orbit around Earth
        if let Some(earth) = solar_system.find_body("Earth") {
            spaceship.launch_from(earth, Vec3::new(0.0, 1.0, 0.0));
        }

        let mut spacetime_grid = SpacetimeGrid::new(40, 35.0);
        spacetime_grid.deformation_scale = 50.0;

        Self {
            ctx,
            renderer,
            solar_system,
            spaceship,
            spacetime_grid,
            camera,
            camera_mode: CameraMode::Orbit,
            keys_pressed: KeyState::default(),
            mouse_pressed: false,
            last_mouse_pos: None,
            paused: false,
            show_grid: true,
            show_trails: true,
            has_black_hole: false,
            focused_body: None,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.camera.update_aspect_ratio(self.ctx.aspect_ratio());
        self.renderer.resize(&self.ctx.device, new_size.width, new_size.height);
    }

    fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        // Handle spaceship input
        let thrust_strength = if self.keys_pressed.boost { 5.0 } else { 1.0 };

        if self.keys_pressed.forward {
            self.spaceship.apply_thrust(thrust_strength, dt);
        }
        if self.keys_pressed.backward {
            self.spaceship.apply_thrust(-thrust_strength * 0.5, dt);
        }

        let rotation_speed = 1.0 * dt;
        if self.keys_pressed.left {
            self.spaceship.rotate(0.0, rotation_speed, 0.0);
        }
        if self.keys_pressed.right {
            self.spaceship.rotate(0.0, -rotation_speed, 0.0);
        }
        if self.keys_pressed.up {
            self.spaceship.rotate(-rotation_speed, 0.0, 0.0);
        }
        if self.keys_pressed.down {
            self.spaceship.rotate(rotation_speed, 0.0, 0.0);
        }
        if self.keys_pressed.roll_left {
            self.spaceship.rotate(0.0, 0.0, rotation_speed);
        }
        if self.keys_pressed.roll_right {
            self.spaceship.rotate(0.0, 0.0, -rotation_speed);
        }

        // Update simulation
        let substeps = 4;
        let sub_dt = dt / substeps as f32;
        for _ in 0..substeps {
            self.solar_system.step(sub_dt);
            self.spaceship.update(&self.solar_system.bodies, sub_dt);
        }

        // Update spacetime grid
        if self.show_grid {
            self.spacetime_grid.update(&self.solar_system.bodies);
        }

        // Update camera based on mode
        match self.camera_mode {
            CameraMode::Orbit => {
                if let Some(idx) = self.focused_body {
                    if idx < self.solar_system.bodies.len() {
                        self.camera.target = self.solar_system.bodies[idx].position;
                    }
                }
                self.camera.update_orbital();
            }
            CameraMode::FollowShip => {
                self.camera.target = self.spaceship.position;
                self.camera.update_orbital();
            }
            CameraMode::ShipView => {
                self.camera.position = self.spaceship.position;
                self.camera.target = self.spaceship.position + self.spaceship.forward();
                self.camera.up = self.spaceship.up();
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let grid = if self.show_grid { Some(&self.spacetime_grid) } else { None };

        let render_data = self.renderer.update(
            &self.ctx.queue,
            &self.camera,
            &self.solar_system,
            &self.spaceship,
            grid,
        );

        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.renderer.render(
            &mut encoder,
            &view,
            &render_data,
            self.show_grid,
            self.show_trails,
        );

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        match key {
            // Movement
            KeyCode::KeyW => self.keys_pressed.forward = pressed,
            KeyCode::KeyS => self.keys_pressed.backward = pressed,
            KeyCode::KeyA => self.keys_pressed.left = pressed,
            KeyCode::KeyD => self.keys_pressed.right = pressed,
            KeyCode::KeyR => self.keys_pressed.up = pressed,
            KeyCode::KeyF => self.keys_pressed.down = pressed,
            KeyCode::KeyQ => self.keys_pressed.roll_left = pressed,
            KeyCode::KeyE => self.keys_pressed.roll_right = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.keys_pressed.boost = pressed,

            _ if pressed => {
                // Only handle on press
                match key {
                    KeyCode::Space => self.paused = !self.paused,
                    KeyCode::Tab => {
                        self.camera_mode = match self.camera_mode {
                            CameraMode::Orbit => CameraMode::FollowShip,
                            CameraMode::FollowShip => CameraMode::ShipView,
                            CameraMode::ShipView => CameraMode::Orbit,
                        };
                        println!("Camera: {:?}", self.camera_mode);
                    }
                    KeyCode::KeyG => self.show_grid = !self.show_grid,
                    KeyCode::KeyT => self.show_trails = !self.show_trails,
                    KeyCode::KeyB => {
                        if self.has_black_hole {
                            // Remove black hole (keep only first 9 bodies)
                            self.solar_system.bodies.truncate(9);
                            self.has_black_hole = false;
                            println!("Black hole removed");
                        } else {
                            // Add a stellar-mass black hole approaching the solar system
                            self.solar_system.add_black_hole(
                                10.0, // 10 solar masses
                                Vec3::new(50.0, 5.0, 30.0),
                                Vec3::new(-2.0, -0.2, -1.0),
                            );
                            self.has_black_hole = true;
                            println!("Black hole added! Mass: 10 solar masses");
                        }
                    }
                    KeyCode::Equal | KeyCode::NumpadAdd => {
                        self.solar_system.time_scale *= 2.0;
                        println!("Time scale: {:.1}x", self.solar_system.time_scale);
                    }
                    KeyCode::Minus | KeyCode::NumpadSubtract => {
                        self.solar_system.time_scale /= 2.0;
                        println!("Time scale: {:.1}x", self.solar_system.time_scale);
                    }
                    // Focus on bodies
                    KeyCode::Digit0 => {
                        self.focused_body = Some(0); // Sun
                        self.camera.distance = 5.0;
                    }
                    KeyCode::Digit1 => {
                        self.focused_body = Some(3); // Earth
                        self.camera.distance = 0.5;
                    }
                    KeyCode::Digit2 => {
                        self.focused_body = Some(5); // Jupiter
                        self.camera.distance = 2.0;
                    }
                    KeyCode::Digit3 => {
                        self.focused_body = Some(6); // Saturn
                        self.camera.distance = 2.0;
                    }
                    KeyCode::Escape => {
                        self.focused_body = None;
                        self.camera.distance = 5.0;
                        self.camera.target = Vec3::ZERO;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.mouse_pressed && self.camera_mode != CameraMode::ShipView {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                let dx = (x - last_x) as f32 * 0.01;
                let dy = (y - last_y) as f32 * 0.01;
                self.camera.orbit(dx, dy);
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        if self.camera_mode != CameraMode::ShipView {
            self.camera.zoom(delta * self.camera.distance * 0.1);
        }
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           SOLAR VOYAGE - Interstellar Journey             ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║ Controls:                                                 ║");
    println!("║   WASD      - Rotate spaceship                           ║");
    println!("║   R/F       - Pitch up/down                              ║");
    println!("║   Q/E       - Roll left/right                            ║");
    println!("║   W+Shift   - Boost thrust                               ║");
    println!("║   Tab       - Toggle camera mode                         ║");
    println!("║   Space     - Pause/Resume                               ║");
    println!("║   G         - Toggle spacetime grid                      ║");
    println!("║   T         - Toggle orbital trails                      ║");
    println!("║   B         - Add/Remove black hole                      ║");
    println!("║   0-3       - Focus on celestial body                    ║");
    println!("║   +/-       - Adjust time scale                          ║");
    println!("║   Scroll    - Zoom                                       ║");
    println!("║   Drag      - Orbit camera                               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Starting simulation...");
    println!();

    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Solar Voyage - Interstellar Journey",
        1280,
        720,
    ));

    let mut app = App::new(ctx);
    let mut last_time = std::time::Instant::now();
    let mut frame_count = 0u32;
    let mut fps_timer = std::time::Instant::now();

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
                        event: KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state,
                            ..
                        },
                        ..
                    } => app.handle_key(key, state == ElementState::Pressed),
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

                        // FPS counter
                        frame_count += 1;
                        if fps_timer.elapsed().as_secs_f32() >= 2.0 {
                            let fps = frame_count as f32 / fps_timer.elapsed().as_secs_f32();
                            println!(
                                "FPS: {:.1} | Ship: {} | Time: {:.2} years",
                                fps,
                                app.spaceship.info_string().lines().next().unwrap_or(""),
                                app.solar_system.time
                            );
                            frame_count = 0;
                            fps_timer = std::time::Instant::now();
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
