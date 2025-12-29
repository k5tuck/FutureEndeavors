//! N-body gravitational physics simulation

use glam::Vec2;
use rand::Rng;

/// Gravitational constant (scaled for visualization)
pub const G: f32 = 100.0;

/// A body in the simulation with mass, position, and velocity
#[derive(Debug, Clone, Copy)]
pub struct Body {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub radius: f32,
    pub color: [f32; 4],
}

impl Body {
    pub fn new(position: Vec2, velocity: Vec2, mass: f32) -> Self {
        // Radius proportional to cube root of mass (for volume scaling)
        let radius = (mass / 1000.0).powf(1.0 / 3.0) * 0.5;

        // Color based on mass (blue for light, red for heavy)
        let t = (mass / 10000.0).clamp(0.0, 1.0);
        let color = [
            0.2 + 0.8 * t,           // R
            0.4 + 0.3 * (1.0 - t),   // G
            1.0 - 0.6 * t,           // B
            1.0,                      // A
        ];

        Self {
            position,
            velocity,
            mass,
            radius,
            color,
        }
    }

    /// Create a central massive body (like a star or black hole)
    pub fn central(mass: f32) -> Self {
        Self::new(Vec2::ZERO, Vec2::ZERO, mass)
    }
}

/// The physics simulation state
pub struct Simulation {
    pub bodies: Vec<Body>,
    pub time_scale: f32,
    pub softening: f32, // Prevents singularities at close distances
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            time_scale: 1.0,
            softening: 0.1,
        }
    }

    /// Initialize with a solar system-like configuration
    pub fn init_solar_system(&mut self) {
        self.bodies.clear();

        // Central star
        self.bodies.push(Body::new(
            Vec2::ZERO,
            Vec2::ZERO,
            10000.0,
        ));

        // Planets in orbits
        let mut rng = rand::thread_rng();
        for i in 0..6 {
            let distance = 2.0 + (i as f32) * 1.5;
            let angle: f32 = rng.gen::<f32>() * std::f32::consts::TAU;
            let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);

            // Orbital velocity for circular orbit: v = sqrt(G*M/r)
            let orbital_speed = (G * 10000.0 / distance).sqrt();
            let velocity = Vec2::new(-angle.sin() * orbital_speed, angle.cos() * orbital_speed);

            let mass = 50.0 + rng.gen::<f32>() * 200.0;
            self.bodies.push(Body::new(position, velocity, mass));
        }
    }

    /// Initialize with random particles in a disk
    pub fn init_disk(&mut self, count: usize) {
        self.bodies.clear();

        // Central mass
        self.bodies.push(Body::new(
            Vec2::ZERO,
            Vec2::ZERO,
            50000.0,
        ));

        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let distance = 1.5 + rng.gen::<f32>() * 8.0;
            let angle: f32 = rng.gen::<f32>() * std::f32::consts::TAU;
            let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);

            // Orbital velocity with some randomness
            let orbital_speed = (G * 50000.0 / distance).sqrt();
            let speed_variation = 0.9 + rng.gen::<f32>() * 0.2;
            let velocity = Vec2::new(
                -angle.sin() * orbital_speed * speed_variation,
                angle.cos() * orbital_speed * speed_variation,
            );

            let mass = 10.0 + rng.gen::<f32>() * 50.0;
            self.bodies.push(Body::new(position, velocity, mass));
        }
    }

    /// Initialize with two colliding galaxies
    pub fn init_galaxy_collision(&mut self, particles_per_galaxy: usize) {
        self.bodies.clear();
        let mut rng = rand::thread_rng();

        for (center, center_vel, color_base) in [
            (Vec2::new(-5.0, 0.0), Vec2::new(0.5, 0.3), [0.3, 0.5, 1.0, 1.0]),
            (Vec2::new(5.0, 0.0), Vec2::new(-0.5, -0.3), [1.0, 0.5, 0.3, 1.0]),
        ] {
            // Central black hole
            let mut central = Body::new(center, center_vel, 30000.0);
            central.color = [1.0, 1.0, 0.8, 1.0];
            self.bodies.push(central);

            // Disk particles
            for _ in 0..particles_per_galaxy {
                let distance = 0.5 + rng.gen::<f32>() * 4.0;
                let angle: f32 = rng.gen::<f32>() * std::f32::consts::TAU;
                let position = center + Vec2::new(angle.cos() * distance, angle.sin() * distance);

                let orbital_speed = (G * 30000.0 / distance).sqrt();
                let velocity = center_vel + Vec2::new(
                    -angle.sin() * orbital_speed,
                    angle.cos() * orbital_speed,
                );

                let mass = 5.0 + rng.gen::<f32>() * 20.0;
                let mut body = Body::new(position, velocity, mass);
                body.color = color_base;
                self.bodies.push(body);
            }
        }
    }

    /// Step the simulation forward by dt seconds
    pub fn step(&mut self, dt: f32) {
        let dt = dt * self.time_scale;
        let n = self.bodies.len();

        if n == 0 {
            return;
        }

        // Calculate accelerations using leapfrog integration
        let mut accelerations = vec![Vec2::ZERO; n];

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
        }
    }

    /// Get the center of mass of all bodies
    pub fn center_of_mass(&self) -> Vec2 {
        let mut total_mass = 0.0;
        let mut com = Vec2::ZERO;

        for body in &self.bodies {
            com += body.position * body.mass;
            total_mass += body.mass;
        }

        if total_mass > 0.0 {
            com / total_mass
        } else {
            Vec2::ZERO
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
