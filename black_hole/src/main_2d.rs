//! 2D Gravitational Lensing Visualization
//!
//! Demonstrates how light bends around a black hole using
//! 2D ray tracing. Shows:
//! - Event horizon (black circle)
//! - Photon sphere (where light can orbit)
//! - Light ray paths bending around the black hole
//!
//! Controls:
//! - Click: Spawn light rays from click position
//! - Scroll: Zoom in/out
//! - Arrow keys: Pan camera
//! - R: Reset rays
//! - Space: Toggle continuous ray emission
//! - +/-: Adjust black hole mass

mod physics;
mod renderer;
mod equations_ui;

use common::{Camera2D, GraphicsContext};
use glam::{Vec2, Vec3};
use physics::{BlackHole, LightRay2D};
use renderer::Renderer2D;
use equations_ui::{draw_equations_sidebar, BLACK_HOLE_2D_EQUATIONS, BLACK_HOLE_2D_VARIABLES};
use std::f32::consts::PI;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, PhysicalKey},
};

const MAX_VERTICES: usize = 100000;
const MAX_RAYS: usize = 200;

struct EguiState {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

struct App {
    ctx: GraphicsContext,
    renderer: Renderer2D,
    camera: Camera2D,
    black_hole: BlackHole,
    rays: Vec<LightRay2D>,
    time: f32,
    continuous_emission: bool,
    emission_angle: f32,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer2D::new(&ctx, MAX_VERTICES);
        let mut camera = Camera2D::new(ctx.aspect_ratio());
        camera.zoom = 15.0;

        let black_hole = BlackHole::new(Vec3::ZERO, 1.0);

        // Initial rays from the right side
        let rays = Self::create_parallel_rays(Vec2::new(10.0, 0.0), -PI, 20, 8.0);

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
            camera,
            black_hole,
            rays,
            time: 0.0,
            continuous_emission: false,
            emission_angle: 0.0,
            egui: EguiState {
                ctx: egui_ctx,
                state: egui_state,
                renderer: egui_renderer,
            },
        }
    }

    fn create_parallel_rays(origin: Vec2, angle: f32, count: usize, spread: f32) -> Vec<LightRay2D> {
        let mut rays = Vec::new();
        let direction = Vec2::new(angle.cos(), angle.sin());
        let perpendicular = Vec2::new(-direction.y, direction.x);

        for i in 0..count {
            let t = if count > 1 {
                (i as f32 / (count - 1) as f32) - 0.5
            } else {
                0.0
            };

            let pos = origin + perpendicular * t * spread;
            rays.push(LightRay2D::new(pos, direction));
        }

        rays
    }

    fn create_radial_rays(origin: Vec2, target: Vec2, count: usize, spread: f32) -> Vec<LightRay2D> {
        let mut rays = Vec::new();
        let base_dir = (target - origin).normalize();
        let base_angle = base_dir.y.atan2(base_dir.x);

        for i in 0..count {
            let t = if count > 1 {
                (i as f32 / (count - 1) as f32) - 0.5
            } else {
                0.0
            };

            let angle = base_angle + t * spread;
            let direction = Vec2::new(angle.cos(), angle.sin());
            rays.push(LightRay2D::new(origin, direction));
        }

        rays
    }

    fn trace_all_rays(&mut self) {
        let bh_pos = Vec2::new(self.black_hole.position.x, self.black_hole.position.y);

        for ray in &mut self.rays {
            ray.trace(bh_pos, self.black_hole.mass, 2000, 0.05);
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.camera.update_aspect_ratio(self.ctx.aspect_ratio());
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;

        if self.continuous_emission {
            self.emission_angle += dt * 0.5;

            // Emit rays from a rotating source
            let source = Vec2::new(
                self.emission_angle.cos() * 12.0,
                self.emission_angle.sin() * 12.0,
            );

            let mut new_ray = LightRay2D::new(source, -source.normalize());
            let bh_pos = Vec2::new(self.black_hole.position.x, self.black_hole.position.y);
            new_ray.trace(bh_pos, self.black_hole.mass, 2000, 0.05);

            self.rays.push(new_ray);

            // Limit total rays
            if self.rays.len() > MAX_RAYS {
                self.rays.remove(0);
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.update_camera(&self.ctx.queue, &self.camera);
        self.renderer
            .update_black_hole(&self.ctx.queue, &self.black_hole, self.time);

        let ray_ranges = self.renderer.update_rays(&self.ctx.queue, &self.rays);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let schwarzschild_radius = 2.0 * self.black_hole.mass;
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "Gravitational Lensing",
                BLACK_HOLE_2D_EQUATIONS,
                BLACK_HOLE_2D_VARIABLES,
            );

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Mass: {:.2}", self.black_hole.mass));
                    ui.separator();
                    ui.label(format!("rₛ: {:.2}", schwarzschild_radius));
                    ui.separator();
                    ui.label(format!("Rays: {}", self.rays.len()));
                    ui.separator();
                    if self.continuous_emission {
                        ui.label(egui::RichText::new("EMITTING").color(egui::Color32::GREEN));
                    } else {
                        ui.label("Click to emit rays");
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

        self.renderer.render(&mut encoder, &view, &ray_ranges);

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

    fn handle_click(&mut self, x: f64, y: f64) {
        // Convert screen coordinates to world coordinates
        let normalized_x = (x as f32 / self.ctx.size.width as f32) * 2.0 - 1.0;
        let normalized_y = 1.0 - (y as f32 / self.ctx.size.height as f32) * 2.0;

        let world_x =
            self.camera.position.x + normalized_x * self.camera.zoom * self.camera.aspect_ratio;
        let world_y = self.camera.position.y + normalized_y * self.camera.zoom;

        let click_pos = Vec2::new(world_x, world_y);

        // Create rays pointing toward the black hole
        let mut new_rays = Self::create_radial_rays(click_pos, Vec2::ZERO, 15, 0.3);

        let bh_pos = Vec2::new(self.black_hole.position.x, self.black_hole.position.y);
        for ray in &mut new_rays {
            ray.trace(bh_pos, self.black_hole.mass, 2000, 0.05);
        }

        self.rays.extend(new_rays);

        // Limit total rays
        while self.rays.len() > MAX_RAYS {
            self.rays.remove(0);
        }
    }

    fn handle_key(&mut self, key: KeyCode, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match key {
            KeyCode::KeyR => {
                // Reset with parallel rays
                self.rays = Self::create_parallel_rays(Vec2::new(10.0, 0.0), -PI, 20, 8.0);
                self.trace_all_rays();
            }
            KeyCode::Space => {
                self.continuous_emission = !self.continuous_emission;
                if self.continuous_emission {
                    self.rays.clear();
                }
            }
            KeyCode::Equal | KeyCode::NumpadAdd => {
                self.black_hole = BlackHole::new(Vec3::ZERO, self.black_hole.mass * 1.2);
                self.rays = Self::create_parallel_rays(Vec2::new(10.0, 0.0), -PI, 20, 8.0);
                self.trace_all_rays();
            }
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                self.black_hole = BlackHole::new(Vec3::ZERO, (self.black_hole.mass / 1.2).max(0.1));
                self.rays = Self::create_parallel_rays(Vec2::new(10.0, 0.0), -PI, 20, 8.0);
                self.trace_all_rays();
            }
            KeyCode::ArrowUp | KeyCode::KeyW => self.camera.position.y += self.camera.zoom * 0.1,
            KeyCode::ArrowDown | KeyCode::KeyS => self.camera.position.y -= self.camera.zoom * 0.1,
            KeyCode::ArrowLeft | KeyCode::KeyA => self.camera.position.x -= self.camera.zoom * 0.1,
            KeyCode::ArrowRight | KeyCode::KeyD => self.camera.position.x += self.camera.zoom * 0.1,
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.camera.zoom *= 1.0 - delta * 0.1;
        self.camera.zoom = self.camera.zoom.clamp(1.0, 50.0);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Black Hole - 2D Gravitational Lensing",
        1280,
        720,
    ));

    let mut app = App::new(ctx);

    // Trace initial rays
    app.trace_all_rays();

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
                            WindowEvent::MouseInput {
                                state: ElementState::Pressed,
                                button: MouseButton::Left,
                                ..
                            } => {
                                // Will be handled with cursor position
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                // Store for click handling
                            }
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
