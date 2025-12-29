//! Quantum Tunneling Simulation
//!
//! Simulates a wave packet encountering a potential barrier,
//! demonstrating the quantum mechanical phenomenon of barrier penetration.

use crate::wavefunction::{Complex, Wavefunction1D};

/// Reduced Planck constant (scaled for visualization)
const HBAR: f32 = 1.0;
/// Electron mass (scaled)
const M_E: f32 = 1.0;

/// Potential barrier types
#[derive(Debug, Clone, Copy)]
pub enum Barrier {
    /// Rectangular barrier: height and width
    Rectangular { height: f32, width: f32, center: f32 },
    /// Gaussian barrier
    Gaussian { height: f32, sigma: f32, center: f32 },
    /// Double barrier (resonant tunneling)
    Double { height: f32, width: f32, spacing: f32, center: f32 },
    /// Step potential
    Step { height: f32, position: f32 },
}

impl Barrier {
    /// Evaluate potential at position x
    pub fn potential_at(&self, x: f32) -> f32 {
        match *self {
            Barrier::Rectangular { height, width, center } => {
                if (x - center).abs() < width / 2.0 {
                    height
                } else {
                    0.0
                }
            }
            Barrier::Gaussian { height, sigma, center } => {
                height * (-(x - center).powi(2) / (2.0 * sigma * sigma)).exp()
            }
            Barrier::Double { height, width, spacing, center } => {
                let d = spacing / 2.0 + width / 2.0;
                let in_left = (x - (center - d)).abs() < width / 2.0;
                let in_right = (x - (center + d)).abs() < width / 2.0;
                if in_left || in_right { height } else { 0.0 }
            }
            Barrier::Step { height, position } => {
                if x > position { height } else { 0.0 }
            }
        }
    }
}

/// Tunneling simulation using split-operator method
pub struct TunnelingSimulation {
    /// Wavefunction on spatial grid
    pub wavefunction: Wavefunction1D,
    /// Potential barrier
    pub barrier: Barrier,
    /// Precomputed potential values
    potential: Vec<f32>,
    /// Precomputed kinetic propagator (momentum space)
    kinetic_prop: Vec<Complex>,
    /// Time step
    dt: f32,
    /// Total elapsed time
    pub time: f32,
    /// Mass of particle
    mass: f32,
    /// Initial momentum for reference
    pub initial_k: f32,
    /// Transmission coefficient (computed dynamically)
    pub transmission: f32,
    /// Reflection coefficient
    pub reflection: f32,
}

impl TunnelingSimulation {
    pub fn new(n_points: usize, x_min: f32, x_max: f32, barrier: Barrier) -> Self {
        let wavefunction = Wavefunction1D::new(n_points, x_min, x_max);
        let dx = wavefunction.dx;
        let dt = 0.001;
        let mass = M_E;

        // Compute potential on grid
        let potential: Vec<f32> = (0..n_points)
            .map(|i| {
                let x = x_min + i as f32 * dx;
                barrier.potential_at(x)
            })
            .collect();

        // Compute kinetic propagator in momentum space
        // k values for FFT: 0, 1, ..., N/2, -N/2+1, ..., -1
        let n = n_points;
        let dk = 2.0 * std::f32::consts::PI / ((x_max - x_min) * n as f32);
        let kinetic_prop: Vec<Complex> = (0..n)
            .map(|i| {
                let k = if i <= n / 2 {
                    i as f32 * dk
                } else {
                    (i as i32 - n as i32) as f32 * dk
                };
                let energy = HBAR * HBAR * k * k / (2.0 * mass);
                Complex::exp_i(-energy * dt / HBAR)
            })
            .collect();

        Self {
            wavefunction,
            barrier,
            potential,
            kinetic_prop,
            dt,
            time: 0.0,
            mass,
            initial_k: 0.0,
            transmission: 0.0,
            reflection: 0.0,
        }
    }

    /// Initialize with a Gaussian wave packet
    pub fn init_wave_packet(&mut self, x0: f32, k0: f32, sigma: f32) {
        self.wavefunction.gaussian_packet(x0, k0, sigma);
        self.wavefunction.normalize();
        self.initial_k = k0;
        self.time = 0.0;
    }

    /// Single time step using split-operator method
    /// This is a simplified version using finite differences
    pub fn step(&mut self) {
        let n = self.wavefunction.len();
        let dx = self.wavefunction.dx;
        let dt = self.dt;

        // Half step in potential (position space)
        for i in 0..n {
            let phase = -self.potential[i] * dt / (2.0 * HBAR);
            self.wavefunction.psi[i] = self.wavefunction.psi[i] * Complex::exp_i(phase);
        }

        // Full step in kinetic energy using finite differences (Crank-Nicolson-like)
        let alpha = Complex::new(0.0, HBAR * dt / (4.0 * self.mass * dx * dx));
        let mut new_psi = vec![Complex::ZERO; n];

        for i in 1..n - 1 {
            let laplacian = self.wavefunction.psi[i + 1]
                + self.wavefunction.psi[i - 1]
                - self.wavefunction.psi[i] * 2.0;
            new_psi[i] = self.wavefunction.psi[i] + alpha * laplacian;
        }

        // Absorbing boundary conditions
        new_psi[0] = self.wavefunction.psi[0] * 0.99;
        new_psi[n - 1] = self.wavefunction.psi[n - 1] * 0.99;

        self.wavefunction.psi = new_psi;

        // Half step in potential again
        for i in 0..n {
            let phase = -self.potential[i] * dt / (2.0 * HBAR);
            self.wavefunction.psi[i] = self.wavefunction.psi[i] * Complex::exp_i(phase);
        }

        self.time += dt;

        // Update transmission/reflection coefficients
        self.compute_coefficients();
    }

    /// Compute transmission and reflection coefficients
    fn compute_coefficients(&mut self) {
        let n = self.wavefunction.len();
        let dx = self.wavefunction.dx;

        // Find barrier center
        let barrier_x = match self.barrier {
            Barrier::Rectangular { center, .. } => center,
            Barrier::Gaussian { center, .. } => center,
            Barrier::Double { center, .. } => center,
            Barrier::Step { position, .. } => position,
        };

        // Integrate probability on each side
        let mut left_prob = 0.0;
        let mut right_prob = 0.0;

        for i in 0..n {
            let x = self.wavefunction.x_at(i);
            let prob = self.wavefunction.psi[i].norm_sq() * dx;
            if x < barrier_x {
                left_prob += prob;
            } else {
                right_prob += prob;
            }
        }

        // If initial wave started on left, right is transmitted
        let total = left_prob + right_prob;
        if total > 1e-10 {
            self.transmission = right_prob / total;
            self.reflection = left_prob / total;
        }
    }

    /// Get probability density for visualization
    pub fn probability_density(&self) -> Vec<f32> {
        self.wavefunction.probability_density()
    }

    /// Get potential profile
    pub fn potential_profile(&self) -> &[f32] {
        &self.potential
    }

    /// Get particle position data for rendering
    pub fn get_render_data(&self) -> Vec<(f32, f32, f32, [f32; 4])> {
        let density = self.probability_density();
        let n = self.wavefunction.len();
        let mut data = Vec::with_capacity(n);

        for i in 0..n {
            let x = self.wavefunction.x_at(i);
            let prob = density[i];
            let phase = self.wavefunction.psi[i].arg();

            // Color based on phase (hue) and probability (brightness)
            let hue = (phase + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);

            data.push((x, prob, self.potential[i], [r, g, b, prob.sqrt()]));
        }

        data
    }

    /// Theoretical transmission coefficient for rectangular barrier
    pub fn theoretical_transmission(&self) -> Option<f32> {
        if let Barrier::Rectangular { height, width, .. } = self.barrier {
            let e = HBAR * HBAR * self.initial_k * self.initial_k / (2.0 * self.mass);
            if e < height {
                // Tunneling regime: T ≈ exp(-2κa) where κ = sqrt(2m(V-E))/ℏ
                let kappa = (2.0 * self.mass * (height - e)).sqrt() / HBAR;
                Some((-2.0 * kappa * width).exp())
            } else {
                // Over-barrier: oscillating transmission
                Some(1.0)
            }
        } else {
            None
        }
    }

    /// Reset simulation with new parameters
    pub fn reset(&mut self, x0: f32, k0: f32, sigma: f32) {
        self.init_wave_packet(x0, k0, sigma);
        self.transmission = 0.0;
        self.reflection = 0.0;
    }

    /// Set barrier type
    pub fn set_barrier(&mut self, barrier: Barrier) {
        self.barrier = barrier;
        let n = self.wavefunction.len();
        let dx = self.wavefunction.dx;
        let x_min = self.wavefunction.x_min;

        self.potential = (0..n)
            .map(|i| {
                let x = x_min + i as f32 * dx;
                barrier.potential_at(x)
            })
            .collect();
    }
}

/// Preset tunneling scenarios
impl TunnelingSimulation {
    /// Standard single barrier
    pub fn preset_single_barrier() -> Self {
        let barrier = Barrier::Rectangular {
            height: 15.0,
            width: 0.5,
            center: 0.0,
        };
        let mut sim = Self::new(512, -10.0, 10.0, barrier);
        sim.init_wave_packet(-5.0, 5.0, 0.5);
        sim
    }

    /// Double barrier for resonant tunneling
    pub fn preset_resonant_tunneling() -> Self {
        let barrier = Barrier::Double {
            height: 20.0,
            width: 0.3,
            spacing: 1.0,
            center: 0.0,
        };
        let mut sim = Self::new(512, -10.0, 10.0, barrier);
        sim.init_wave_packet(-5.0, 4.0, 0.5);
        sim
    }

    /// Step potential for reflection/transmission
    pub fn preset_step_potential() -> Self {
        let barrier = Barrier::Step {
            height: 10.0,
            position: 0.0,
        };
        let mut sim = Self::new(512, -10.0, 10.0, barrier);
        sim.init_wave_packet(-5.0, 6.0, 0.5);
        sim
    }
}

/// Helper: HSV to RGB conversion
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
