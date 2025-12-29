//! Quantum Mechanics Simulations
//!
//! This crate provides interactive visualizations of various quantum mechanical phenomena:
//!
//! - **Quantum Tunneling**: Wave packet transmission through potential barriers
//! - **Atomic Orbitals**: 3D probability cloud visualization of electron wavefunctions
//! - **Quantum Teleportation**: Bell state entanglement and quantum state transfer
//! - **Quarks & Hadrons**: Strong force and quark confinement simulation
//! - **Quantum Hall Effect**: Landau levels and edge states in 2D electron gases
//! - **4D Visualization**: Hypercube and tesseract projections into 3D space

pub mod wavefunction;
pub mod quantum_state;
pub mod tunneling;
pub mod orbitals;
pub mod teleportation;
pub mod quarks;
pub mod hall_effect;
pub mod hypercube;
pub mod renderer;
pub mod equations_ui;

/// Physical constants for quantum simulations
pub mod constants {
    /// Reduced Planck constant (scaled for visualization)
    pub const HBAR: f32 = 1.0;

    /// Electron mass (scaled)
    pub const M_E: f32 = 1.0;

    /// Elementary charge (scaled)
    pub const E: f32 = 1.0;

    /// Speed of light (scaled)
    pub const C: f32 = 10.0;

    /// Fine structure constant (approximately 1/137)
    pub const ALPHA: f32 = 1.0 / 137.0;

    /// Bohr radius (scaled)
    pub const A0: f32 = 1.0;

    /// Strong coupling constant (scaled for visualization)
    pub const ALPHA_S: f32 = 0.5;
}
