//! Quantum state representations
//!
//! Provides qubit states, density matrices, and entanglement

use crate::wavefunction::Complex;
use std::f32::consts::FRAC_1_SQRT_2;

/// Single qubit state |ψ⟩ = α|0⟩ + β|1⟩
#[derive(Debug, Clone, Copy)]
pub struct Qubit {
    pub alpha: Complex, // Coefficient of |0⟩
    pub beta: Complex,  // Coefficient of |1⟩
}

impl Qubit {
    /// |0⟩ state
    pub const ZERO: Qubit = Qubit {
        alpha: Complex::ONE,
        beta: Complex::ZERO,
    };

    /// |1⟩ state
    pub const ONE: Qubit = Qubit {
        alpha: Complex::ZERO,
        beta: Complex::ONE,
    };

    /// |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn plus() -> Self {
        Self {
            alpha: Complex::new(FRAC_1_SQRT_2, 0.0),
            beta: Complex::new(FRAC_1_SQRT_2, 0.0),
        }
    }

    /// |-⟩ = (|0⟩ - |1⟩)/√2
    pub fn minus() -> Self {
        Self {
            alpha: Complex::new(FRAC_1_SQRT_2, 0.0),
            beta: Complex::new(-FRAC_1_SQRT_2, 0.0),
        }
    }

    /// Create from Bloch sphere angles
    pub fn from_bloch(theta: f32, phi: f32) -> Self {
        Self {
            alpha: Complex::new((theta / 2.0).cos(), 0.0),
            beta: Complex::from_polar((theta / 2.0).sin(), phi),
        }
    }

    /// Get Bloch sphere coordinates
    pub fn bloch_vector(&self) -> (f32, f32, f32) {
        // x = 2*Re(α*β*)
        // y = 2*Im(α*β*)
        // z = |α|² - |β|²
        let ab_star = self.alpha * self.beta.conj();
        (
            2.0 * ab_star.re,
            2.0 * ab_star.im,
            self.alpha.norm_sq() - self.beta.norm_sq(),
        )
    }

    /// Probability of measuring |0⟩
    pub fn prob_zero(&self) -> f32 {
        self.alpha.norm_sq()
    }

    /// Probability of measuring |1⟩
    pub fn prob_one(&self) -> f32 {
        self.beta.norm_sq()
    }

    /// Apply Hadamard gate
    pub fn hadamard(&mut self) {
        let new_alpha = (self.alpha + self.beta) * FRAC_1_SQRT_2;
        let new_beta = (self.alpha - self.beta) * FRAC_1_SQRT_2;
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Apply Pauli-X gate (NOT)
    pub fn pauli_x(&mut self) {
        std::mem::swap(&mut self.alpha, &mut self.beta);
    }

    /// Apply Pauli-Y gate
    pub fn pauli_y(&mut self) {
        let new_alpha = self.beta * Complex::new(0.0, -1.0);
        let new_beta = self.alpha * Complex::new(0.0, 1.0);
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Apply Pauli-Z gate
    pub fn pauli_z(&mut self) {
        self.beta = self.beta * (-1.0);
    }

    /// Apply phase gate S
    pub fn phase_s(&mut self) {
        self.beta = self.beta * Complex::I;
    }

    /// Apply T gate (π/8 gate)
    pub fn t_gate(&mut self) {
        self.beta = self.beta * Complex::exp_i(std::f32::consts::FRAC_PI_4);
    }

    /// Normalize the state
    pub fn normalize(&mut self) {
        let norm = (self.alpha.norm_sq() + self.beta.norm_sq()).sqrt();
        if norm > 1e-10 {
            self.alpha = self.alpha * (1.0 / norm);
            self.beta = self.beta * (1.0 / norm);
        }
    }
}

impl Default for Qubit {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Two-qubit state
#[derive(Debug, Clone)]
pub struct TwoQubit {
    /// Coefficients for |00⟩, |01⟩, |10⟩, |11⟩
    pub amplitudes: [Complex; 4],
}

impl TwoQubit {
    /// Product state |00⟩
    pub fn zero_zero() -> Self {
        Self {
            amplitudes: [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
        }
    }

    /// Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
    pub fn bell_phi_plus() -> Self {
        Self {
            amplitudes: [
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::ZERO,
                Complex::ZERO,
                Complex::new(FRAC_1_SQRT_2, 0.0),
            ],
        }
    }

    /// Bell state |Φ-⟩ = (|00⟩ - |11⟩)/√2
    pub fn bell_phi_minus() -> Self {
        Self {
            amplitudes: [
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::ZERO,
                Complex::ZERO,
                Complex::new(-FRAC_1_SQRT_2, 0.0),
            ],
        }
    }

    /// Bell state |Ψ+⟩ = (|01⟩ + |10⟩)/√2
    pub fn bell_psi_plus() -> Self {
        Self {
            amplitudes: [
                Complex::ZERO,
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::ZERO,
            ],
        }
    }

    /// Bell state |Ψ-⟩ = (|01⟩ - |10⟩)/√2 (singlet state)
    pub fn bell_psi_minus() -> Self {
        Self {
            amplitudes: [
                Complex::ZERO,
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::new(-FRAC_1_SQRT_2, 0.0),
                Complex::ZERO,
            ],
        }
    }

    /// Create from two separable qubits
    pub fn from_product(q1: &Qubit, q2: &Qubit) -> Self {
        Self {
            amplitudes: [
                q1.alpha * q2.alpha, // |00⟩
                q1.alpha * q2.beta,  // |01⟩
                q1.beta * q2.alpha,  // |10⟩
                q1.beta * q2.beta,   // |11⟩
            ],
        }
    }

    /// Apply CNOT gate (control on first qubit)
    pub fn cnot(&mut self) {
        // CNOT flips |10⟩ ↔ |11⟩
        self.amplitudes.swap(2, 3);
    }

    /// Apply Hadamard to first qubit
    pub fn hadamard_first(&mut self) {
        let [a00, a01, a10, a11] = self.amplitudes;
        self.amplitudes = [
            (a00 + a10) * FRAC_1_SQRT_2,
            (a01 + a11) * FRAC_1_SQRT_2,
            (a00 - a10) * FRAC_1_SQRT_2,
            (a01 - a11) * FRAC_1_SQRT_2,
        ];
    }

    /// Apply Hadamard to second qubit
    pub fn hadamard_second(&mut self) {
        let [a00, a01, a10, a11] = self.amplitudes;
        self.amplitudes = [
            (a00 + a01) * FRAC_1_SQRT_2,
            (a00 - a01) * FRAC_1_SQRT_2,
            (a10 + a11) * FRAC_1_SQRT_2,
            (a10 - a11) * FRAC_1_SQRT_2,
        ];
    }

    /// Get entanglement measure (simplified concurrence)
    pub fn concurrence(&self) -> f32 {
        // C = 2|α₀₀α₁₁ - α₀₁α₁₀|
        let [a00, a01, a10, a11] = self.amplitudes;
        let det = a00 * a11 - a01 * a10;
        2.0 * det.norm()
    }

    /// Probability of measuring each basis state
    pub fn probabilities(&self) -> [f32; 4] {
        [
            self.amplitudes[0].norm_sq(),
            self.amplitudes[1].norm_sq(),
            self.amplitudes[2].norm_sq(),
            self.amplitudes[3].norm_sq(),
        ]
    }

    /// Normalize the state
    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.amplitudes.iter().map(|c| c.norm_sq()).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-10 {
            for c in &mut self.amplitudes {
                *c = *c * (1.0 / norm);
            }
        }
    }
}

/// Three-qubit state (for GHZ states and teleportation)
#[derive(Debug, Clone)]
pub struct ThreeQubit {
    /// Coefficients for |000⟩ through |111⟩
    pub amplitudes: [Complex; 8],
}

impl ThreeQubit {
    /// Ground state |000⟩
    pub fn ground() -> Self {
        let mut amps = [Complex::ZERO; 8];
        amps[0] = Complex::ONE;
        Self { amplitudes: amps }
    }

    /// GHZ state (|000⟩ + |111⟩)/√2
    pub fn ghz() -> Self {
        let mut amps = [Complex::ZERO; 8];
        amps[0] = Complex::new(FRAC_1_SQRT_2, 0.0);
        amps[7] = Complex::new(FRAC_1_SQRT_2, 0.0);
        Self { amplitudes: amps }
    }

    /// W state (|001⟩ + |010⟩ + |100⟩)/√3
    pub fn w_state() -> Self {
        let mut amps = [Complex::ZERO; 8];
        let coeff = 1.0 / 3.0_f32.sqrt();
        amps[1] = Complex::new(coeff, 0.0); // |001⟩
        amps[2] = Complex::new(coeff, 0.0); // |010⟩
        amps[4] = Complex::new(coeff, 0.0); // |100⟩
        Self { amplitudes: amps }
    }

    /// Apply CNOT on qubits i (control) and j (target)
    pub fn cnot(&mut self, control: usize, target: usize) {
        let control_mask = 1 << (2 - control);
        let target_mask = 1 << (2 - target);

        for i in 0..8 {
            if (i & control_mask) != 0 {
                let j = i ^ target_mask;
                if i < j {
                    self.amplitudes.swap(i, j);
                }
            }
        }
    }

    /// Apply Hadamard to qubit i
    pub fn hadamard(&mut self, qubit: usize) {
        let mask = 1 << (2 - qubit);
        let mut new_amps = [Complex::ZERO; 8];

        for i in 0..8 {
            let partner = i ^ mask;
            if i < partner {
                new_amps[i] = (self.amplitudes[i] + self.amplitudes[partner]) * FRAC_1_SQRT_2;
                new_amps[partner] = (self.amplitudes[i] - self.amplitudes[partner]) * FRAC_1_SQRT_2;
            }
        }

        self.amplitudes = new_amps;
    }

    /// Probabilities of each basis state
    pub fn probabilities(&self) -> [f32; 8] {
        let mut probs = [0.0; 8];
        for i in 0..8 {
            probs[i] = self.amplitudes[i].norm_sq();
        }
        probs
    }

    /// Normalize
    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.amplitudes.iter().map(|c| c.norm_sq()).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-10 {
            for c in &mut self.amplitudes {
                *c = *c * (1.0 / norm);
            }
        }
    }
}

/// Spin state for spin-1/2 particles
#[derive(Debug, Clone, Copy)]
pub struct Spin {
    pub up: Complex,   // Coefficient of |↑⟩
    pub down: Complex, // Coefficient of |↓⟩
}

impl Spin {
    pub const UP: Spin = Spin {
        up: Complex::ONE,
        down: Complex::ZERO,
    };

    pub const DOWN: Spin = Spin {
        up: Complex::ZERO,
        down: Complex::ONE,
    };

    /// Spin pointing in direction (theta, phi) on Bloch sphere
    pub fn in_direction(theta: f32, phi: f32) -> Self {
        Self {
            up: Complex::new((theta / 2.0).cos(), 0.0),
            down: Complex::from_polar((theta / 2.0).sin(), phi),
        }
    }

    /// Expected value of spin along z-axis: ⟨σ_z⟩
    pub fn expectation_z(&self) -> f32 {
        self.up.norm_sq() - self.down.norm_sq()
    }

    /// Expected value of spin along x-axis: ⟨σ_x⟩
    pub fn expectation_x(&self) -> f32 {
        let cross = self.up * self.down.conj();
        2.0 * cross.re
    }

    /// Expected value of spin along y-axis: ⟨σ_y⟩
    pub fn expectation_y(&self) -> f32 {
        let cross = self.up * self.down.conj();
        2.0 * cross.im
    }
}
