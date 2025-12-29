//! Rendering system for atom simulation

use common::{Camera2D, CameraUniform, GraphicsContext};
use wgpu::util::DeviceExt;

use crate::physics::{Atom, Bond, Simulation};

/// Instance data for GPU rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtomInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl AtomInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x3,
        3 => Float32,
        4 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<AtomInstance>() as wgpu::BufferAddress,
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

/// Bond line vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BondVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl BondVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BondVertex>() as wgpu::BufferAddress,
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
    atom_pipeline: wgpu::RenderPipeline,
    bond_pipeline: wgpu::RenderPipeline,
    grid_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    atom_buffer: wgpu::Buffer,
    bond_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    max_atoms: usize,
    max_bond_vertices: usize,
}

impl Renderer {
    pub fn new(ctx: &GraphicsContext, max_atoms: usize) -> Self {
        let device = &ctx.device;
        let max_bond_vertices = max_atoms * 4; // 2 vertices per bond, estimate 2 bonds per atom

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Atom Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/atom.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Camera bind group layout
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

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Atom render pipeline
        let atom_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Atom Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_atom"),
                buffers: &[QuadVertex::layout(), AtomInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_atom"),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Bond render pipeline
        let bond_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bond Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_bond"),
                buffers: &[BondVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_bond"),
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
            depth_stencil: None,
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
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_grid"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
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

        // Atom instance buffer
        let atom_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Atom Buffer"),
            size: (std::mem::size_of::<AtomInstance>() * max_atoms) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bond vertex buffer
        let bond_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bond Buffer"),
            size: (std::mem::size_of::<BondVertex>() * max_bond_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            atom_pipeline,
            bond_pipeline,
            grid_pipeline,
            quad_buffer,
            atom_buffer,
            bond_buffer,
            camera_buffer,
            camera_bind_group,
            max_atoms,
            max_bond_vertices,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera2D) {
        let uniform = CameraUniform::from_camera_2d(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_simulation(&self, queue: &wgpu::Queue, sim: &Simulation) -> (u32, u32) {
        // Update atom instances
        let atom_instances: Vec<AtomInstance> = sim
            .atoms
            .iter()
            .take(self.max_atoms)
            .map(|atom| AtomInstance {
                position: [atom.position.x, atom.position.y, 0.0],
                radius: atom.radius(),
                color: atom.color(),
            })
            .collect();

        queue.write_buffer(&self.atom_buffer, 0, bytemuck::cast_slice(&atom_instances));

        // Update bond vertices
        let mut bond_vertices: Vec<BondVertex> = Vec::new();
        let bond_color = [0.5, 0.5, 0.5, 0.8];

        for bond in &sim.bonds {
            if bond.atom_a >= sim.atoms.len() || bond.atom_b >= sim.atoms.len() {
                continue;
            }

            let a = &sim.atoms[bond.atom_a];
            let b = &sim.atoms[bond.atom_b];

            bond_vertices.push(BondVertex {
                position: [a.position.x, a.position.y],
                color: bond_color,
            });
            bond_vertices.push(BondVertex {
                position: [b.position.x, b.position.y],
                color: bond_color,
            });

            if bond_vertices.len() >= self.max_bond_vertices {
                break;
            }
        }

        queue.write_buffer(&self.bond_buffer, 0, bytemuck::cast_slice(&bond_vertices));

        (atom_instances.len() as u32, bond_vertices.len() as u32)
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_atoms: u32,
        num_bond_vertices: u32,
        show_grid: bool,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.08,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw grid
        if show_grid {
            render_pass.set_pipeline(&self.grid_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw(0..40, 0..1); // 10 horizontal + 10 vertical lines * 2 vertices
        }

        // Draw bonds
        if num_bond_vertices > 0 {
            render_pass.set_pipeline(&self.bond_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.bond_buffer.slice(..));
            render_pass.draw(0..num_bond_vertices, 0..1);
        }

        // Draw atoms
        if num_atoms > 0 {
            render_pass.set_pipeline(&self.atom_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.atom_buffer.slice(..));
            render_pass.draw(0..6, 0..num_atoms);
        }
    }
}
