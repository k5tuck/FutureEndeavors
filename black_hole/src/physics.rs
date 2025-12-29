//! Black hole physics and ray tracing
//!
//! Implements Schwarzschild spacetime geodesics for simulating
//! how light bends around a non-rotating black hole.

use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// Physical constants (scaled for simulation)
pub const C: f32 = 1.0; // Speed of light (normalized)
pub const G: f32 = 1.0; // Gravitational constant (normalized)

/// Schwarzschild black hole
#[derive(Debug, Clone, Copy)]
pub struct BlackHole {
    pub position: Vec3,
    pub mass: f32,
    pub schwarzschild_radius: f32,
}

impl BlackHole {
    pub fn new(position: Vec3, mass: f32) -> Self {
        // Schwarzschild radius: rs = 2GM/c²
        let schwarzschild_radius = 2.0 * G * mass / (C * C);

        Self {
            position,
            mass,
            schwarzschild_radius,
        }
    }

    /// Check if a point is inside the event horizon
    pub fn is_inside_horizon(&self, point: Vec3) -> bool {
        (point - self.position).length() < self.schwarzschild_radius
    }

    /// Calculate gravitational potential at a point
    pub fn potential(&self, point: Vec3) -> f32 {
        let r = (point - self.position).length();
        if r < self.schwarzschild_radius {
            return f32::NEG_INFINITY;
        }
        -G * self.mass / r
    }
}

impl Default for BlackHole {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 1.0)
    }
}

/// A light ray for ray tracing through curved spacetime
#[derive(Debug, Clone, Copy)]
pub struct LightRay {
    pub position: Vec3,
    pub direction: Vec3,
    pub wavelength: f32, // For color/redshift
}

impl LightRay {
    pub fn new(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            wavelength: 550.0, // Green light
        }
    }

    /// Trace the ray through spacetime using RK4 integration
    /// Returns the final direction (or None if captured by black hole)
    pub fn trace(&self, black_hole: &BlackHole, steps: usize, step_size: f32) -> Option<Vec3> {
        let mut pos = self.position;
        let mut dir = self.direction;

        for _ in 0..steps {
            // Check if ray fell into black hole
            if black_hole.is_inside_horizon(pos) {
                return None;
            }

            // RK4 integration for geodesic equation
            let (new_pos, new_dir) = rk4_step(pos, dir, black_hole, step_size);
            pos = new_pos;
            dir = new_dir.normalize();

            // Check if ray escaped to infinity
            let r = (pos - black_hole.position).length();
            if r > 50.0 {
                return Some(dir);
            }
        }

        Some(dir)
    }
}

/// Calculate gravitational acceleration (for light bending)
fn gravitational_acceleration(pos: Vec3, black_hole: &BlackHole) -> Vec3 {
    let r_vec = pos - black_hole.position;
    let r = r_vec.length();

    if r < black_hole.schwarzschild_radius * 1.01 {
        return Vec3::ZERO;
    }

    // Newtonian approximation for visualization
    // For more accuracy, use full Schwarzschild geodesic equations
    let r_hat = r_vec / r;

    // Enhanced bending near the photon sphere (1.5 * rs)
    let photon_sphere = 1.5 * black_hole.schwarzschild_radius;
    let enhancement = if r < photon_sphere * 2.0 {
        1.0 + 2.0 * (photon_sphere / r).powi(2)
    } else {
        1.0
    };

    -r_hat * (G * black_hole.mass / (r * r)) * enhancement * 3.0
}

/// RK4 integration step for geodesic motion
fn rk4_step(pos: Vec3, vel: Vec3, black_hole: &BlackHole, dt: f32) -> (Vec3, Vec3) {
    // k1
    let a1 = gravitational_acceleration(pos, black_hole);
    let k1_v = a1;
    let k1_x = vel;

    // k2
    let a2 = gravitational_acceleration(pos + k1_x * dt * 0.5, black_hole);
    let k2_v = a2;
    let k2_x = vel + k1_v * dt * 0.5;

    // k3
    let a3 = gravitational_acceleration(pos + k2_x * dt * 0.5, black_hole);
    let k3_v = a3;
    let k3_x = vel + k2_v * dt * 0.5;

    // k4
    let a4 = gravitational_acceleration(pos + k3_x * dt, black_hole);
    let k4_v = a4;
    let k4_x = vel + k3_v * dt;

    let new_pos = pos + (k1_x + k2_x * 2.0 + k3_x * 2.0 + k4_x) * dt / 6.0;
    let new_vel = vel + (k1_v + k2_v * 2.0 + k3_v * 2.0 + k4_v) * dt / 6.0;

    (new_pos, new_vel)
}

/// 2D light ray for simpler visualization
#[derive(Debug, Clone)]
pub struct LightRay2D {
    pub position: Vec2,
    pub direction: Vec2,
    pub path: Vec<Vec2>,
}

impl LightRay2D {
    pub fn new(position: Vec2, direction: Vec2) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            path: vec![position],
        }
    }

    /// Trace the ray and store the path
    pub fn trace(&mut self, black_hole_pos: Vec2, mass: f32, steps: usize, step_size: f32) -> bool {
        let rs = 2.0 * G * mass / (C * C);
        let mut pos = self.position;
        let mut dir = self.direction;

        for _ in 0..steps {
            let r_vec = pos - black_hole_pos;
            let r = r_vec.length();

            // Captured by black hole
            if r < rs {
                self.path.push(black_hole_pos);
                return false;
            }

            // Escaped to infinity
            if r > 30.0 {
                self.path.push(pos + dir * 10.0);
                return true;
            }

            // Calculate bending
            let r_hat = r_vec / r;
            let photon_sphere = 1.5 * rs;
            let enhancement = if r < photon_sphere * 2.0 {
                1.0 + 3.0 * (photon_sphere / r).powi(2)
            } else {
                1.0
            };

            let accel = -r_hat * (G * mass / (r * r)) * enhancement * 3.0;

            // Update using velocity Verlet
            let new_dir = (dir + accel * step_size).normalize();
            pos += (dir + new_dir) * 0.5 * step_size;
            dir = new_dir;

            self.path.push(pos);
        }

        true
    }
}

/// Accretion disk properties
#[derive(Debug, Clone, Copy)]
pub struct AccretionDisk {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub temperature_inner: f32, // Kelvin
    pub temperature_outer: f32,
}

impl AccretionDisk {
    pub fn new(black_hole: &BlackHole) -> Self {
        // Inner edge at ISCO (Innermost Stable Circular Orbit) = 3 * rs
        let inner_radius = 3.0 * black_hole.schwarzschild_radius;
        let outer_radius = 15.0 * black_hole.schwarzschild_radius;

        Self {
            inner_radius,
            outer_radius,
            temperature_inner: 10000.0,
            temperature_outer: 3000.0,
        }
    }

    /// Get temperature at a given radius (for color calculation)
    pub fn temperature_at(&self, radius: f32) -> f32 {
        if radius < self.inner_radius || radius > self.outer_radius {
            return 0.0;
        }

        let t = (radius - self.inner_radius) / (self.outer_radius - self.inner_radius);
        self.temperature_inner * (1.0 - t) + self.temperature_outer * t
    }

    /// Convert temperature to RGB color (blackbody approximation)
    pub fn temperature_to_color(temp: f32) -> [f32; 3] {
        // Simplified blackbody color
        let t = temp / 10000.0;

        let r = if t < 0.5 {
            1.0
        } else {
            (1.0 - (t - 0.5) * 0.5).clamp(0.5, 1.0)
        };

        let g = if t < 0.3 {
            t / 0.3 * 0.8
        } else if t < 0.7 {
            0.8 + (t - 0.3) * 0.5
        } else {
            1.0 - (t - 0.7) * 0.3
        };

        let b = if t < 0.5 {
            t * 0.6
        } else {
            0.3 + (t - 0.5) * 1.4
        };

        [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
    }
}

/// Spacetime grid for visualization
pub struct SpacetimeGrid {
    pub vertices: Vec<Vec3>,
    pub grid_size: usize,
    pub extent: f32,
}

impl SpacetimeGrid {
    pub fn new(grid_size: usize, extent: f32) -> Self {
        let mut vertices = Vec::new();

        let step = extent * 2.0 / grid_size as f32;
        for i in 0..=grid_size {
            for j in 0..=grid_size {
                let x = -extent + i as f32 * step;
                let z = -extent + j as f32 * step;
                vertices.push(Vec3::new(x, 0.0, z));
            }
        }

        Self {
            vertices,
            grid_size,
            extent,
        }
    }

    /// Deform the grid based on black hole's gravity
    pub fn deform(&mut self, black_hole: &BlackHole) {
        let step = self.extent * 2.0 / self.grid_size as f32;

        for i in 0..=self.grid_size {
            for j in 0..=self.grid_size {
                let idx = i * (self.grid_size + 1) + j;
                let x = -self.extent + i as f32 * step;
                let z = -self.extent + j as f32 * step;

                let r = (Vec2::new(x, z) - Vec2::new(black_hole.position.x, black_hole.position.z))
                    .length();

                // Deformation based on Schwarzschild metric (simplified)
                let y = if r > black_hole.schwarzschild_radius {
                    -black_hole.schwarzschild_radius * (black_hole.schwarzschild_radius / r).sqrt()
                } else {
                    -self.extent // Deep depression at singularity
                };

                self.vertices[idx] = Vec3::new(x, y, z);
            }
        }
    }
}

/// Generate star field background
pub fn generate_star_field(count: usize, radius: f32) -> Vec<(Vec3, f32)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut stars = Vec::with_capacity(count);

    for _ in 0..count {
        // Random point on sphere
        let theta = rng.gen::<f32>() * 2.0 * PI;
        let phi = (rng.gen::<f32>() * 2.0 - 1.0).acos();

        let pos = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.sin() * theta.sin(),
            radius * phi.cos(),
        );

        let brightness = 0.3 + rng.gen::<f32>() * 0.7;
        stars.push((pos, brightness));
    }

    stars
}
