//! Accurate solar system model with real planetary data
//!
//! All distances in AU, masses in solar masses, times in years

use glam::Vec3;
use std::f32::consts::PI;

/// Gravitational constant in AU³/(solar mass * year²)
pub const G: f32 = 4.0 * PI * PI; // ~39.478

/// Speed of light in AU/year
pub const C: f32 = 63241.077; // ~63241 AU/year

/// Astronomical Unit in km
pub const AU_KM: f64 = 149_597_870.7;

/// Solar mass in kg
pub const SOLAR_MASS_KG: f64 = 1.989e30;

/// Celestial body types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    BlackHole,
    Spaceship,
}

/// A celestial body in the solar system
#[derive(Debug, Clone)]
pub struct CelestialBody {
    pub name: String,
    pub body_type: BodyType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,           // In solar masses
    pub radius: f32,         // In AU (visual radius, scaled up for visibility)
    pub display_radius: f32, // Scaled for display
    pub color: [f32; 4],
    pub trail: Vec<Vec3>,
    pub trail_max_length: usize,
    pub orbital_period: f32, // In years
    pub semi_major_axis: f32, // In AU
}

impl CelestialBody {
    pub fn new(name: &str, body_type: BodyType, mass: f32, radius: f32, color: [f32; 4]) -> Self {
        Self {
            name: name.to_string(),
            body_type,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            mass,
            radius,
            display_radius: radius,
            color,
            trail: Vec::new(),
            trail_max_length: 500,
            orbital_period: 0.0,
            semi_major_axis: 0.0,
        }
    }

    pub fn at_orbit(mut self, distance: f32, angle: f32, inclination: f32) -> Self {
        self.semi_major_axis = distance;

        // Position in 3D with inclination
        self.position = Vec3::new(
            distance * angle.cos() * inclination.cos(),
            distance * inclination.sin(),
            distance * angle.sin() * inclination.cos(),
        );

        // Orbital velocity (circular approximation)
        // v = sqrt(G * M_sun / r) for circular orbit
        let orbital_speed = (G / distance).sqrt();
        self.velocity = Vec3::new(
            -angle.sin() * orbital_speed,
            0.0,
            angle.cos() * orbital_speed,
        );

        // Kepler's third law: T² = a³ (in years and AU around the Sun)
        self.orbital_period = distance.powf(1.5);

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

    /// Calculate Schwarzschild radius (for black holes)
    pub fn schwarzschild_radius(&self) -> f32 {
        // rs = 2GM/c² in our units
        2.0 * G * self.mass / (C * C)
    }
}

/// The complete solar system
pub struct SolarSystem {
    pub bodies: Vec<CelestialBody>,
    pub time: f32, // In years
    pub time_scale: f32,
}

impl SolarSystem {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            time: 0.0,
            time_scale: 1.0, // 1 second = 1 year of simulation
        }
    }

    /// Initialize with accurate solar system data
    pub fn init_accurate(&mut self) {
        self.bodies.clear();

        // The Sun
        let sun = CelestialBody::new("Sun", BodyType::Star, 1.0, 0.00465, [1.0, 0.95, 0.8, 1.0])
            .with_trail_length(0);
        self.bodies.push(sun);

        // Mercury
        let mercury = CelestialBody::new(
            "Mercury",
            BodyType::Planet,
            1.66e-7, // 0.055 Earth masses
            0.0024,
            [0.7, 0.7, 0.7, 1.0],
        )
        .at_orbit(0.387, 0.0, 0.122) // 7° inclination
        .with_trail_length(200);
        self.bodies.push(mercury);

        // Venus
        let venus = CelestialBody::new(
            "Venus",
            BodyType::Planet,
            2.45e-6,
            0.006,
            [0.9, 0.7, 0.5, 1.0],
        )
        .at_orbit(0.723, 0.8, 0.059)
        .with_trail_length(300);
        self.bodies.push(venus);

        // Earth
        let earth = CelestialBody::new(
            "Earth",
            BodyType::Planet,
            3.0e-6, // 1 Earth mass / solar mass
            0.0064,
            [0.2, 0.4, 0.8, 1.0],
        )
        .at_orbit(1.0, 1.5, 0.0)
        .with_trail_length(365);
        self.bodies.push(earth);

        // Mars
        let mars = CelestialBody::new(
            "Mars",
            BodyType::Planet,
            3.23e-7,
            0.0034,
            [0.8, 0.4, 0.2, 1.0],
        )
        .at_orbit(1.524, 2.3, 0.032)
        .with_trail_length(400);
        self.bodies.push(mars);

        // Jupiter
        let jupiter = CelestialBody::new(
            "Jupiter",
            BodyType::Planet,
            9.55e-4, // 318 Earth masses
            0.07,
            [0.9, 0.8, 0.6, 1.0],
        )
        .at_orbit(5.203, 3.5, 0.023)
        .with_trail_length(600);
        self.bodies.push(jupiter);

        // Saturn
        let saturn = CelestialBody::new(
            "Saturn",
            BodyType::Planet,
            2.86e-4,
            0.058,
            [0.9, 0.85, 0.6, 1.0],
        )
        .at_orbit(9.537, 4.2, 0.043)
        .with_trail_length(800);
        self.bodies.push(saturn);

        // Uranus
        let uranus = CelestialBody::new(
            "Uranus",
            BodyType::Planet,
            4.37e-5,
            0.025,
            [0.6, 0.8, 0.9, 1.0],
        )
        .at_orbit(19.19, 5.0, 0.013)
        .with_trail_length(1000);
        self.bodies.push(uranus);

        // Neptune
        let neptune = CelestialBody::new(
            "Neptune",
            BodyType::Planet,
            5.15e-5,
            0.024,
            [0.3, 0.4, 0.8, 1.0],
        )
        .at_orbit(30.07, 5.8, 0.031)
        .with_trail_length(1200);
        self.bodies.push(neptune);
    }

    /// Add a rogue black hole passing through the solar system
    pub fn add_black_hole(&mut self, mass_solar: f32, position: Vec3, velocity: Vec3) {
        let mut bh = CelestialBody::new(
            "Black Hole",
            BodyType::BlackHole,
            mass_solar,
            mass_solar.powf(0.3) * 0.01, // Visual size
            [0.0, 0.0, 0.0, 1.0],
        );
        bh.position = position;
        bh.velocity = velocity;
        bh.trail_max_length = 1000;
        self.bodies.push(bh);
    }

    /// Step the simulation using Velocity Verlet integration
    pub fn step(&mut self, dt: f32) {
        let dt = dt * self.time_scale;
        self.time += dt;

        let n = self.bodies.len();
        if n == 0 {
            return;
        }

        // Softening parameter to prevent singularities
        let softening = 0.001;

        // Calculate accelerations
        let mut accelerations = vec![Vec3::ZERO; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.bodies[j].position - self.bodies[i].position;
                let dist_sq = r.length_squared() + softening * softening;
                let dist = dist_sq.sqrt();
                let force_mag = G / dist_sq;
                let force_dir = r / dist;

                accelerations[i] += force_dir * force_mag * self.bodies[j].mass;
                accelerations[j] -= force_dir * force_mag * self.bodies[i].mass;
            }
        }

        // Update positions and velocities
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.velocity += accelerations[i] * dt;
            body.position += body.velocity * dt;
            body.update_trail();
        }
    }

    /// Find body by name
    pub fn find_body(&self, name: &str) -> Option<&CelestialBody> {
        self.bodies.iter().find(|b| b.name == name)
    }

    /// Find body by name (mutable)
    pub fn find_body_mut(&mut self, name: &str) -> Option<&mut CelestialBody> {
        self.bodies.iter_mut().find(|b| b.name == name)
    }
}

impl Default for SolarSystem {
    fn default() -> Self {
        Self::new()
    }
}
