//! Quantum simulation rendering system
//!
//! Provides GPU-accelerated rendering for various quantum phenomena

use common::{Camera2D, Camera3D, CameraUniform, GraphicsContext};
use wgpu::util::DeviceExt;
use glam::Vec3;

/// Instance data for probability cloud points
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointInstance {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
}

impl PointInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x3,  // position
        3 => Float32,    // size
        4 => Float32x4,  // color
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PointInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Quad vertex for billboards
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
}

impl QuadVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, 1.0] },
];

/// Line vertex for connections and edges
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl LineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
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

/// Wavefunction rendering data (for 1D plots)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WaveVertex {
    pub position: [f32; 2], // x, y in screen space
    pub value: f32,         // Wavefunction value (for coloring)
    pub phase: f32,         // Phase angle
}

impl WaveVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32,
        2 => Float32,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WaveVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// General quantum renderer supporting multiple visualization modes
pub struct QuantumRenderer {
    // Point cloud pipeline (for orbitals, electrons)
    point_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    point_buffer: wgpu::Buffer,
    max_points: usize,

    // Line pipeline (for edges, flux tubes)
    line_pipeline: wgpu::RenderPipeline,
    line_buffer: wgpu::Buffer,
    max_lines: usize,

    // Camera
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl QuantumRenderer {
    pub fn new(ctx: &GraphicsContext, max_points: usize, max_lines: usize) -> Self {
        let device = &ctx.device;

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quantum Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quantum.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quantum Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Point cloud pipeline
        let point_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Point Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_point",
                buffers: &[QuadVertex::layout(), PointInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_point",
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

        // Line pipeline
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
                topology: wgpu::PrimitiveTopology::LineList,
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

        // Vertex buffers
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let point_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Point Instance Buffer"),
            size: (std::mem::size_of::<PointInstance>() * max_points) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer"),
            size: (std::mem::size_of::<LineVertex>() * max_lines * 2) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            point_pipeline,
            quad_buffer,
            point_buffer,
            max_points,
            line_pipeline,
            line_buffer,
            max_lines,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn update_camera_3d(&self, queue: &wgpu::Queue, camera: &Camera3D) {
        let uniform = CameraUniform::from_camera_3d(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_camera_2d(&self, queue: &wgpu::Queue, camera: &Camera2D) {
        let uniform = CameraUniform::from_camera_2d(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_points(&self, queue: &wgpu::Queue, points: &[PointInstance]) {
        let data = &points[..points.len().min(self.max_points)];
        queue.write_buffer(&self.point_buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn update_lines(&self, queue: &wgpu::Queue, lines: &[(Vec3, Vec3, [f32; 4])]) {
        let vertices: Vec<LineVertex> = lines
            .iter()
            .take(self.max_lines)
            .flat_map(|(v1, v2, color)| {
                [
                    LineVertex {
                        position: [v1.x, v1.y, v1.z],
                        color: *color,
                    },
                    LineVertex {
                        position: [v2.x, v2.y, v2.z],
                        color: *color,
                    },
                ]
            })
            .collect();

        queue.write_buffer(&self.line_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    pub fn render_points(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_points: u32,
        clear: bool,
    ) {
        let load_op = if clear {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.02,
                g: 0.02,
                b: 0.08,
                a: 1.0,
            })
        } else {
            wgpu::LoadOp::Load
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Point Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.point_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.point_buffer.slice(..));
        render_pass.draw(0..6, 0..num_points);
    }

    pub fn render_lines(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_lines: u32,
        clear: bool,
    ) {
        let load_op = if clear {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.02,
                g: 0.02,
                b: 0.08,
                a: 1.0,
            })
        } else {
            wgpu::LoadOp::Load
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Line Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.line_buffer.slice(..));
        render_pass.draw(0..(num_lines * 2), 0..1);
    }
}

/// Convert orbital data to point instances
pub fn orbital_to_points(data: &[(Vec3, f32, [f32; 4])]) -> Vec<PointInstance> {
    data.iter()
        .map(|(pos, prob, color)| PointInstance {
            position: [pos.x, pos.y, pos.z],
            size: 0.05 + prob.sqrt() * 0.1,
            color: *color,
        })
        .collect()
}

/// Convert quark data to point instances
pub fn quarks_to_points(data: &[(Vec3, f32, [f32; 4], String)]) -> Vec<PointInstance> {
    data.iter()
        .map(|(pos, radius, color, _)| PointInstance {
            position: [pos.x, pos.y, pos.z],
            size: *radius,
            color: *color,
        })
        .collect()
}

/// Convert 4D vertices to point instances
pub fn hypercube_to_points(data: &[(Vec3, [f32; 4])]) -> Vec<PointInstance> {
    data.iter()
        .map(|(pos, color)| PointInstance {
            position: [pos.x, pos.y, pos.z],
            size: 0.08,
            color: *color,
        })
        .collect()
}
