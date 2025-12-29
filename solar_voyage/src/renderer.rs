//! Rendering system for Solar Voyage simulation

use common::{Camera3D, GraphicsContext};
use wgpu::util::DeviceExt;

use crate::solar_system::{CelestialBody, BodyType, SolarSystem};
use crate::spaceship::Spaceship;
use crate::spacetime::SpacetimeGrid;

/// Camera uniform with view matrix
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub position: [f32; 4],
}

/// Simulation state uniform
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationUniform {
    pub time: f32,
    pub ship_gamma: f32,
    pub ship_speed_c: f32,
    pub curvature_scale: f32,
}

/// Body instance for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BodyInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
    pub body_type: u32,
    pub _padding: [f32; 3],
}

impl BodyInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        2 => Float32x3,
        3 => Float32,
        4 => Float32x4,
        5 => Uint32,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BodyInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn from_body(body: &CelestialBody) -> Self {
        let body_type = match body.body_type {
            BodyType::Star => 0,
            BodyType::Planet | BodyType::DwarfPlanet | BodyType::Moon | BodyType::Asteroid => 1,
            BodyType::BlackHole => 2,
            BodyType::Spaceship => 3,
        };

        Self {
            position: [body.position.x, body.position.y, body.position.z],
            radius: body.display_radius,
            color: body.color,
            body_type,
            _padding: [0.0; 3],
        }
    }
}

/// Quad vertex
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

/// Trail/Grid vertex
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

const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, -1.0] },
    QuadVertex { position: [1.0, 1.0] },
    QuadVertex { position: [-1.0, 1.0] },
];

pub struct Renderer {
    body_pipeline: wgpu::RenderPipeline,
    trail_pipeline: wgpu::RenderPipeline,
    grid_pipeline: wgpu::RenderPipeline,
    skybox_pipeline: wgpu::RenderPipeline,

    quad_buffer: wgpu::Buffer,
    body_buffer: wgpu::Buffer,
    trail_buffer: wgpu::Buffer,
    grid_buffer: wgpu::Buffer,

    camera_buffer: wgpu::Buffer,
    sim_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    depth_texture: wgpu::TextureView,

    max_bodies: usize,
    max_trail_vertices: usize,
    max_grid_vertices: usize,
}

impl Renderer {
    pub fn new(ctx: &GraphicsContext) -> Self {
        let device = &ctx.device;

        let max_bodies = 100;
        let max_trail_vertices = 50000;
        let max_grid_vertices = 50000;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voyage Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/voyage.wgsl").into()),
        });

        // Uniform buffers
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sim_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simulation Buffer"),
            size: std::mem::size_of::<SimulationUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                    resource: sim_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
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

        // Body pipeline
        let body_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Body Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_body"),
                buffers: &[QuadVertex::layout(), BodyInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_body"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Trail pipeline
        let trail_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Trail Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_trail"),
                buffers: &[LineVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_trail"),
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
            cache: None,
        });

        // Grid pipeline
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_grid"),
                buffers: &[LineVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_grid"),
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
            cache: None,
        });

        // Skybox pipeline
        let skybox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_skybox"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_skybox"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Vertex buffers
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let body_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Body Buffer"),
            size: (std::mem::size_of::<BodyInstance>() * max_bodies) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let trail_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Trail Buffer"),
            size: (std::mem::size_of::<LineVertex>() * max_trail_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer"),
            size: (std::mem::size_of::<LineVertex>() * max_grid_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            body_pipeline,
            trail_pipeline,
            grid_pipeline,
            skybox_pipeline,
            quad_buffer,
            body_buffer,
            trail_buffer,
            grid_buffer,
            camera_buffer,
            sim_buffer,
            bind_group,
            depth_texture,
            max_bodies,
            max_trail_vertices,
            max_grid_vertices,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_texture = Self::create_depth_texture(device, width, height);
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera3D,
        solar_system: &SolarSystem,
        spaceship: &Spaceship,
        grid: Option<&SpacetimeGrid>,
    ) -> RenderData {
        // Update camera uniform
        let camera_uniform = CameraUniform {
            view_proj: camera.view_projection().to_cols_array_2d(),
            view: camera.view_matrix().to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 1.0],
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        // Update simulation uniform
        let sim_uniform = SimulationUniform {
            time: solar_system.time,
            ship_gamma: spaceship.gamma,
            ship_speed_c: spaceship.velocity_fraction_c(),
            curvature_scale: 1.0,
        };
        queue.write_buffer(&self.sim_buffer, 0, bytemuck::cast_slice(&[sim_uniform]));

        // Update body instances
        let mut body_instances: Vec<BodyInstance> = solar_system
            .bodies
            .iter()
            .take(self.max_bodies - 1)
            .map(BodyInstance::from_body)
            .collect();

        // Add spaceship as a body
        let ship_body: CelestialBody = spaceship.into();
        body_instances.push(BodyInstance::from_body(&ship_body));

        queue.write_buffer(&self.body_buffer, 0, bytemuck::cast_slice(&body_instances));

        // Update trails
        let mut trail_vertices = Vec::new();
        let mut trail_ranges = Vec::new();

        for body in &solar_system.bodies {
            if body.trail.len() < 2 {
                continue;
            }

            let start = trail_vertices.len() as u32;
            for (i, pos) in body.trail.iter().enumerate() {
                let alpha = (i as f32 / body.trail.len() as f32) * 0.5;
                trail_vertices.push(LineVertex {
                    position: [pos.x, pos.y, pos.z],
                    color: [body.color[0], body.color[1], body.color[2], alpha],
                });
            }
            let count = trail_vertices.len() as u32 - start;
            if count > 1 {
                trail_ranges.push((start, count));
            }
        }

        // Ship trail
        if spaceship.trail.len() > 1 {
            let start = trail_vertices.len() as u32;
            for (i, pos) in spaceship.trail.iter().enumerate() {
                let alpha = (i as f32 / spaceship.trail.len() as f32) * 0.7;
                trail_vertices.push(LineVertex {
                    position: [pos.x, pos.y, pos.z],
                    color: [0.2, 0.8, 0.2, alpha],
                });
            }
            let count = trail_vertices.len() as u32 - start;
            trail_ranges.push((start, count));
        }

        if !trail_vertices.is_empty() && trail_vertices.len() <= self.max_trail_vertices {
            queue.write_buffer(&self.trail_buffer, 0, bytemuck::cast_slice(&trail_vertices));
        }

        // Update spacetime grid
        let mut grid_vertex_count = 0;
        if let Some(grid) = grid {
            let grid_lines = grid.get_line_vertices();
            let grid_vertices: Vec<LineVertex> = grid_lines
                .iter()
                .take(self.max_grid_vertices)
                .map(|(pos, color)| LineVertex {
                    position: [pos.x, pos.y, pos.z],
                    color: *color,
                })
                .collect();

            grid_vertex_count = grid_vertices.len() as u32;
            if !grid_vertices.is_empty() {
                queue.write_buffer(&self.grid_buffer, 0, bytemuck::cast_slice(&grid_vertices));
            }
        }

        RenderData {
            body_count: body_instances.len() as u32,
            trail_ranges,
            grid_vertex_count,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        data: &RenderData,
        show_grid: bool,
        show_trails: bool,
    ) {
        // Skybox pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            pass.set_pipeline(&self.skybox_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // Main 3D pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
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

            // Spacetime grid
            if show_grid && data.grid_vertex_count > 0 {
                pass.set_pipeline(&self.grid_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.grid_buffer.slice(..));
                pass.draw(0..data.grid_vertex_count, 0..1);
            }

            // Trails
            if show_trails && !data.trail_ranges.is_empty() {
                pass.set_pipeline(&self.trail_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.trail_buffer.slice(..));
                for (start, count) in &data.trail_ranges {
                    pass.draw(*start..(*start + *count), 0..1);
                }
            }

            // Bodies
            if data.body_count > 0 {
                pass.set_pipeline(&self.body_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                pass.set_vertex_buffer(1, self.body_buffer.slice(..));
                pass.draw(0..6, 0..data.body_count);
            }
        }
    }
}

pub struct RenderData {
    pub body_count: u32,
    pub trail_ranges: Vec<(u32, u32)>,
    pub grid_vertex_count: u32,
}
