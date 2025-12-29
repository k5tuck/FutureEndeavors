//! 3D Rendering system for gravity simulation

use common::{Camera3D, GraphicsContext};
use wgpu::util::DeviceExt;

use crate::physics_3d::{Body3D, Simulation3D};

/// Camera uniform with view matrix for billboarding
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform3D {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub position: [f32; 4],
}

impl CameraUniform3D {
    pub fn from_camera(camera: &Camera3D) -> Self {
        Self {
            view_proj: camera.view_projection().to_cols_array_2d(),
            view: camera.view_matrix().to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 1.0],
        }
    }
}

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

/// Trail vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TrailVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl TrailVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TrailVertex>() as wgpu::BufferAddress,
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

pub struct Renderer3D {
    particle_pipeline: wgpu::RenderPipeline,
    trail_pipeline: wgpu::RenderPipeline,
    grid_pipeline: wgpu::RenderPipeline,
    skybox_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    trail_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::TextureView,
    max_instances: usize,
    max_trail_vertices: usize,
}

impl Renderer3D {
    pub fn new(ctx: &GraphicsContext, max_instances: usize) -> Self {
        let device = &ctx.device;
        let max_trail_vertices = max_instances * 300;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle_3d.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform3D>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Depth texture
        let depth_texture = Self::create_depth_texture(device, ctx.size.width, ctx.size.height);

        let depth_stencil_state = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        // Particle pipeline
        let particle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_particle_3d",
                buffers: &[QuadVertex::layout(), ParticleInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_particle_3d",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Trail pipeline
        let trail_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Trail Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_trail",
                buffers: &[TrailVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_trail",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Grid pipeline
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_grid",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_grid",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Skybox pipeline
        let skybox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_skybox",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_skybox",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Buffers
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<ParticleInstance>() * max_instances) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let trail_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Trail Buffer"),
            size: (std::mem::size_of::<TrailVertex>() * max_trail_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            particle_pipeline,
            trail_pipeline,
            grid_pipeline,
            skybox_pipeline,
            quad_buffer,
            instance_buffer,
            trail_buffer,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            max_instances,
            max_trail_vertices,
        }
    }

    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_texture = Self::create_depth_texture(device, width, height);
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera3D) {
        let uniform = CameraUniform3D::from_camera(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_simulation(
        &self,
        queue: &wgpu::Queue,
        sim: &Simulation3D,
    ) -> (u32, Vec<(u32, u32)>) {
        // Update particle instances
        let instances: Vec<ParticleInstance> = sim
            .bodies
            .iter()
            .take(self.max_instances)
            .map(|body| ParticleInstance {
                position: [body.position.x, body.position.y, body.position.z],
                radius: body.radius,
                color: body.color,
            })
            .collect();

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        // Update trails
        let mut trail_vertices = Vec::new();
        let mut trail_ranges = Vec::new();

        for body in &sim.bodies {
            if body.trail.len() < 2 {
                continue;
            }

            let start = trail_vertices.len() as u32;
            let trail_len = body.trail.len();

            for (i, pos) in body.trail.iter().enumerate() {
                let alpha = (i as f32 / trail_len as f32) * 0.6;
                trail_vertices.push(TrailVertex {
                    position: [pos.x, pos.y, pos.z],
                    color: [body.color[0], body.color[1], body.color[2], alpha],
                });
            }

            let count = trail_vertices.len() as u32 - start;
            if count > 1 {
                trail_ranges.push((start, count));
            }

            if trail_vertices.len() >= self.max_trail_vertices {
                break;
            }
        }

        if !trail_vertices.is_empty() {
            queue.write_buffer(&self.trail_buffer, 0, bytemuck::cast_slice(&trail_vertices));
        }

        (instances.len() as u32, trail_ranges)
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_instances: u32,
        trail_ranges: &[(u32, u32)],
        show_grid: bool,
        show_trails: bool,
    ) {
        // First pass: skybox (no depth)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Skybox Pass"),
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

            render_pass.set_pipeline(&self.skybox_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Second pass: 3D scene with depth
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main 3D Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Grid
            if show_grid {
                render_pass.set_pipeline(&self.grid_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.draw(0..84, 0..1); // 21 * 2 * 2 lines
            }

            // Trails
            if show_trails && !trail_ranges.is_empty() {
                render_pass.set_pipeline(&self.trail_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.trail_buffer.slice(..));

                for (start, count) in trail_ranges {
                    render_pass.draw(*start..(*start + *count), 0..1);
                }
            }

            // Particles
            if num_instances > 0 {
                render_pass.set_pipeline(&self.particle_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.draw(0..6, 0..num_instances);
            }
        }
    }
}
