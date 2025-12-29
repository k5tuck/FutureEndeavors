//! Rendering system for gravity simulation

use common::{Camera2D, CameraUniform, GraphicsContext};
use wgpu::util::DeviceExt;

use crate::physics::{Body, Simulation};

/// Instance data for GPU rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl ParticleInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x3,
        3 => Float32,
        4 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Quad vertex for instanced rendering
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

// Unit quad vertices
const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, 1.0] },
];

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    max_instances: usize,
}

impl Renderer {
    pub fn new(ctx: &GraphicsContext, max_instances: usize) -> Self {
        let device = &ctx.device;

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Camera bind group layout
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
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_instanced"),
                buffers: &[QuadVertex::layout(), ParticleInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_instanced"),
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
            cache: None,
        });

        // Quad vertex buffer
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Instance buffer
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<ParticleInstance>() * max_instances) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            quad_buffer,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            max_instances,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera2D) {
        let uniform = CameraUniform::from_camera_2d(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_instances(&self, queue: &wgpu::Queue, bodies: &[Body]) {
        let instances: Vec<ParticleInstance> = bodies
            .iter()
            .take(self.max_instances)
            .map(|body| ParticleInstance {
                position: [body.position.x, body.position.y, 0.0],
                radius: body.radius,
                color: body.color,
            })
            .collect();

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_instances: u32,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..num_instances);
    }
}
