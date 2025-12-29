//! 3D N-body Gravity Simulation with spacetime grid visualization
//!
//! Extends the 2D simulation with:
//! - 3D particle rendering
//! - Orbital camera controls
//! - Spacetime curvature grid visualization
//!
//! Controls:
//! - Left mouse drag: Orbit camera
//! - Right mouse drag: Pan camera
//! - Scroll: Zoom in/out
//! - Space: Pause/resume simulation
//! - G: Toggle spacetime grid
//! - 1/2/3: Load different presets
//! - R: Reset simulation

mod physics;
mod renderer;

use common::{Camera3D, GraphicsContext};
use physics::Simulation;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "3D Gravity Simulation - Rust/wgpu",
        1280,
        720,
    ));

    let camera = Camera3D::new(ctx.aspect_ratio());
    let mut simulation = Simulation::new();
    simulation.init_disk(200);

    let mut paused = false;
    let mut camera = camera;
    camera.distance = 30.0;
    camera.update_orbital();

    let mut mouse_pressed = false;
    let mut last_mouse_pos: Option<(f64, f64)> = None;
    let mut last_time = std::time::Instant::now();

    // For 3D, we'd need a separate 3D renderer
    // This is a placeholder that demonstrates the structure
    println!("3D Gravity Simulation");
    println!("Note: Full 3D rendering requires additional shader work.");
    println!("Running physics simulation in background...");
    println!();
    println!("Controls:");
    println!("  Space - Pause/Resume");
    println!("  1/2/3 - Load presets");
    println!("  ESC - Exit");

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            camera.update_aspect_ratio(size.width as f32 / size.height as f32);
                        }
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match key {
                        KeyCode::Escape => elwt.exit(),
                        KeyCode::Space => {
                            paused = !paused;
                            println!("Simulation {}", if paused { "paused" } else { "running" });
                        }
                        KeyCode::Digit1 => {
                            simulation.init_solar_system();
                            println!("Loaded: Solar System ({} bodies)", simulation.bodies.len());
                        }
                        KeyCode::Digit2 => {
                            simulation.init_disk(300);
                            println!("Loaded: Accretion Disk ({} bodies)", simulation.bodies.len());
                        }
                        KeyCode::Digit3 => {
                            simulation.init_galaxy_collision(200);
                            println!(
                                "Loaded: Galaxy Collision ({} bodies)",
                                simulation.bodies.len()
                            );
                        }
                        _ => {}
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            mouse_pressed = state == ElementState::Pressed;
                            if !mouse_pressed {
                                last_mouse_pos = None;
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if mouse_pressed {
                            if let Some((last_x, last_y)) = last_mouse_pos {
                                let dx = (position.x - last_x) as f32 * 0.01;
                                let dy = (position.y - last_y) as f32 * 0.01;
                                camera.orbit(dx, dy);
                            }
                            last_mouse_pos = Some((position.x, position.y));
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let scroll = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                        };
                        camera.zoom(scroll * 2.0);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_time).as_secs_f32().min(0.1);
                        last_time = now;

                        if !paused {
                            let substeps = 4;
                            let sub_dt = dt / substeps as f32;
                            for _ in 0..substeps {
                                simulation.step(sub_dt);
                            }
                        }

                        // In a full implementation, we'd render the 3D scene here
                        // For now, just request another frame
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    ctx.window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
