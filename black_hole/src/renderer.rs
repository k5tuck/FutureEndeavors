//! Rendering system for black hole simulation

use common::{Camera2D, CameraUniform, GraphicsContext};
use glam::Vec2;
use wgpu::util::DeviceExt;

use crate::physics::{BlackHole, LightRay2D};

/// Uniform data for black hole
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlackHoleUniform {
    pub position: [f32; 4],
    pub mass: f32,
    pub schwarzschild_radius: f32,
    pub time: f32,
    pub _padding: f32,
}

impl BlackHoleUniform {
    pub fn from_black_hole(bh: &BlackHole, time: f32) -> Self {
        Self {
            position: [bh.position.x, bh.position.y, bh.position.z, 1.0],
            mass: bh.mass,
            schwarzschild_radius: bh.schwarzschild_radius,
            time,
            _padding: 0.0,
        }
    }
}

/// Line vertex for ray path visualization
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl LineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// 2D renderer for gravitational lensing visualization
pub struct Renderer2D {
    line_pipeline: wgpu::RenderPipeline,
    circle_pipeline: wgpu::RenderPipeline,
    line_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    black_hole_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    black_hole_bind_group: wgpu::BindGroup,
    max_vertices: usize,
}

impl Renderer2D {
    pub fn new(ctx: &GraphicsContext, max_vertices: usize) -> Self {
        let device = &ctx.device;

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Black Hole Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/black_hole.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Black hole uniform buffer
        let black_hole_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Black Hole Buffer"),
            size: std::mem::size_of::<BlackHoleUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layouts
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let black_hole_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Black Hole Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let black_hole_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Black Hole Bind Group"),
            layout: &black_hole_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: black_hole_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &black_hole_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Line render pipeline
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_line",
                buffers: &[LineVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_line",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Circle pipeline (for event horizon)
        let circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Circle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_circle",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_circle",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Line vertex buffer
        let line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer"),
            size: (std::mem::size_of::<LineVertex>() * max_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            line_pipeline,
            circle_pipeline,
            line_buffer,
            camera_buffer,
            black_hole_buffer,
            camera_bind_group,
            black_hole_bind_group,
            max_vertices,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera2D) {
        let uniform = CameraUniform::from_camera_2d(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_black_hole(&self, queue: &wgpu::Queue, bh: &BlackHole, time: f32) {
        let uniform = BlackHoleUniform::from_black_hole(bh, time);
        queue.write_buffer(
            &self.black_hole_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    /// Convert light rays to vertex data
    pub fn update_rays(&self, queue: &wgpu::Queue, rays: &[LightRay2D]) -> Vec<(u32, u32)> {
        let mut vertices = Vec::new();
        let mut ranges = Vec::new();

        for ray in rays {
            let start = vertices.len() as u32;

            // Color gradient along the ray
            let path_len = ray.path.len();
            for (i, pos) in ray.path.iter().enumerate() {
                let t = i as f32 / path_len.max(1) as f32;
                let color = [
                    1.0 - t * 0.3, // R: bright to slightly dimmer
                    0.8 - t * 0.5, // G: yellow to orange
                    0.2 + t * 0.3, // B: slight blue shift at end
                    1.0 - t * 0.5, // A: fade out
                ];

                vertices.push(LineVertex {
                    position: [pos.x, pos.y],
                    color,
                });
            }

            ranges.push((start, vertices.len() as u32 - start));
        }

        if vertices.len() > self.max_vertices {
            vertices.truncate(self.max_vertices);
        }

        queue.write_buffer(&self.line_buffer, 0, bytemuck::cast_slice(&vertices));

        ranges
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        ray_ranges: &[(u32, u32)],
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.02,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw event horizon and photon sphere
        render_pass.set_pipeline(&self.circle_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.black_hole_bind_group, &[]);
        render_pass.draw(0..65, 0..2); // 64 segments + closing vertex, 2 instances

        // Draw light rays
        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_vertex_buffer(0, self.line_buffer.slice(..));

        for (start, count) in ray_ranges {
            if *count > 1 {
                render_pass.draw(*start..(*start + *count), 0..1);
            }
        }
    }
}

/// Full-screen ray marching renderer for 3D visualization
pub struct Renderer3D {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    black_hole_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Renderer3D {
    pub fn new(ctx: &GraphicsContext) -> Self {
        let device = &ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Black Hole 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/black_hole.wgsl").into()),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let black_hole_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Black Hole Buffer"),
            size: std::mem::size_of::<BlackHoleUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: black_hole_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_fullscreen",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            camera_buffer,
            black_hole_buffer,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, camera_pos: [f32; 3], bh: &BlackHole, time: f32) {
        let camera_uniform = CameraUniform {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            position: [camera_pos[0], camera_pos[1], camera_pos[2], 1.0],
        };
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let bh_uniform = BlackHoleUniform::from_black_hole(bh, time);
        queue.write_buffer(
            &self.black_hole_buffer,
            0,
            bytemuck::cast_slice(&[bh_uniform]),
        );
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("3D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Full-screen triangle
    }
}
