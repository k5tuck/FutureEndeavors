//! Quantum Hall Effect Simulation
//!
//! Simulates 2D electron gas in a magnetic field, demonstrating:
//! - Landau level quantization
//! - Integer quantum Hall effect
//! - Edge states and chiral transport
//! - Hall conductance quantization

use glam::{Vec2, Vec3};
use rand::Rng;
use std::f32::consts::PI;

/// Physical constants (scaled for visualization)
const HBAR: f32 = 1.0;
const E_CHARGE: f32 = 1.0;
const M_EFF: f32 = 1.0; // Effective electron mass

/// Electron in 2D electron gas
#[derive(Debug, Clone)]
pub struct Electron {
    pub position: Vec2,
    pub velocity: Vec2,
    pub phase: f32,           // Quantum phase
    pub landau_level: u32,    // Which Landau level
    pub guiding_center: Vec2, // Center of cyclotron orbit
    pub is_edge_state: bool,
}

impl Electron {
    pub fn new(position: Vec2, landau_level: u32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            phase: 0.0,
            landau_level,
            guiding_center: position,
            is_edge_state: false,
        }
    }
}

/// Landau level energy state
#[derive(Debug, Clone)]
pub struct LandauLevel {
    pub n: u32,              // Level index (0, 1, 2, ...)
    pub energy: f32,         // E = ℏωc(n + 1/2)
    pub degeneracy: u32,     // Number of states
    pub filled: u32,         // Number of electrons
    pub color: [f32; 4],     // Visualization color
}

impl LandauLevel {
    pub fn new(n: u32, cyclotron_freq: f32, degeneracy: u32) -> Self {
        let energy = HBAR * cyclotron_freq * (n as f32 + 0.5);

        // Color gradient from blue (n=0) to red (high n)
        let t = (n as f32 / 5.0).min(1.0);
        let color = [0.2 + 0.6 * t, 0.3, 0.9 - 0.6 * t, 0.8];

        Self {
            n,
            energy,
            degeneracy,
            filled: 0,
            color,
        }
    }

    pub fn filling_fraction(&self) -> f32 {
        if self.degeneracy > 0 {
            self.filled as f32 / self.degeneracy as f32
        } else {
            0.0
        }
    }
}

/// Edge state channel
#[derive(Debug, Clone)]
pub struct EdgeChannel {
    pub side: EdgeSide,
    pub level: u32,      // Landau level
    pub velocity: f32,   // Drift velocity along edge
    pub current: f32,    // Current carried
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Quantum Hall simulation
pub struct HallSimulation {
    /// Electrons in the system
    pub electrons: Vec<Electron>,
    /// Landau levels
    pub landau_levels: Vec<LandauLevel>,
    /// Magnetic field strength (perpendicular to 2D plane)
    pub magnetic_field: f32,
    /// Cyclotron frequency ωc = eB/m
    pub cyclotron_freq: f32,
    /// Magnetic length lb = √(ℏ/eB)
    pub magnetic_length: f32,
    /// Sample dimensions
    pub width: f32,
    pub height: f32,
    /// Electric field (for Hall voltage)
    pub electric_field: Vec2,
    /// Hall conductance in units of e²/h
    pub hall_conductance: f32,
    /// Filling factor ν
    pub filling_factor: f32,
    /// Edge channels
    pub edge_channels: Vec<EdgeChannel>,
    /// Time
    pub time: f32,
    /// Show edge states
    pub show_edge_states: bool,
    /// Temperature (affects Fermi distribution)
    pub temperature: f32,
}

impl HallSimulation {
    pub fn new(width: f32, height: f32, magnetic_field: f32) -> Self {
        let cyclotron_freq = E_CHARGE * magnetic_field / M_EFF;
        let magnetic_length = (HBAR / (E_CHARGE * magnetic_field)).sqrt();

        let mut sim = Self {
            electrons: Vec::new(),
            landau_levels: Vec::new(),
            magnetic_field,
            cyclotron_freq,
            magnetic_length,
            width,
            height,
            electric_field: Vec2::new(0.1, 0.0), // Small field in x direction
            hall_conductance: 0.0,
            filling_factor: 0.0,
            edge_channels: Vec::new(),
            time: 0.0,
            show_edge_states: true,
            temperature: 0.1,
        };

        sim.initialize_levels(5);
        sim
    }

    /// Initialize Landau levels
    fn initialize_levels(&mut self, num_levels: u32) {
        self.landau_levels.clear();

        // Degeneracy ~ area * eB / (2πℏ)
        let degeneracy = ((self.width * self.height * E_CHARGE * self.magnetic_field)
            / (2.0 * PI * HBAR)) as u32;

        for n in 0..num_levels {
            self.landau_levels.push(LandauLevel::new(n, self.cyclotron_freq, degeneracy));
        }
    }

    /// Fill electrons up to a certain Fermi level
    pub fn fill_electrons(&mut self, num_electrons: usize) {
        self.electrons.clear();

        // Reset filling
        for level in &mut self.landau_levels {
            level.filled = 0;
        }

        let mut rng = rand::thread_rng();
        let mut remaining = num_electrons;
        let width = self.width;
        let height = self.height;
        let magnetic_length = self.magnetic_length;

        // Collect level info first to avoid borrow issues
        let level_info: Vec<(u32, u32)> = self.landau_levels.iter()
            .map(|l| (l.n, l.degeneracy))
            .collect();

        let mut level_idx = 0;
        for (n, degeneracy) in level_info {
            if remaining == 0 {
                break;
            }

            let to_fill = remaining.min(degeneracy as usize);
            self.landau_levels[level_idx].filled = to_fill as u32;
            remaining -= to_fill;

            // Create electrons in this level
            for _ in 0..to_fill {
                let x = rng.gen::<f32>() * width - width / 2.0;
                let y = rng.gen::<f32>() * height - height / 2.0;
                let pos = Vec2::new(x, y);

                let mut electron = Electron::new(pos, n);
                electron.phase = rng.gen::<f32>() * 2.0 * PI;

                // Check if edge state - inline distance calculation
                let dx_left = pos.x + width / 2.0;
                let dx_right = width / 2.0 - pos.x;
                let dy_bottom = pos.y + height / 2.0;
                let dy_top = height / 2.0 - pos.y;
                let edge_dist = dx_left.min(dx_right).min(dy_bottom).min(dy_top);

                electron.is_edge_state = edge_dist < magnetic_length * 2.0;

                self.electrons.push(electron);
            }

            level_idx += 1;
        }

        self.update_observables();
        self.create_edge_channels();
    }

    /// Distance to nearest edge
    fn distance_to_edge(&self, pos: Vec2) -> f32 {
        let dx_left = pos.x + self.width / 2.0;
        let dx_right = self.width / 2.0 - pos.x;
        let dy_bottom = pos.y + self.height / 2.0;
        let dy_top = self.height / 2.0 - pos.y;

        dx_left.min(dx_right).min(dy_bottom).min(dy_top)
    }

    /// Create edge channels based on filled levels
    fn create_edge_channels(&mut self) {
        self.edge_channels.clear();

        for level in &self.landau_levels {
            if level.filled > 0 && level.filling_fraction() > 0.9 {
                // Each filled level contributes one edge channel per edge
                let drift_velocity = self.electric_field.length() / self.magnetic_field;

                for side in [EdgeSide::Top, EdgeSide::Bottom, EdgeSide::Left, EdgeSide::Right] {
                    let velocity = match side {
                        EdgeSide::Top | EdgeSide::Bottom => drift_velocity,
                        EdgeSide::Left | EdgeSide::Right => -drift_velocity,
                    };

                    self.edge_channels.push(EdgeChannel {
                        side,
                        level: level.n,
                        velocity,
                        current: E_CHARGE * velocity / (2.0 * PI * HBAR),
                    });
                }
            }
        }
    }

    /// Update physical observables
    fn update_observables(&mut self) {
        // Filling factor ν = n_e * h / (eB)
        let electron_density = self.electrons.len() as f32 / (self.width * self.height);
        self.filling_factor = electron_density * 2.0 * PI * HBAR / (E_CHARGE * self.magnetic_field);

        // Quantized Hall conductance σ_xy = ν * e²/h
        // For integer QHE, ν rounds to integer
        let nu_int = self.filling_factor.round();
        self.hall_conductance = nu_int; // In units of e²/h
    }

    /// Simulation step
    pub fn step(&mut self, dt: f32) {
        self.time += dt;

        let omega = self.cyclotron_freq;
        let magnetic_length = self.magnetic_length;
        let electric_field = self.electric_field;
        let magnetic_field = self.magnetic_field;
        let width = self.width;
        let height = self.height;

        for electron in &mut self.electrons {
            // Cyclotron motion
            let radius = magnetic_length * (2.0 * electron.landau_level as f32 + 1.0).sqrt();

            electron.phase += omega * dt;

            if electron.is_edge_state {
                // Edge states: drift along boundary
                let drift_vel = electric_field.perp() / magnetic_field;

                // Move along edge - calculate edge tangent inline
                let pos = electron.position;
                let dx_left = pos.x + width / 2.0;
                let dx_right = width / 2.0 - pos.x;
                let dy_bottom = pos.y + height / 2.0;
                let dy_top = height / 2.0 - pos.y;
                let min = dx_left.min(dx_right).min(dy_bottom).min(dy_top);

                let edge_direction = if min == dx_left {
                    Vec2::new(0.0, 1.0)
                } else if min == dx_right {
                    Vec2::new(0.0, -1.0)
                } else if min == dy_bottom {
                    Vec2::new(1.0, 0.0)
                } else {
                    Vec2::new(-1.0, 0.0)
                };

                electron.velocity = edge_direction * drift_vel.length();
                electron.position += electron.velocity * dt;

                // Keep on edge - constrain inline
                let margin = magnetic_length * 2.0;
                let half_w = width / 2.0;
                let half_h = height / 2.0;

                let dx_left = electron.position.x + half_w;
                let dx_right = half_w - electron.position.x;
                let dy_bottom = electron.position.y + half_h;
                let dy_top = half_h - electron.position.y;
                let min = dx_left.min(dx_right).min(dy_bottom).min(dy_top);

                if min == dx_left {
                    electron.position.x = -half_w + margin;
                } else if min == dx_right {
                    electron.position.x = half_w - margin;
                } else if min == dy_bottom {
                    electron.position.y = -half_h + margin;
                } else {
                    electron.position.y = half_h - margin;
                }
            } else {
                // Bulk states: cyclotron orbit
                let offset = Vec2::new(
                    radius * electron.phase.cos(),
                    radius * electron.phase.sin(),
                );
                electron.position = electron.guiding_center + offset;

                // Guiding center drift in crossed E×B fields
                let drift = electric_field.perp() / magnetic_field;
                electron.guiding_center += drift * dt;
            }

            // Boundary conditions - inline
            let half_w = width / 2.0;
            let half_h = height / 2.0;

            if electron.position.x < -half_w {
                electron.position.x += width;
                electron.guiding_center.x += width;
            }
            if electron.position.x > half_w {
                electron.position.x -= width;
                electron.guiding_center.x -= width;
            }
            if electron.position.y < -half_h {
                electron.position.y += height;
                electron.guiding_center.y += height;
            }
            if electron.position.y > half_h {
                electron.position.y -= height;
                electron.guiding_center.y -= height;
            }
        }
    }

    /// Cyclotron radius for Landau level n
    fn cyclotron_radius(&self, n: u32) -> f32 {
        self.magnetic_length * (2.0 * n as f32 + 1.0).sqrt()
    }

    /// Get tangent direction along edge
    fn edge_tangent(&self, pos: Vec2) -> Vec2 {
        let dx_left = pos.x + self.width / 2.0;
        let dx_right = self.width / 2.0 - pos.x;
        let dy_bottom = pos.y + self.height / 2.0;
        let dy_top = self.height / 2.0 - pos.y;

        let min = dx_left.min(dx_right).min(dy_bottom).min(dy_top);

        // Return tangent (counterclockwise)
        if min == dx_left {
            Vec2::new(0.0, 1.0)  // Left edge: up
        } else if min == dx_right {
            Vec2::new(0.0, -1.0) // Right edge: down
        } else if min == dy_bottom {
            Vec2::new(1.0, 0.0)  // Bottom: right
        } else {
            Vec2::new(-1.0, 0.0) // Top: left
        }
    }

    /// Constrain electron to edge
    fn constrain_to_edge(&self, electron: &mut Electron) {
        let margin = self.magnetic_length * 2.0;
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;

        // Find nearest edge and snap to it
        let dx_left = electron.position.x + half_w;
        let dx_right = half_w - electron.position.x;
        let dy_bottom = electron.position.y + half_h;
        let dy_top = half_h - electron.position.y;

        let min = dx_left.min(dx_right).min(dy_bottom).min(dy_top);

        if min == dx_left {
            electron.position.x = -half_w + margin;
        } else if min == dx_right {
            electron.position.x = half_w - margin;
        } else if min == dy_bottom {
            electron.position.y = -half_h + margin;
        } else {
            electron.position.y = half_h - margin;
        }
    }

    /// Apply periodic boundary conditions
    fn apply_boundary(&self, electron: &mut Electron) {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;

        if electron.position.x < -half_w {
            electron.position.x += self.width;
            electron.guiding_center.x += self.width;
        }
        if electron.position.x > half_w {
            electron.position.x -= self.width;
            electron.guiding_center.x -= self.width;
        }
        if electron.position.y < -half_h {
            electron.position.y += self.height;
            electron.guiding_center.y += self.height;
        }
        if electron.position.y > half_h {
            electron.position.y -= self.height;
            electron.guiding_center.y -= self.height;
        }
    }

    /// Set magnetic field and recalculate
    pub fn set_magnetic_field(&mut self, b: f32) {
        self.magnetic_field = b.max(0.1);
        self.cyclotron_freq = E_CHARGE * self.magnetic_field / M_EFF;
        self.magnetic_length = (HBAR / (E_CHARGE * self.magnetic_field)).sqrt();

        self.initialize_levels(5);
        self.fill_electrons(self.electrons.len());
    }

    /// Get render data
    pub fn get_electron_data(&self) -> Vec<(Vec2, [f32; 4], bool)> {
        self.electrons
            .iter()
            .map(|e| {
                let color = if e.is_edge_state {
                    [1.0, 0.8, 0.2, 1.0] // Yellow for edge states
                } else {
                    self.landau_levels[e.landau_level as usize].color
                };
                (e.position, color, e.is_edge_state)
            })
            .collect()
    }

    /// Get cyclotron orbit visualization
    pub fn get_orbits(&self) -> Vec<(Vec2, f32, [f32; 4])> {
        self.electrons
            .iter()
            .filter(|e| !e.is_edge_state)
            .map(|e| {
                let radius = self.cyclotron_radius(e.landau_level);
                let color = self.landau_levels[e.landau_level as usize].color;
                (e.guiding_center, radius, color)
            })
            .collect()
    }

    /// Get energy level diagram data
    pub fn get_level_diagram(&self) -> Vec<(f32, f32, [f32; 4])> {
        self.landau_levels
            .iter()
            .map(|level| {
                (level.energy, level.filling_fraction(), level.color)
            })
            .collect()
    }
}

impl Default for HallSimulation {
    fn default() -> Self {
        let mut sim = Self::new(10.0, 8.0, 2.0);
        sim.fill_electrons(100);
        sim
    }
}

/// Presets for different filling factors
impl HallSimulation {
    /// Integer quantum Hall effect at ν=1
    pub fn preset_nu_1() -> Self {
        let mut sim = Self::new(10.0, 8.0, 2.0);
        // Calculate electron number for ν ≈ 1
        let n_phi = (sim.width * sim.height * E_CHARGE * sim.magnetic_field)
            / (2.0 * PI * HBAR);
        sim.fill_electrons(n_phi as usize);
        sim
    }

    /// ν=2 state
    pub fn preset_nu_2() -> Self {
        let mut sim = Self::new(10.0, 8.0, 2.0);
        let n_phi = (sim.width * sim.height * E_CHARGE * sim.magnetic_field)
            / (2.0 * PI * HBAR);
        sim.fill_electrons((2.0 * n_phi) as usize);
        sim
    }
}
