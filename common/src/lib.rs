//! Common utilities for physics simulations
//!
//! This crate provides shared graphics setup, camera controls, and rendering utilities
//! used by both the gravity simulation and black hole simulation projects.

pub mod graphics;
pub mod camera;

pub use graphics::*;
pub use camera::*;

/// Physical constants used in simulations
pub mod constants {
    /// Gravitational constant (scaled for simulation)
    pub const G: f32 = 6.674e-11;

    /// Speed of light in m/s
    pub const C: f32 = 299_792_458.0;

    /// Schwarzschild radius coefficient: 2G/c²
    pub const SCHWARZSCHILD_COEFF: f32 = 2.0 * G / (C * C);
}
