//! 3D N-body gravitational physics simulation

use glam::Vec3;
use rand::Rng;
use std::f32::consts::{PI, TAU};

/// Gravitational constant (scaled for visualization)
pub const G: f32 = 100.0;

/// A body in the 3D simulation with mass, position, and velocity
#[derive(Debug, Clone)]
pub struct Body3D {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub radius: f32,
    pub color: [f32; 4],
    pub trail: Vec<Vec3>,
    pub trail_max_length: usize,
    pub name: Option<String>,
}

impl Body3D {
    pub fn new(position: Vec3, velocity: Vec3, mass: f32) -> Self {
        let radius = (mass / 1000.0).powf(1.0 / 3.0) * 0.5;

        let t = (mass / 10000.0).clamp(0.0, 1.0);
        let color = [
            0.2 + 0.8 * t,
            0.4 + 0.3 * (1.0 - t),
            1.0 - 0.6 * t,
            1.0,
        ];

        Self {
            position,
            velocity,
            mass,
            radius,
            color,
            trail: Vec::new(),
            trail_max_length: 200,
            name: None,
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_trail_length(mut self, length: usize) -> Self {
        self.trail_max_length = length;
        self
    }

    pub fn update_trail(&mut self) {
        self.trail.push(self.position);
        if self.trail.len() > self.trail_max_length {
            self.trail.remove(0);
        }
    }
}

/// The 3D physics simulation state
pub struct Simulation3D {
    pub bodies: Vec<Body3D>,
    pub time_scale: f32,
    pub softening: f32,
    pub elapsed_time: f32,
    pub record_trails: bool,
}

impl Simulation3D {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            time_scale: 1.0,
            softening: 0.1,
            elapsed_time: 0.0,
            record_trails: true,
        }
    }

    /// Initialize with a solar system-like configuration in 3D
    pub fn init_solar_system(&mut self) {
        self.bodies.clear();

        // Central star (Sun-like)
        let sun = Body3D::new(Vec3::ZERO, Vec3::ZERO, 10000.0)
            .with_color([1.0, 0.9, 0.6, 1.0])
            .with_radius(1.5)
            .with_name("Sun")
            .with_trail_length(0);
        self.bodies.push(sun);

        let mut rng = rand::thread_rng();

        // Planets with slight orbital inclinations
        let planet_data = [
            (2.5, 80.0, [0.6, 0.6, 0.6, 1.0], "Mercury", 0.1),
            (4.0, 120.0, [0.9, 0.7, 0.5, 1.0], "Venus", 0.05),
            (5.5, 150.0, [0.2, 0.4, 0.8, 1.0], "Earth", 0.0),
            (7.5, 100.0, [0.8, 0.3, 0.2, 1.0], "Mars", 0.03),
            (12.0, 400.0, [0.9, 0.7, 0.5, 1.0], "Jupiter", 0.02),
            (18.0, 300.0, [0.9, 0.8, 0.6, 1.0], "Saturn", 0.04),
        ];

        for (distance, mass, color, name, inclination) in planet_data {
            let angle: f32 = rng.gen::<f32>() * TAU;
            let incl_angle = inclination * PI;

            let position = Vec3::new(
                angle.cos() * distance,
                incl_angle.sin() * distance * 0.1,
                angle.sin() * distance,
            );

            let orbital_speed = (G * 10000.0 / distance).sqrt();
            let velocity = Vec3::new(
                -angle.sin() * orbital_speed,
                0.0,
                angle.cos() * orbital_speed,
            );

            let planet = Body3D::new(position, velocity, mass)
                .with_color(color)
                .with_name(name)
                .with_trail_length(300);
            self.bodies.push(planet);
        }
    }

    /// Initialize with an accretion disk
    pub fn init_accretion_disk(&mut self, count: usize) {
        self.bodies.clear();

        // Central massive object
        let center = Body3D::new(Vec3::ZERO, Vec3::ZERO, 50000.0)
            .with_color([1.0, 1.0, 0.9, 1.0])
            .with_radius(2.0)
            .with_trail_length(0);
        self.bodies.push(center);

        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let distance = 2.0 + rng.gen::<f32>() * 15.0;
            let angle: f32 = rng.gen::<f32>() * TAU;
            let height = (rng.gen::<f32>() - 0.5) * 0.5 * (1.0 - distance / 20.0);

            let position = Vec3::new(
                angle.cos() * distance,
                height,
                angle.sin() * distance,
            );

            let orbital_speed = (G * 50000.0 / distance).sqrt();
            let speed_var = 0.95 + rng.gen::<f32>() * 0.1;
            let velocity = Vec3::new(
                -angle.sin() * orbital_speed * speed_var,
                0.0,
                angle.cos() * orbital_speed * speed_var,
            );

            // Hot inner disk, cooler outer disk
            let t = (distance - 2.0) / 15.0;
            let color = [
                1.0 - t * 0.3,
                0.5 + t * 0.3,
                0.2 + t * 0.6,
                0.9,
            ];

            let particle = Body3D::new(position, velocity, 5.0 + rng.gen::<f32>() * 15.0)
                .with_color(color)
                .with_trail_length(50);
            self.bodies.push(particle);
        }
    }

    /// Initialize with colliding galaxies
    pub fn init_galaxy_collision(&mut self, particles_per_galaxy: usize) {
        self.bodies.clear();
        let mut rng = rand::thread_rng();

        let galaxy_params = [
            (Vec3::new(-15.0, 2.0, 0.0), Vec3::new(1.0, -0.2, 0.5), [0.4, 0.6, 1.0, 1.0]),
            (Vec3::new(15.0, -2.0, 0.0), Vec3::new(-1.0, 0.2, -0.5), [1.0, 0.6, 0.4, 1.0]),
        ];

        for (center, center_vel, color_base) in galaxy_params {
            // Central black hole
            let bh = Body3D::new(center, center_vel, 30000.0)
                .with_color([1.0, 1.0, 0.9, 1.0])
                .with_radius(1.5)
                .with_trail_length(500);
            self.bodies.push(bh);

            // Disk particles with random rotation axis
            let axis_tilt = rng.gen::<f32>() * 0.3;
            for _ in 0..particles_per_galaxy {
                let distance = 1.0 + rng.gen::<f32>() * 8.0;
                let angle: f32 = rng.gen::<f32>() * TAU;
                let height = (rng.gen::<f32>() - 0.5) * 0.3;

                let local_pos = Vec3::new(
                    angle.cos() * distance,
                    height + axis_tilt.sin() * distance * 0.2,
                    angle.sin() * distance,
                );
                let position = center + local_pos;

                let orbital_speed = (G * 30000.0 / distance).sqrt();
                let local_vel = Vec3::new(
                    -angle.sin() * orbital_speed,
                    0.0,
                    angle.cos() * orbital_speed,
                );
                let velocity = center_vel + local_vel;

                let particle = Body3D::new(position, velocity, 3.0 + rng.gen::<f32>() * 10.0)
                    .with_color(color_base)
                    .with_trail_length(100);
                self.bodies.push(particle);
            }
        }
    }

    /// Step the simulation forward
    pub fn step(&mut self, dt: f32) {
        let dt = dt * self.time_scale;
        let n = self.bodies.len();

        if n == 0 {
            return;
        }

        self.elapsed_time += dt;

        // Calculate accelerations using leapfrog integration
        let mut accelerations = vec![Vec3::ZERO; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.bodies[j].position - self.bodies[i].position;
                let dist_sq = r.length_squared() + self.softening * self.softening;
                let dist = dist_sq.sqrt();
                let force_mag = G / dist_sq;
                let force_dir = r / dist;

                accelerations[i] += force_dir * force_mag * self.bodies[j].mass;
                accelerations[j] -= force_dir * force_mag * self.bodies[i].mass;
            }
        }

        // Update velocities and positions
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.velocity += accelerations[i] * dt;
            body.position += body.velocity * dt;

            if self.record_trails {
                body.update_trail();
            }
        }
    }

    pub fn center_of_mass(&self) -> Vec3 {
        let mut total_mass = 0.0;
        let mut com = Vec3::ZERO;

        for body in &self.bodies {
            com += body.position * body.mass;
            total_mass += body.mass;
        }

        if total_mass > 0.0 {
            com / total_mass
        } else {
            Vec3::ZERO
        }
    }

    pub fn total_energy(&self) -> f32 {
        let mut kinetic = 0.0;
        let mut potential = 0.0;

        for (i, body_i) in self.bodies.iter().enumerate() {
            kinetic += 0.5 * body_i.mass * body_i.velocity.length_squared();

            for body_j in self.bodies.iter().skip(i + 1) {
                let r = (body_j.position - body_i.position).length();
                potential -= G * body_i.mass * body_j.mass / r.max(self.softening);
            }
        }

        kinetic + potential
    }
}

impl Default for Simulation3D {
    fn default() -> Self {
        Self::new()
    }
}
