//! Spaceship physics with relativistic effects
//!
//! Simulates a spacecraft traveling through the solar system with:
//! - Controllable thrust
//! - Relativistic time dilation
//! - Length contraction
//! - Gravitational time dilation near massive objects

use glam::{Mat4, Quat, Vec3};

use crate::solar_system::{CelestialBody, BodyType, C, G};

/// Spaceship state and physics
#[derive(Debug, Clone)]
pub struct Spaceship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub mass: f32, // In solar masses (tiny!)

    // Thrust parameters
    pub thrust: f32,           // Current thrust force
    pub max_thrust: f32,       // Maximum thrust
    pub fuel: f32,             // Remaining fuel (0-1)
    pub fuel_consumption: f32, // Fuel used per unit thrust per second

    // Relativistic state
    pub proper_time: f32,      // Time experienced on ship
    pub coordinate_time: f32,  // External observer time
    pub gamma: f32,            // Lorentz factor
    pub gravitational_gamma: f32, // Gravitational time dilation factor

    // Trail for visualization
    pub trail: Vec<Vec3>,
    pub trail_max_length: usize,

    // Display properties
    pub color: [f32; 4],
    pub display_radius: f32,
}

impl Spaceship {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(1.5, 0.0, 0.0), // Start near Earth
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            mass: 1e-20, // ~100 tons, negligible in solar mass units

            thrust: 0.0,
            max_thrust: 1e-10, // Very small in solar system units
            fuel: 1.0,
            fuel_consumption: 0.001,

            proper_time: 0.0,
            coordinate_time: 0.0,
            gamma: 1.0,
            gravitational_gamma: 1.0,

            trail: Vec::new(),
            trail_max_length: 1000,

            color: [0.2, 0.8, 0.2, 1.0],
            display_radius: 0.02,
        }
    }

    /// Launch from a specific planet with escape velocity
    pub fn launch_from(&mut self, body: &CelestialBody, direction: Vec3) {
        // Position slightly above the surface
        let offset = direction.normalize() * (body.radius + 0.01);
        self.position = body.position + offset;

        // Escape velocity: v_esc = sqrt(2GM/r)
        let escape_v = (2.0 * G * body.mass / body.radius).sqrt();
        self.velocity = body.velocity + direction.normalize() * escape_v * 1.1;
    }

    /// Calculate the Lorentz factor (special relativity)
    pub fn calculate_gamma(&self) -> f32 {
        let v_squared = self.velocity.length_squared();
        let c_squared = C * C;

        if v_squared >= c_squared * 0.999 {
            // Cap at high but finite gamma
            return 22.36; // gamma at 0.999c
        }

        1.0 / (1.0 - v_squared / c_squared).sqrt()
    }

    /// Calculate gravitational time dilation factor near a massive body
    pub fn calculate_gravitational_dilation(&self, bodies: &[CelestialBody]) -> f32 {
        let mut total_potential = 0.0;

        for body in bodies {
            let r = (self.position - body.position).length();
            if r > 0.0001 {
                // Gravitational potential: φ = -GM/r
                let potential = G * body.mass / r;
                total_potential += potential;
            }
        }

        // Time dilation: τ/t = sqrt(1 - 2φ/c²)
        let c_squared = C * C;
        let factor = 1.0 - 2.0 * total_potential / c_squared;

        if factor > 0.0 {
            factor.sqrt()
        } else {
            0.001 // Inside event horizon, extreme dilation
        }
    }

    /// Get the forward direction based on orientation
    pub fn forward(&self) -> Vec3 {
        self.orientation * Vec3::Z
    }

    /// Get the right direction
    pub fn right(&self) -> Vec3 {
        self.orientation * Vec3::X
    }

    /// Get the up direction
    pub fn up(&self) -> Vec3 {
        self.orientation * Vec3::Y
    }

    /// Apply thrust in the forward direction
    pub fn apply_thrust(&mut self, amount: f32, dt: f32) {
        if self.fuel <= 0.0 {
            return;
        }

        self.thrust = (amount * self.max_thrust).min(self.max_thrust);

        // Consume fuel
        self.fuel -= self.fuel_consumption * self.thrust.abs() * dt;
        self.fuel = self.fuel.max(0.0);

        // Apply acceleration (F = ma, but we need relativistic mass)
        // For simplicity, we'll use classical mechanics but display relativistic effects
        let acceleration = self.forward() * self.thrust / self.mass;
        self.velocity += acceleration * dt;

        // Cap velocity at 0.99c
        let max_speed = C * 0.99;
        if self.velocity.length() > max_speed {
            self.velocity = self.velocity.normalize() * max_speed;
        }
    }

    /// Rotate the spaceship
    pub fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32) {
        let pitch_rot = Quat::from_axis_angle(self.right(), pitch);
        let yaw_rot = Quat::from_axis_angle(self.up(), yaw);
        let roll_rot = Quat::from_axis_angle(self.forward(), roll);

        self.orientation = pitch_rot * yaw_rot * roll_rot * self.orientation;
        self.orientation = self.orientation.normalize();
    }

    /// Update spaceship physics
    pub fn update(&mut self, bodies: &[CelestialBody], dt: f32) {
        // Calculate gravitational acceleration from all bodies
        let mut acceleration = Vec3::ZERO;

        for body in bodies {
            let r = body.position - self.position;
            let dist_sq = r.length_squared();

            if dist_sq > 0.0001 {
                let dist = dist_sq.sqrt();
                let force_mag = G * body.mass / dist_sq;
                acceleration += r.normalize() * force_mag;
            }
        }

        // Update velocity and position
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        // Update relativistic factors
        self.gamma = self.calculate_gamma();
        self.gravitational_gamma = self.calculate_gravitational_dilation(bodies);

        // Update proper time (time experienced on ship)
        // Combines special and general relativistic time dilation
        let total_dilation = self.gravitational_gamma / self.gamma;
        self.proper_time += dt * total_dilation;
        self.coordinate_time += dt;

        // Update trail
        self.trail.push(self.position);
        if self.trail.len() > self.trail_max_length {
            self.trail.remove(0);
        }
    }

    /// Get the view matrix from the spaceship's perspective
    pub fn view_matrix(&self) -> Mat4 {
        let target = self.position + self.forward();
        Mat4::look_at_rh(self.position, target, self.up())
    }

    /// Get velocity as fraction of c
    pub fn velocity_fraction_c(&self) -> f32 {
        self.velocity.length() / C
    }

    /// Get speed in km/s (for display)
    pub fn speed_km_per_s(&self) -> f32 {
        // 1 AU/year ≈ 4.74 km/s
        self.velocity.length() * 4.74
    }

    /// Calculate time dilation factor (how much slower ship time runs)
    pub fn time_dilation_factor(&self) -> f32 {
        self.gravitational_gamma / self.gamma
    }

    /// Calculate length contraction in direction of motion
    pub fn length_contraction(&self) -> f32 {
        1.0 / self.gamma
    }

    /// Get the Lorentz factor (gamma)
    pub fn lorentz_factor(&self) -> f32 {
        self.gamma
    }

    /// Get info string for HUD
    pub fn info_string(&self) -> String {
        format!(
            "Speed: {:.2}% c ({:.0} km/s)\n\
             Lorentz γ: {:.4}\n\
             Time dilation: {:.4}x\n\
             Ship time: {:.2} years\n\
             Coord time: {:.2} years\n\
             Fuel: {:.1}%",
            self.velocity_fraction_c() * 100.0,
            self.speed_km_per_s(),
            self.gamma,
            self.time_dilation_factor(),
            self.proper_time,
            self.coordinate_time,
            self.fuel * 100.0,
        )
    }
}

impl Default for Spaceship {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert spaceship to a CelestialBody for rendering
impl From<&Spaceship> for CelestialBody {
    fn from(ship: &Spaceship) -> Self {
        let mut body = CelestialBody::new(
            "Spaceship",
            BodyType::Spaceship,
            ship.mass,
            ship.display_radius,
            ship.color,
        );
        body.position = ship.position;
        body.velocity = ship.velocity;
        body.trail = ship.trail.clone();
        body.trail_max_length = ship.trail_max_length;
        body
    }
}
