//! Atomic Orbital Visualization
//!
//! Renders 3D probability clouds for hydrogen-like atomic orbitals,
//! showing the spatial distribution of electron wavefunctions.

use crate::wavefunction::{Complex, hydrogen_radial, spherical_harmonic};

/// Bohr radius (scaled)
const A0: f32 = 1.0;
use glam::Vec3;
use rand::Rng;
use std::f32::consts::PI;

/// Quantum numbers for an orbital
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantumNumbers {
    /// Principal quantum number n (1, 2, 3, ...)
    pub n: u32,
    /// Angular momentum quantum number l (0 to n-1)
    pub l: u32,
    /// Magnetic quantum number m (-l to +l)
    pub m: i32,
}

impl QuantumNumbers {
    pub fn new(n: u32, l: u32, m: i32) -> Option<Self> {
        if l >= n || m.abs() as u32 > l {
            None
        } else {
            Some(Self { n, l, m })
        }
    }

    /// Orbital name (1s, 2p, 3d, etc.)
    pub fn name(&self) -> String {
        let l_char = match self.l {
            0 => 's',
            1 => 'p',
            2 => 'd',
            3 => 'f',
            _ => 'g',
        };
        format!("{}{}{:+}", self.n, l_char, self.m)
    }

    /// Common orbital presets
    pub fn s1() -> Self { Self { n: 1, l: 0, m: 0 } }
    pub fn s2() -> Self { Self { n: 2, l: 0, m: 0 } }
    pub fn p2_0() -> Self { Self { n: 2, l: 1, m: 0 } }
    pub fn p2_1() -> Self { Self { n: 2, l: 1, m: 1 } }
    pub fn p2_m1() -> Self { Self { n: 2, l: 1, m: -1 } }
    pub fn s3() -> Self { Self { n: 3, l: 0, m: 0 } }
    pub fn p3_0() -> Self { Self { n: 3, l: 1, m: 0 } }
    pub fn d3_0() -> Self { Self { n: 3, l: 2, m: 0 } }
    pub fn d3_1() -> Self { Self { n: 3, l: 2, m: 1 } }
    pub fn d3_2() -> Self { Self { n: 3, l: 2, m: 2 } }
}

/// A point in the probability cloud
#[derive(Debug, Clone, Copy)]
pub struct CloudPoint {
    pub position: Vec3,
    pub probability: f32,
    pub phase: f32,
    pub color: [f32; 4],
}

/// Orbital probability cloud simulation
pub struct OrbitalSimulation {
    /// Current quantum numbers
    pub quantum_numbers: QuantumNumbers,
    /// Sample points representing the probability cloud
    pub points: Vec<CloudPoint>,
    /// Number of sample points
    pub num_points: usize,
    /// Bohr radius scale
    pub scale: f32,
    /// Time for animation
    pub time: f32,
    /// Show phase animation
    pub animate_phase: bool,
    /// Cross-section mode
    pub cross_section: Option<CrossSection>,
}

/// Cross-section plane for viewing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossSection {
    XY(f32), // z = constant
    XZ(f32), // y = constant
    YZ(f32), // x = constant
}

impl OrbitalSimulation {
    pub fn new(quantum_numbers: QuantumNumbers, num_points: usize) -> Self {
        let mut sim = Self {
            quantum_numbers,
            points: Vec::new(),
            num_points,
            scale: A0 * 3.0,
            time: 0.0,
            animate_phase: true,
            cross_section: None,
        };
        sim.regenerate_points();
        sim
    }

    /// Compute wavefunction at a point
    pub fn wavefunction_at(&self, pos: Vec3) -> Complex {
        let r = pos.length();
        if r < 1e-6 {
            return Complex::ZERO;
        }

        let theta = (pos.z / r).acos();
        let phi = pos.y.atan2(pos.x);

        let radial = hydrogen_radial(
            self.quantum_numbers.n,
            self.quantum_numbers.l,
            r / self.scale,
            1.0,
        );

        let angular = spherical_harmonic(
            self.quantum_numbers.l as i32,
            self.quantum_numbers.m,
            theta,
            phi,
        );

        angular * radial
    }

    /// Probability density |ψ|² at a point
    pub fn probability_at(&self, pos: Vec3) -> f32 {
        self.wavefunction_at(pos).norm_sq()
    }

    /// Generate sample points using rejection sampling
    pub fn regenerate_points(&mut self) {
        self.points.clear();
        let mut rng = rand::thread_rng();

        // Estimate maximum probability for rejection sampling
        let max_r = self.scale * self.quantum_numbers.n as f32 * 5.0;
        let mut max_prob = 0.0f32;

        // Sample to find approximate max
        for _ in 0..1000 {
            let r = rng.gen::<f32>() * max_r;
            let theta = rng.gen::<f32>() * PI;
            let phi = rng.gen::<f32>() * 2.0 * PI;
            let pos = Vec3::new(
                r * theta.sin() * phi.cos(),
                r * theta.sin() * phi.sin(),
                r * theta.cos(),
            );
            let prob = self.probability_at(pos);
            max_prob = max_prob.max(prob);
        }
        max_prob *= 1.5; // Safety margin

        // Rejection sampling
        let target_points = self.num_points;
        let mut attempts = 0;
        let max_attempts = target_points * 100;

        while self.points.len() < target_points && attempts < max_attempts {
            attempts += 1;

            // Sample in spherical coordinates with r² weighting
            let u = rng.gen::<f32>();
            let r = max_r * u.cbrt(); // r² dr weighting
            let theta = (1.0 - 2.0 * rng.gen::<f32>()).acos();
            let phi = rng.gen::<f32>() * 2.0 * PI;

            let pos = Vec3::new(
                r * theta.sin() * phi.cos(),
                r * theta.sin() * phi.sin(),
                r * theta.cos(),
            );

            // Apply cross-section filter if enabled
            if let Some(cs) = self.cross_section {
                let passes = match cs {
                    CrossSection::XY(z) => (pos.z - z).abs() < self.scale * 0.2,
                    CrossSection::XZ(y) => (pos.y - y).abs() < self.scale * 0.2,
                    CrossSection::YZ(x) => (pos.x - x).abs() < self.scale * 0.2,
                };
                if !passes {
                    continue;
                }
            }

            let psi = self.wavefunction_at(pos);
            let prob = psi.norm_sq();
            let phase = psi.arg();

            // Rejection sampling
            if rng.gen::<f32>() * max_prob < prob {
                // Color based on phase
                let hue = (phase + PI) / (2.0 * PI);
                let (r, g, b) = hsv_to_rgb(hue, 0.8, 1.0);

                self.points.push(CloudPoint {
                    position: pos,
                    probability: prob,
                    phase,
                    color: [r, g, b, 0.6],
                });
            }
        }
    }

    /// Update animation
    pub fn step(&mut self, dt: f32) {
        if self.animate_phase {
            self.time += dt;

            // Rotate phase colors
            let omega = 2.0; // Angular frequency
            for point in &mut self.points {
                let animated_phase = point.phase + omega * self.time;
                let hue = (animated_phase + PI) / (2.0 * PI);
                let hue = hue.rem_euclid(1.0);
                let (r, g, b) = hsv_to_rgb(hue, 0.8, 1.0);
                point.color = [r, g, b, 0.6];
            }
        }
    }

    /// Set new quantum numbers and regenerate
    pub fn set_orbital(&mut self, qn: QuantumNumbers) {
        if qn != self.quantum_numbers {
            self.quantum_numbers = qn;
            self.regenerate_points();
        }
    }

    /// Toggle cross-section mode
    pub fn set_cross_section(&mut self, cs: Option<CrossSection>) {
        if self.cross_section != cs {
            self.cross_section = cs;
            self.regenerate_points();
        }
    }

    /// Get radial probability distribution P(r) = r²|R(r)|²
    pub fn radial_distribution(&self, num_points: usize) -> Vec<(f32, f32)> {
        let max_r = self.scale * self.quantum_numbers.n as f32 * 5.0;
        let dr = max_r / num_points as f32;

        (0..num_points)
            .map(|i| {
                let r = i as f32 * dr;
                let radial = hydrogen_radial(
                    self.quantum_numbers.n,
                    self.quantum_numbers.l,
                    r / self.scale,
                    1.0,
                );
                let prob = r * r * radial * radial;
                (r, prob)
            })
            .collect()
    }

    /// Get render data for GPU
    pub fn get_render_data(&self) -> Vec<(Vec3, f32, [f32; 4])> {
        self.points
            .iter()
            .map(|p| (p.position, p.probability, p.color))
            .collect()
    }
}

/// Common orbital presets
impl OrbitalSimulation {
    pub fn preset_1s() -> Self {
        Self::new(QuantumNumbers::s1(), 5000)
    }

    pub fn preset_2s() -> Self {
        Self::new(QuantumNumbers::s2(), 5000)
    }

    pub fn preset_2p() -> Self {
        Self::new(QuantumNumbers::p2_0(), 5000)
    }

    pub fn preset_3d() -> Self {
        Self::new(QuantumNumbers::d3_0(), 8000)
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
