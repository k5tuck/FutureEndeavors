//! Quantum Teleportation Simulation
//!
//! Demonstrates the quantum teleportation protocol using entangled Bell pairs.
//! Visualizes the transfer of quantum state from Alice to Bob.

use crate::quantum_state::{Qubit, TwoQubit, ThreeQubit};
use crate::wavefunction::Complex;
use glam::Vec3;
use rand::Rng;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

/// Stage in the teleportation protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportationStage {
    /// Initial state: Alice has qubit to teleport, shared Bell pair
    Initial,
    /// Alice applies CNOT between her qubits
    AliceCNOT,
    /// Alice applies Hadamard to her original qubit
    AliceHadamard,
    /// Alice measures both her qubits
    AliceMeasure,
    /// Classical communication of measurement results
    ClassicalChannel,
    /// Bob applies corrections based on Alice's results
    BobCorrection,
    /// Teleportation complete
    Complete,
}

/// Visual representation of a qubit for rendering
#[derive(Debug, Clone)]
pub struct QubitVisual {
    pub position: Vec3,
    pub bloch_vector: Vec3,
    pub label: String,
    pub color: [f32; 4],
    pub measured: bool,
    pub measurement_result: Option<bool>,
}

impl QubitVisual {
    pub fn new(position: Vec3, label: &str, color: [f32; 4]) -> Self {
        Self {
            position,
            bloch_vector: Vec3::new(0.0, 0.0, 1.0), // |0⟩ state
            label: label.to_string(),
            color,
            measured: false,
            measurement_result: None,
        }
    }

    pub fn from_qubit(qubit: &Qubit, position: Vec3, label: &str, color: [f32; 4]) -> Self {
        let (x, y, z) = qubit.bloch_vector();
        Self {
            position,
            bloch_vector: Vec3::new(x, y, z),
            label: label.to_string(),
            color,
            measured: false,
            measurement_result: None,
        }
    }
}

/// Entanglement connection for visualization
#[derive(Debug, Clone)]
pub struct EntanglementLink {
    pub qubit_a: usize,
    pub qubit_b: usize,
    pub strength: f32, // 0 to 1, based on concurrence
    pub color: [f32; 4],
}

/// Teleportation simulation
pub struct TeleportationSimulation {
    /// Current stage of the protocol
    pub stage: TeleportationStage,

    /// The state to teleport (Alice's original qubit)
    pub state_to_teleport: Qubit,

    /// Full three-qubit state
    /// Qubit 0: Alice's original (to teleport)
    /// Qubit 1: Alice's half of Bell pair
    /// Qubit 2: Bob's half of Bell pair
    pub three_qubit: ThreeQubit,

    /// Alice's measurement results (bit 0: first qubit, bit 1: second)
    pub alice_results: Option<(bool, bool)>,

    /// Visual representations
    pub qubits: Vec<QubitVisual>,

    /// Entanglement connections
    pub entanglement_links: Vec<EntanglementLink>,

    /// Animation time
    pub time: f32,

    /// Stage progress for animations
    pub stage_progress: f32,

    /// Fidelity of teleportation
    pub fidelity: f32,
}

impl TeleportationSimulation {
    pub fn new() -> Self {
        let state_to_teleport = Qubit::from_bloch(PI / 3.0, PI / 4.0); // Arbitrary state

        // Initialize visual representations
        let qubits = vec![
            QubitVisual::from_qubit(
                &state_to_teleport,
                Vec3::new(-3.0, 0.0, 0.0),
                "ψ (Alice)",
                [0.2, 0.8, 0.2, 1.0],
            ),
            QubitVisual::new(
                Vec3::new(-1.0, 0.0, 0.0),
                "A (Alice's Bell)",
                [0.8, 0.2, 0.2, 1.0],
            ),
            QubitVisual::new(
                Vec3::new(3.0, 0.0, 0.0),
                "B (Bob's Bell)",
                [0.2, 0.2, 0.8, 1.0],
            ),
        ];

        let entanglement_links = vec![EntanglementLink {
            qubit_a: 1,
            qubit_b: 2,
            strength: 1.0,
            color: [1.0, 0.5, 1.0, 0.8],
        }];

        let mut sim = Self {
            stage: TeleportationStage::Initial,
            state_to_teleport,
            three_qubit: ThreeQubit::ground(),
            alice_results: None,
            qubits,
            entanglement_links,
            time: 0.0,
            stage_progress: 0.0,
            fidelity: 0.0,
        };

        sim.initialize_state();
        sim
    }

    /// Initialize the quantum state: |ψ⟩ ⊗ |Φ+⟩
    fn initialize_state(&mut self) {
        // Create |ψ⟩ ⊗ |00⟩
        // Then entangle qubits 1 and 2 to create Bell pair

        let alpha = self.state_to_teleport.alpha;
        let beta = self.state_to_teleport.beta;

        // Initial: |ψ⟩|00⟩ = α|000⟩ + β|100⟩
        self.three_qubit.amplitudes = [
            alpha,          // |000⟩
            Complex::ZERO,  // |001⟩
            Complex::ZERO,  // |010⟩
            Complex::ZERO,  // |011⟩
            beta,           // |100⟩
            Complex::ZERO,  // |101⟩
            Complex::ZERO,  // |110⟩
            Complex::ZERO,  // |111⟩
        ];

        // Apply H to qubit 1: |0⟩ → |+⟩
        self.three_qubit.hadamard(1);

        // Apply CNOT(1,2) to create Bell pair between qubits 1 and 2
        self.three_qubit.cnot(1, 2);

        // Now state is: α(|000⟩+|011⟩)/√2 + β(|100⟩+|111⟩)/√2
        // = (α|0⟩+β|1⟩) ⊗ (|00⟩+|11⟩)/√2

        self.update_visuals();
    }

    /// Advance to next stage of the protocol
    pub fn next_stage(&mut self) {
        self.stage_progress = 0.0;

        match self.stage {
            TeleportationStage::Initial => {
                // Apply CNOT with qubit 0 as control, qubit 1 as target
                self.three_qubit.cnot(0, 1);
                self.stage = TeleportationStage::AliceCNOT;
            }
            TeleportationStage::AliceCNOT => {
                // Apply Hadamard to qubit 0
                self.three_qubit.hadamard(0);
                self.stage = TeleportationStage::AliceHadamard;
            }
            TeleportationStage::AliceHadamard => {
                // Measure qubits 0 and 1
                self.perform_measurement();
                self.stage = TeleportationStage::AliceMeasure;
            }
            TeleportationStage::AliceMeasure => {
                self.stage = TeleportationStage::ClassicalChannel;
            }
            TeleportationStage::ClassicalChannel => {
                // Apply corrections based on measurement
                self.apply_corrections();
                self.stage = TeleportationStage::BobCorrection;
            }
            TeleportationStage::BobCorrection => {
                self.compute_fidelity();
                self.stage = TeleportationStage::Complete;
            }
            TeleportationStage::Complete => {
                // Reset
                self.reset();
            }
        }

        self.update_visuals();
    }

    /// Perform Alice's measurement
    fn perform_measurement(&mut self) {
        let probs = self.three_qubit.probabilities();
        let mut rng = rand::thread_rng();
        let r: f32 = rng.gen();

        // Compute marginal probabilities for qubits 0 and 1
        // Sum over qubit 2 (Bob's)
        let p00 = probs[0] + probs[1]; // |00x⟩
        let p01 = probs[2] + probs[3]; // |01x⟩
        let p10 = probs[4] + probs[5]; // |10x⟩
        let p11 = probs[6] + probs[7]; // |11x⟩

        // Determine measurement outcome
        let (m0, m1) = if r < p00 {
            (false, false)
        } else if r < p00 + p01 {
            (false, true)
        } else if r < p00 + p01 + p10 {
            (true, false)
        } else {
            (true, true)
        };

        self.alice_results = Some((m0, m1));

        // Update visual states
        self.qubits[0].measured = true;
        self.qubits[0].measurement_result = Some(m0);
        self.qubits[0].bloch_vector = if m0 { Vec3::NEG_Z } else { Vec3::Z };

        self.qubits[1].measured = true;
        self.qubits[1].measurement_result = Some(m1);
        self.qubits[1].bloch_vector = if m1 { Vec3::NEG_Z } else { Vec3::Z };

        // Collapse the state
        // After measurement, only amplitudes consistent with outcome survive
        let mask = (if m0 { 4 } else { 0 }) | (if m1 { 2 } else { 0 });

        for i in 0..8 {
            if (i & 6) != mask {
                self.three_qubit.amplitudes[i] = Complex::ZERO;
            }
        }
        self.three_qubit.normalize();
    }

    /// Apply Bob's corrections based on Alice's measurement
    fn apply_corrections(&mut self) {
        if let Some((m0, m1)) = self.alice_results {
            // If m1 = 1, apply X to Bob's qubit
            if m1 {
                // X on qubit 2: swap |xy0⟩ ↔ |xy1⟩
                for i in 0..4 {
                    let j = 2 * i;
                    self.three_qubit.amplitudes.swap(j, j + 1);
                }
            }

            // If m0 = 1, apply Z to Bob's qubit
            if m0 {
                // Z on qubit 2: |xy1⟩ → -|xy1⟩
                for i in 0..4 {
                    self.three_qubit.amplitudes[2 * i + 1] =
                        self.three_qubit.amplitudes[2 * i + 1] * (-1.0);
                }
            }
        }
    }

    /// Compute teleportation fidelity
    fn compute_fidelity(&mut self) {
        // Bob's final state should match the original state to teleport
        // Extract Bob's qubit state from the three-qubit state

        // Since qubits 0,1 are measured, Bob's state is the remaining superposition
        // For |ab⟩ measurement outcome, Bob's state is in amplitudes with that prefix

        if let Some((m0, m1)) = self.alice_results {
            let offset = (if m0 { 4 } else { 0 }) + (if m1 { 2 } else { 0 });

            // Bob's qubit coefficients
            let bob_alpha = self.three_qubit.amplitudes[offset];     // |ab0⟩
            let bob_beta = self.three_qubit.amplitudes[offset + 1];  // |ab1⟩

            // Fidelity = |⟨ψ_original|ψ_bob⟩|²
            let overlap = self.state_to_teleport.alpha.conj() * bob_alpha
                + self.state_to_teleport.beta.conj() * bob_beta;
            self.fidelity = overlap.norm_sq();

            // Update Bob's visual
            let bob_qubit = Qubit {
                alpha: bob_alpha,
                beta: bob_beta,
            };
            let (x, y, z) = bob_qubit.bloch_vector();
            self.qubits[2].bloch_vector = Vec3::new(x, y, z);
            self.qubits[2].color = self.qubits[0].color; // Same color as original
        }
    }

    /// Update visual representations based on current state
    fn update_visuals(&mut self) {
        // Update entanglement visualization
        match self.stage {
            TeleportationStage::Initial => {
                // Bell pair between qubits 1 and 2
                self.entanglement_links = vec![EntanglementLink {
                    qubit_a: 1,
                    qubit_b: 2,
                    strength: 1.0,
                    color: [1.0, 0.5, 1.0, 0.8],
                }];
            }
            TeleportationStage::AliceCNOT | TeleportationStage::AliceHadamard => {
                // Full three-way entanglement
                self.entanglement_links = vec![
                    EntanglementLink {
                        qubit_a: 0,
                        qubit_b: 1,
                        strength: 0.7,
                        color: [0.5, 1.0, 0.5, 0.6],
                    },
                    EntanglementLink {
                        qubit_a: 1,
                        qubit_b: 2,
                        strength: 0.7,
                        color: [1.0, 0.5, 1.0, 0.6],
                    },
                    EntanglementLink {
                        qubit_a: 0,
                        qubit_b: 2,
                        strength: 0.5,
                        color: [0.5, 0.5, 1.0, 0.4],
                    },
                ];
            }
            TeleportationStage::AliceMeasure | TeleportationStage::ClassicalChannel => {
                // Entanglement broken by measurement
                self.entanglement_links.clear();
            }
            TeleportationStage::BobCorrection | TeleportationStage::Complete => {
                // State transferred to Bob
                self.entanglement_links.clear();
            }
        }
    }

    /// Animation update
    pub fn step(&mut self, dt: f32) {
        self.time += dt;
        self.stage_progress = (self.stage_progress + dt * 0.5).min(1.0);

        // Animate qubit positions during classical channel stage
        if self.stage == TeleportationStage::ClassicalChannel {
            // Animate "classical bits" traveling from Alice to Bob
        }
    }

    /// Reset to initial state with optional new state to teleport
    pub fn reset(&mut self) {
        self.stage = TeleportationStage::Initial;
        self.alice_results = None;
        self.fidelity = 0.0;
        self.time = 0.0;
        self.stage_progress = 0.0;

        // Reset visual states
        for qubit in &mut self.qubits {
            qubit.measured = false;
            qubit.measurement_result = None;
        }

        let (x, y, z) = self.state_to_teleport.bloch_vector();
        self.qubits[0].bloch_vector = Vec3::new(x, y, z);
        self.qubits[1].bloch_vector = Vec3::Z;
        self.qubits[2].bloch_vector = Vec3::Z;

        self.initialize_state();
    }

    /// Set a new state to teleport
    pub fn set_state_to_teleport(&mut self, theta: f32, phi: f32) {
        self.state_to_teleport = Qubit::from_bloch(theta, phi);
        self.reset();
    }

    /// Get description of current stage
    pub fn stage_description(&self) -> &'static str {
        match self.stage {
            TeleportationStage::Initial =>
                "Initial: Alice has |ψ⟩, shares Bell pair |Φ+⟩ with Bob",
            TeleportationStage::AliceCNOT =>
                "Alice applies CNOT between her qubits",
            TeleportationStage::AliceHadamard =>
                "Alice applies Hadamard to qubit |ψ⟩",
            TeleportationStage::AliceMeasure =>
                "Alice measures both her qubits",
            TeleportationStage::ClassicalChannel =>
                "Alice sends measurement results to Bob (classical)",
            TeleportationStage::BobCorrection =>
                "Bob applies corrections based on Alice's results",
            TeleportationStage::Complete =>
                "Teleportation complete! Bob now has |ψ⟩",
        }
    }
}

impl Default for TeleportationSimulation {
    fn default() -> Self {
        Self::new()
    }
}
