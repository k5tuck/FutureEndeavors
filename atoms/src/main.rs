//! Atomic and Molecular Dynamics Simulation
//!
//! A real-time simulation of atoms and molecules featuring:
//! - Coulomb forces for charged particles
//! - Lennard-Jones potential for van der Waals interactions
//! - Dynamic bond formation
//! - Multiple presets (water, salt, organic molecules)
//!
//! Controls:
//! - Click: Add atoms at cursor position
//! - Scroll: Zoom in/out
//! - Arrow keys / WASD: Pan camera
//! - 1/2/3/4: Load presets (Water, Salt, Organic, Random)
//! - Space: Pause/resume simulation
//! - G: Toggle grid
//! - R: Reset simulation
//! - T: Increase temperature
//! - Shift+T: Decrease temperature

mod physics;
mod renderer;
mod equations_ui;

use common::{Camera2D, GraphicsContext};
use glam::Vec2;
use physics::{Element, Simulation};
use renderer::Renderer;
use equations_ui::{draw_equations_sidebar, ATOMS_EQUATIONS, ATOMS_VARIABLES};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

const MAX_ATOMS: usize = 1000;

struct EguiState {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

struct App {
    ctx: GraphicsContext,
    renderer: Renderer,
    simulation: Simulation,
    camera: Camera2D,
    paused: bool,
    show_grid: bool,
    current_element: Element,
    modifiers: ModifiersState,
    egui: EguiState,
}

impl App {
    fn new(ctx: GraphicsContext) -> Self {
        let renderer = Renderer::new(&ctx, MAX_ATOMS);
        let mut camera = Camera2D::new(ctx.aspect_ratio());
        camera.zoom = 15.0;

        let mut simulation = Simulation::new();
        simulation.init_water(10);

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
            show_grid: true,
            current_element: Element::Hydrogen,
            modifiers: ModifiersState::empty(),
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
            // Substep for stability
            let substeps = 4;
            let sub_dt = dt / substeps as f32;
            for _ in 0..substeps {
                self.simulation.step(sub_dt);
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update GPU buffers
        self.renderer.update_camera(&self.ctx.queue, &self.camera);
        let (num_atoms, num_bonds) = self.renderer.update_simulation(&self.ctx.queue, &self.simulation);

        // Build egui UI
        let raw_input = self.egui.state.take_egui_input(&self.ctx.window);
        let full_output = self.egui.ctx.run(raw_input, |ctx| {
            draw_equations_sidebar(
                ctx,
                "Molecular Dynamics",
                ATOMS_EQUATIONS,
                ATOMS_VARIABLES,
            );

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Atoms: {}", self.simulation.atoms.len()));
                    ui.separator();
                    ui.label(format!("Bonds: {}", self.simulation.bonds.len()));
                    ui.separator();
                    ui.label(format!("Damping: {:.2}", self.simulation.damping));
                    ui.separator();
                    ui.label(format!("Element: {:?}", self.current_element));
                    ui.separator();
                    if self.paused {
                        ui.label(egui::RichText::new("PAUSED").color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("RUNNING").color(egui::Color32::GREEN));
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

        self.renderer.render(&mut encoder, &view, num_atoms, num_bonds, self.show_grid);

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

        let world_x = self.camera.position.x + normalized_x * self.camera.zoom * self.camera.aspect_ratio;
        let world_y = self.camera.position.y + normalized_y * self.camera.zoom;

        self.simulation.add_atom(self.current_element, Vec2::new(world_x, world_y));
    }

    fn handle_key(&mut self, key: KeyCode, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match key {
            KeyCode::Space => self.paused = !self.paused,
            KeyCode::KeyG => self.show_grid = !self.show_grid,
            KeyCode::KeyR => {
                self.simulation.clear();
            }
            KeyCode::Digit1 => {
                self.simulation.init_water(10);
            }
            KeyCode::Digit2 => {
                self.simulation.init_salt(15);
            }
            KeyCode::Digit3 => {
                self.simulation.init_organic(8);
            }
            KeyCode::Digit4 => {
                self.simulation.init_random(50);
            }
            KeyCode::KeyH => self.current_element = Element::Hydrogen,
            KeyCode::KeyC => self.current_element = Element::Carbon,
            KeyCode::KeyN => self.current_element = Element::Nitrogen,
            KeyCode::KeyO => self.current_element = Element::Oxygen,
            KeyCode::KeyT => {
                if self.modifiers.shift_key() {
                    self.simulation.damping = (self.simulation.damping + 0.01).min(1.0);
                } else {
                    self.simulation.damping = (self.simulation.damping - 0.01).max(0.9);
                }
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
        self.camera.zoom = self.camera.zoom.clamp(5.0, 50.0);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.state.on_window_event(&self.ctx.window, event).consumed
    }
}

fn main() {
    let (ctx, event_loop) = pollster::block_on(GraphicsContext::new(
        "Atoms - Molecular Dynamics",
        1280,
        720,
    ));

    let mut app = App::new(ctx);
    let mut last_time = std::time::Instant::now();
    let mut mouse_pos = (0.0, 0.0);

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
                            WindowEvent::ModifiersChanged(modifiers) => {
                                app.modifiers = modifiers.state();
                            }
                            WindowEvent::MouseInput {
                                state: ElementState::Pressed,
                                button: MouseButton::Left,
                                ..
                            } => {
                                app.handle_click(mouse_pos.0, mouse_pos.1);
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                mouse_pos = (position.x, position.y);
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
