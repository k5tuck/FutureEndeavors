//! Spacetime curvature visualization
//!
//! Visualizes how mass curves spacetime using a deformable grid

use glam::Vec3;

use crate::solar_system::{CelestialBody, BodyType, G, C};

/// A vertex in the spacetime grid
#[derive(Debug, Clone, Copy)]
pub struct GridVertex {
    pub rest_position: Vec3,   // Position without curvature
    pub curved_position: Vec3, // Position with gravitational curvature
    pub curvature: f32,        // Local curvature intensity
}

/// Spacetime grid for visualization
pub struct SpacetimeGrid {
    pub vertices: Vec<GridVertex>,
    pub grid_size: usize,
    pub extent: f32,
    pub deformation_scale: f32,
}

impl SpacetimeGrid {
    pub fn new(grid_size: usize, extent: f32) -> Self {
        let mut vertices = Vec::new();

        let step = extent * 2.0 / grid_size as f32;
        for i in 0..=grid_size {
            for j in 0..=grid_size {
                let x = -extent + i as f32 * step;
                let z = -extent + j as f32 * step;
                let pos = Vec3::new(x, 0.0, z);

                vertices.push(GridVertex {
                    rest_position: pos,
                    curved_position: pos,
                    curvature: 0.0,
                });
            }
        }

        Self {
            vertices,
            grid_size,
            extent,
            deformation_scale: 1.0,
        }
    }

    /// Update grid deformation based on gravitating bodies
    pub fn update(&mut self, bodies: &[CelestialBody]) {
        let c_squared = C * C;

        for vertex in &mut self.vertices {
            let mut total_potential = 0.0;
            let mut max_potential = 0.0;
            let pos_2d = Vec3::new(vertex.rest_position.x, 0.0, vertex.rest_position.z);

            for body in bodies {
                // Only deform for significant masses
                if body.mass < 1e-8 && body.body_type != BodyType::BlackHole {
                    continue;
                }

                let body_pos_2d = Vec3::new(body.position.x, 0.0, body.position.z);
                let r = (pos_2d - body_pos_2d).length();

                if r > 0.001 {
                    // Gravitational potential: φ = -GM/r
                    let potential = G * body.mass / r;
                    total_potential += potential;

                    // Enhanced potential near black holes
                    if body.body_type == BodyType::BlackHole {
                        let rs = body.schwarzschild_radius();
                        if r < rs * 3.0 {
                            let enhancement = (rs * 3.0 / r).powi(2);
                            max_potential = max_potential.max(potential * enhancement);
                        }
                    } else {
                        max_potential = max_potential.max(potential);
                    }
                }
            }

            // Deformation: y = -potential * scale
            // This creates the "rubber sheet" visualization
            let deformation = -total_potential * self.deformation_scale;

            vertex.curved_position = Vec3::new(
                vertex.rest_position.x,
                deformation.max(-self.extent * 0.5), // Clamp to prevent extreme depths
                vertex.rest_position.z,
            );

            // Curvature intensity for coloring
            vertex.curvature = (max_potential / (c_squared * 0.01)).min(1.0);
        }
    }

    /// Get the grid as line vertices for rendering
    pub fn get_line_vertices(&self) -> Vec<(Vec3, [f32; 4])> {
        let mut lines = Vec::new();
        let n = self.grid_size + 1;

        // Horizontal lines
        for i in 0..n {
            for j in 0..self.grid_size {
                let idx1 = i * n + j;
                let idx2 = i * n + j + 1;

                let v1 = &self.vertices[idx1];
                let v2 = &self.vertices[idx2];

                let color1 = curvature_color(v1.curvature);
                let color2 = curvature_color(v2.curvature);

                lines.push((v1.curved_position, color1));
                lines.push((v2.curved_position, color2));
            }
        }

        // Vertical lines
        for i in 0..self.grid_size {
            for j in 0..n {
                let idx1 = i * n + j;
                let idx2 = (i + 1) * n + j;

                let v1 = &self.vertices[idx1];
                let v2 = &self.vertices[idx2];

                let color1 = curvature_color(v1.curvature);
                let color2 = curvature_color(v2.curvature);

                lines.push((v1.curved_position, color1));
                lines.push((v2.curved_position, color2));
            }
        }

        lines
    }

    /// Get the deformation at a specific point (for spaceship effects)
    pub fn sample_curvature(&self, position: Vec3) -> f32 {
        // Find nearest grid vertex
        let x = ((position.x + self.extent) / (self.extent * 2.0) * self.grid_size as f32) as usize;
        let z = ((position.z + self.extent) / (self.extent * 2.0) * self.grid_size as f32) as usize;

        let x = x.min(self.grid_size);
        let z = z.min(self.grid_size);

        let idx = x * (self.grid_size + 1) + z;
        if idx < self.vertices.len() {
            self.vertices[idx].curvature
        } else {
            0.0
        }
    }
}

/// Convert curvature intensity to color
fn curvature_color(curvature: f32) -> [f32; 4] {
    // Blue (low curvature) -> Green -> Yellow -> Red (high curvature)
    let t = curvature.clamp(0.0, 1.0);

    let r = if t < 0.5 { t * 2.0 } else { 1.0 };
    let g = if t < 0.5 { 0.5 + t } else { 1.0 - (t - 0.5) * 2.0 };
    let b = 1.0 - t;

    [r * 0.7, g * 0.7, b * 0.7, 0.6]
}

/// Gravitational lensing effect
pub struct LensingEffect {
    pub rays: Vec<LensedRay>,
}

#[derive(Debug, Clone)]
pub struct LensedRay {
    pub path: Vec<Vec3>,
    pub color: [f32; 4],
}

impl LensingEffect {
    pub fn new() -> Self {
        Self { rays: Vec::new() }
    }

    /// Cast rays around a black hole to show lensing
    pub fn trace_around_black_hole(
        &mut self,
        black_hole: &CelestialBody,
        observer_pos: Vec3,
        ray_count: usize,
        steps: usize,
    ) {
        self.rays.clear();

        if black_hole.body_type != BodyType::BlackHole {
            return;
        }

        let rs = black_hole.schwarzschild_radius();
        let bh_pos = black_hole.position;

        // Generate rays in a fan pattern
        for i in 0..ray_count {
            let angle = (i as f32 / ray_count as f32) * std::f32::consts::TAU;

            // Start point: ring around black hole
            let start_radius = rs * 5.0;
            let start = bh_pos + Vec3::new(
                angle.cos() * start_radius,
                0.0,
                angle.sin() * start_radius,
            );

            // Initial direction: toward observer with slight offset
            let to_observer = (observer_pos - start).normalize();

            let mut ray = LensedRay {
                path: Vec::new(),
                color: [1.0, 0.8, 0.3, 0.5],
            };

            // Trace the ray
            let mut pos = start;
            let mut dir = to_observer;
            let step_size = rs * 0.1;

            for _ in 0..steps {
                ray.path.push(pos);

                let r_vec = pos - bh_pos;
                let r = r_vec.length();

                // Inside event horizon
                if r < rs * 1.1 {
                    ray.color = [0.5, 0.0, 0.0, 0.3]; // Red for captured
                    break;
                }

                // Gravitational deflection
                let r_hat = r_vec / r;
                let deflection_strength = rs / (r * r) * 2.0;
                let deflection = -r_hat * deflection_strength * step_size;

                dir = (dir + deflection).normalize();
                pos += dir * step_size;

                // Stop if far from black hole
                if r > rs * 20.0 {
                    break;
                }
            }

            if ray.path.len() > 1 {
                self.rays.push(ray);
            }
        }
    }
}

impl Default for LensingEffect {
    fn default() -> Self {
        Self::new()
    }
}
