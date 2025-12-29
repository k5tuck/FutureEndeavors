//! 4D Visualization - Tesseract and Hypercube
//!
//! Projects 4-dimensional geometric objects into 3D for visualization.
//! Demonstrates rotation in 4D space and perspective projection.

use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::PI;

/// A point in 4D space
#[derive(Debug, Clone, Copy)]
pub struct Vec4D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4D {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 1e-10 {
            Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
        } else {
            *self
        }
    }

    /// Project to 3D using perspective projection
    pub fn project_to_3d(&self, w_distance: f32) -> Vec3 {
        // Perspective projection from 4D to 3D
        // Similar to 3D→2D but with w as the "depth" axis
        let scale = w_distance / (w_distance - self.w);
        Vec3::new(self.x * scale, self.y * scale, self.z * scale)
    }

    /// Orthographic projection (ignore w)
    pub fn project_orthographic(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl std::ops::Add for Vec4D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl std::ops::Sub for Vec4D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl std::ops::Mul<f32> for Vec4D {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

/// 4x4 rotation matrix for 4D rotations
/// In 4D, there are 6 basic rotation planes: XY, XZ, XW, YZ, YW, ZW
#[derive(Debug, Clone, Copy)]
pub struct Rotation4D {
    pub xy: f32, // Rotation in XY plane
    pub xz: f32, // Rotation in XZ plane
    pub xw: f32, // Rotation in XW plane
    pub yz: f32, // Rotation in YZ plane
    pub yw: f32, // Rotation in YW plane
    pub zw: f32, // Rotation in ZW plane
}

impl Rotation4D {
    pub fn new() -> Self {
        Self {
            xy: 0.0,
            xz: 0.0,
            xw: 0.0,
            yz: 0.0,
            yw: 0.0,
            zw: 0.0,
        }
    }

    /// Apply rotation to a 4D point
    pub fn rotate(&self, p: Vec4D) -> Vec4D {
        let mut result = p;

        // XY rotation
        if self.xy.abs() > 1e-10 {
            let (s, c) = self.xy.sin_cos();
            let x = result.x * c - result.y * s;
            let y = result.x * s + result.y * c;
            result.x = x;
            result.y = y;
        }

        // XZ rotation
        if self.xz.abs() > 1e-10 {
            let (s, c) = self.xz.sin_cos();
            let x = result.x * c - result.z * s;
            let z = result.x * s + result.z * c;
            result.x = x;
            result.z = z;
        }

        // XW rotation
        if self.xw.abs() > 1e-10 {
            let (s, c) = self.xw.sin_cos();
            let x = result.x * c - result.w * s;
            let w = result.x * s + result.w * c;
            result.x = x;
            result.w = w;
        }

        // YZ rotation
        if self.yz.abs() > 1e-10 {
            let (s, c) = self.yz.sin_cos();
            let y = result.y * c - result.z * s;
            let z = result.y * s + result.z * c;
            result.y = y;
            result.z = z;
        }

        // YW rotation
        if self.yw.abs() > 1e-10 {
            let (s, c) = self.yw.sin_cos();
            let y = result.y * c - result.w * s;
            let w = result.y * s + result.w * c;
            result.y = y;
            result.w = w;
        }

        // ZW rotation
        if self.zw.abs() > 1e-10 {
            let (s, c) = self.zw.sin_cos();
            let z = result.z * c - result.w * s;
            let w = result.z * s + result.w * c;
            result.z = z;
            result.w = w;
        }

        result
    }
}

impl Default for Rotation4D {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge connecting two vertices
#[derive(Debug, Clone, Copy)]
pub struct Edge4D {
    pub v1: usize,
    pub v2: usize,
}

/// A 4D polytope (generalization of polyhedron)
pub struct Polytope4D {
    /// Vertices in 4D space
    pub vertices: Vec<Vec4D>,
    /// Edges connecting vertices
    pub edges: Vec<Edge4D>,
    /// Optional: faces (for more complex rendering)
    pub faces: Vec<Vec<usize>>,
    /// Name of the polytope
    pub name: String,
}

impl Polytope4D {
    /// Create a tesseract (4D hypercube)
    pub fn tesseract(size: f32) -> Self {
        let half = size / 2.0;
        let mut vertices = Vec::new();

        // 16 vertices: all combinations of ±half for x, y, z, w
        for &x in &[-half, half] {
            for &y in &[-half, half] {
                for &z in &[-half, half] {
                    for &w in &[-half, half] {
                        vertices.push(Vec4D::new(x, y, z, w));
                    }
                }
            }
        }

        // 32 edges: connect vertices that differ in exactly one coordinate
        let mut edges = Vec::new();
        for i in 0..16 {
            for j in (i + 1)..16 {
                let diff = vertices[i] - vertices[j];
                let diff_count = [diff.x.abs() > 0.1, diff.y.abs() > 0.1,
                                  diff.z.abs() > 0.1, diff.w.abs() > 0.1]
                    .iter()
                    .filter(|&&b| b)
                    .count();
                if diff_count == 1 {
                    edges.push(Edge4D { v1: i, v2: j });
                }
            }
        }

        Self {
            vertices,
            edges,
            faces: Vec::new(),
            name: "Tesseract".to_string(),
        }
    }

    /// Create a 16-cell (4D cross-polytope, dual of tesseract)
    pub fn cell_16(size: f32) -> Self {
        let mut vertices = Vec::new();

        // 8 vertices: ±size on each axis
        for &sign in &[-1.0f32, 1.0] {
            vertices.push(Vec4D::new(sign * size, 0.0, 0.0, 0.0));
            vertices.push(Vec4D::new(0.0, sign * size, 0.0, 0.0));
            vertices.push(Vec4D::new(0.0, 0.0, sign * size, 0.0));
            vertices.push(Vec4D::new(0.0, 0.0, 0.0, sign * size));
        }

        // 24 edges: connect all pairs except antipodal
        let mut edges = Vec::new();
        for i in 0..8 {
            for j in (i + 1)..8 {
                // Skip antipodal pairs
                if (i ^ 1) != j {
                    edges.push(Edge4D { v1: i, v2: j });
                }
            }
        }

        Self {
            vertices,
            edges,
            faces: Vec::new(),
            name: "16-cell".to_string(),
        }
    }

    /// Create a 24-cell (self-dual regular 4D polytope)
    pub fn cell_24(size: f32) -> Self {
        let half = size / 2.0;
        let mut vertices = Vec::new();

        // 24 vertices: permutations of (±1, ±1, 0, 0)
        for &s1 in &[-half, half] {
            for &s2 in &[-half, half] {
                // (±1, ±1, 0, 0) and permutations
                vertices.push(Vec4D::new(s1, s2, 0.0, 0.0));
                vertices.push(Vec4D::new(s1, 0.0, s2, 0.0));
                vertices.push(Vec4D::new(s1, 0.0, 0.0, s2));
                vertices.push(Vec4D::new(0.0, s1, s2, 0.0));
                vertices.push(Vec4D::new(0.0, s1, 0.0, s2));
                vertices.push(Vec4D::new(0.0, 0.0, s1, s2));
            }
        }

        // 96 edges: connect vertices at distance √2
        let mut edges = Vec::new();
        let target_dist_sq = size * size; // Distance √2 * half * √2 = size

        for i in 0..vertices.len() {
            for j in (i + 1)..vertices.len() {
                let d = vertices[i] - vertices[j];
                let dist_sq = d.x * d.x + d.y * d.y + d.z * d.z + d.w * d.w;
                if (dist_sq - target_dist_sq).abs() < 0.1 {
                    edges.push(Edge4D { v1: i, v2: j });
                }
            }
        }

        Self {
            vertices,
            edges,
            faces: Vec::new(),
            name: "24-cell".to_string(),
        }
    }

    /// Create a 4D simplex (5-cell, pentatope)
    pub fn simplex_5(size: f32) -> Self {
        // 5 vertices of a regular 4-simplex
        let a = size;
        let vertices = vec![
            Vec4D::new(a, a, a, -a / (5.0_f32.sqrt())),
            Vec4D::new(a, -a, -a, -a / (5.0_f32.sqrt())),
            Vec4D::new(-a, a, -a, -a / (5.0_f32.sqrt())),
            Vec4D::new(-a, -a, a, -a / (5.0_f32.sqrt())),
            Vec4D::new(0.0, 0.0, 0.0, a * 4.0 / (5.0_f32.sqrt())),
        ];

        // 10 edges: all pairs connected
        let mut edges = Vec::new();
        for i in 0..5 {
            for j in (i + 1)..5 {
                edges.push(Edge4D { v1: i, v2: j });
            }
        }

        Self {
            vertices,
            edges,
            faces: Vec::new(),
            name: "5-cell (Pentatope)".to_string(),
        }
    }
}

/// 4D visualization simulation
pub struct Hypercube4DSimulation {
    /// Current polytope
    pub polytope: Polytope4D,
    /// Current rotation angles
    pub rotation: Rotation4D,
    /// Angular velocities for animation
    pub angular_velocity: Rotation4D,
    /// Projection distance (for perspective)
    pub projection_distance: f32,
    /// Use perspective projection
    pub use_perspective: bool,
    /// Projected 3D vertices
    pub projected_vertices: Vec<Vec3>,
    /// Colors for vertices based on w coordinate
    pub vertex_colors: Vec<[f32; 4]>,
    /// Time
    pub time: f32,
    /// Auto-rotate
    pub auto_rotate: bool,
}

impl Hypercube4DSimulation {
    pub fn new() -> Self {
        let polytope = Polytope4D::tesseract(1.0);
        let n = polytope.vertices.len();

        let mut sim = Self {
            polytope,
            rotation: Rotation4D::new(),
            angular_velocity: Rotation4D {
                xy: 0.0,
                xz: 0.0,
                xw: 0.5, // Rotate in XW plane
                yz: 0.0,
                yw: 0.3, // And YW plane
                zw: 0.0,
            },
            projection_distance: 3.0,
            use_perspective: true,
            projected_vertices: vec![Vec3::ZERO; n],
            vertex_colors: vec![[1.0, 1.0, 1.0, 1.0]; n],
            time: 0.0,
            auto_rotate: true,
        };

        sim.update_projection();
        sim
    }

    /// Set the polytope type
    pub fn set_polytope(&mut self, polytope: Polytope4D) {
        let n = polytope.vertices.len();
        self.polytope = polytope;
        self.projected_vertices = vec![Vec3::ZERO; n];
        self.vertex_colors = vec![[1.0, 1.0, 1.0, 1.0]; n];
        self.update_projection();
    }

    /// Update projected vertices
    fn update_projection(&mut self) {
        let n = self.polytope.vertices.len();
        self.projected_vertices.resize(n, Vec3::ZERO);
        self.vertex_colors.resize(n, [1.0, 1.0, 1.0, 1.0]);

        for i in 0..n {
            // Apply rotation
            let rotated = self.rotation.rotate(self.polytope.vertices[i]);

            // Project to 3D
            self.projected_vertices[i] = if self.use_perspective {
                rotated.project_to_3d(self.projection_distance)
            } else {
                rotated.project_orthographic()
            };

            // Color based on w coordinate (depth in 4D)
            // Objects with different w appear in different colors
            let w_normalized = (rotated.w + 1.0) / 2.0; // Map to [0, 1]
            self.vertex_colors[i] = w_to_color(w_normalized);
        }
    }

    /// Step the simulation
    pub fn step(&mut self, dt: f32) {
        self.time += dt;

        if self.auto_rotate {
            self.rotation.xy += self.angular_velocity.xy * dt;
            self.rotation.xz += self.angular_velocity.xz * dt;
            self.rotation.xw += self.angular_velocity.xw * dt;
            self.rotation.yz += self.angular_velocity.yz * dt;
            self.rotation.yw += self.angular_velocity.yw * dt;
            self.rotation.zw += self.angular_velocity.zw * dt;
        }

        self.update_projection();
    }

    /// Manually rotate
    pub fn rotate_xw(&mut self, angle: f32) {
        self.rotation.xw += angle;
        self.update_projection();
    }

    pub fn rotate_yw(&mut self, angle: f32) {
        self.rotation.yw += angle;
        self.update_projection();
    }

    pub fn rotate_zw(&mut self, angle: f32) {
        self.rotation.zw += angle;
        self.update_projection();
    }

    /// Get edge render data
    pub fn get_edges_3d(&self) -> Vec<(Vec3, Vec3, [f32; 4])> {
        self.polytope
            .edges
            .iter()
            .map(|edge| {
                let v1 = self.projected_vertices[edge.v1];
                let v2 = self.projected_vertices[edge.v2];
                // Blend colors of endpoints
                let c1 = self.vertex_colors[edge.v1];
                let c2 = self.vertex_colors[edge.v2];
                let color = [
                    (c1[0] + c2[0]) / 2.0,
                    (c1[1] + c2[1]) / 2.0,
                    (c1[2] + c2[2]) / 2.0,
                    0.8,
                ];
                (v1, v2, color)
            })
            .collect()
    }

    /// Get vertex render data
    pub fn get_vertices_3d(&self) -> Vec<(Vec3, [f32; 4])> {
        self.projected_vertices
            .iter()
            .zip(self.vertex_colors.iter())
            .map(|(&v, &c)| (v, c))
            .collect()
    }

    /// Get current polytope name
    pub fn current_polytope_name(&self) -> &str {
        &self.polytope.name
    }
}

impl Default for Hypercube4DSimulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Map w coordinate to color (rainbow gradient)
fn w_to_color(w: f32) -> [f32; 4] {
    // HSV to RGB with hue based on w
    let hue = w.clamp(0.0, 1.0);
    let (r, g, b) = hsv_to_rgb(hue, 0.9, 1.0);
    [r, g, b, 1.0]
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// Preset configurations
impl Hypercube4DSimulation {
    pub fn preset_tesseract() -> Self {
        let mut sim = Self::new();
        sim.set_polytope(Polytope4D::tesseract(1.0));
        sim
    }

    pub fn preset_16_cell() -> Self {
        let mut sim = Self::new();
        sim.set_polytope(Polytope4D::cell_16(1.0));
        sim
    }

    pub fn preset_24_cell() -> Self {
        let mut sim = Self::new();
        sim.set_polytope(Polytope4D::cell_24(0.7));
        sim
    }

    pub fn preset_5_cell() -> Self {
        let mut sim = Self::new();
        sim.set_polytope(Polytope4D::simplex_5(0.5));
        sim
    }
}
