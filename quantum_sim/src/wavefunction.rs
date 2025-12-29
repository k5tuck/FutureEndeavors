//! Wavefunction mathematics and operations
//!
//! Provides complex wavefunction representation and quantum operators

use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// Complex number representation for wavefunctions
#[derive(Debug, Clone, Copy, Default)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };
    pub const I: Complex = Complex { re: 0.0, im: 1.0 };

    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    /// Create from polar form: r * e^(i*theta)
    pub fn from_polar(r: f32, theta: f32) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Magnitude squared |z|^2 = probability density
    pub fn norm_sq(&self) -> f32 {
        self.re * self.re + self.im * self.im
    }

    /// Magnitude |z|
    pub fn norm(&self) -> f32 {
        self.norm_sq().sqrt()
    }

    /// Phase angle
    pub fn arg(&self) -> f32 {
        self.im.atan2(self.re)
    }

    /// Complex conjugate
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Complex exponential e^(i*x)
    pub fn exp_i(x: f32) -> Self {
        Self {
            re: x.cos(),
            im: x.sin(),
        }
    }

    /// Full complex exponential e^z
    pub fn exp(&self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Mul<f32> for Complex {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl std::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

/// 1D Wavefunction on a discrete grid
#[derive(Clone)]
pub struct Wavefunction1D {
    /// Complex amplitudes at each grid point
    pub psi: Vec<Complex>,
    /// Spatial extent
    pub x_min: f32,
    pub x_max: f32,
    /// Grid spacing
    pub dx: f32,
}

impl Wavefunction1D {
    pub fn new(n_points: usize, x_min: f32, x_max: f32) -> Self {
        let dx = (x_max - x_min) / (n_points - 1) as f32;
        Self {
            psi: vec![Complex::ZERO; n_points],
            x_min,
            x_max,
            dx,
        }
    }

    /// Initialize as a Gaussian wave packet
    pub fn gaussian_packet(&mut self, x0: f32, k0: f32, sigma: f32) {
        let norm = 1.0 / (sigma * (2.0 * PI).sqrt()).sqrt();
        for (i, c) in self.psi.iter_mut().enumerate() {
            let x = self.x_min + i as f32 * self.dx;
            let gaussian = (-(x - x0).powi(2) / (4.0 * sigma * sigma)).exp();
            let phase = Complex::exp_i(k0 * x);
            *c = phase * (norm * gaussian);
        }
    }

    /// Get probability density at each point
    pub fn probability_density(&self) -> Vec<f32> {
        self.psi.iter().map(|c| c.norm_sq()).collect()
    }

    /// Normalize the wavefunction
    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.psi.iter().map(|c| c.norm_sq()).sum::<f32>() * self.dx;
        let norm = norm_sq.sqrt();
        if norm > 1e-10 {
            for c in &mut self.psi {
                *c = *c * (1.0 / norm);
            }
        }
    }

    /// Get x coordinate at index
    pub fn x_at(&self, i: usize) -> f32 {
        self.x_min + i as f32 * self.dx
    }

    /// Number of grid points
    pub fn len(&self) -> usize {
        self.psi.len()
    }

    pub fn is_empty(&self) -> bool {
        self.psi.is_empty()
    }
}

/// 3D Wavefunction for orbital visualization
#[derive(Clone)]
pub struct Wavefunction3D {
    /// Complex amplitudes on 3D grid
    pub psi: Vec<Complex>,
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// Spatial bounds
    pub bounds: (Vec3, Vec3),
}

impl Wavefunction3D {
    pub fn new(nx: usize, ny: usize, nz: usize, min: Vec3, max: Vec3) -> Self {
        Self {
            psi: vec![Complex::ZERO; nx * ny * nz],
            nx,
            ny,
            nz,
            bounds: (min, max),
        }
    }

    /// Get index in flattened array
    fn index(&self, ix: usize, iy: usize, iz: usize) -> usize {
        ix + iy * self.nx + iz * self.nx * self.ny
    }

    /// Get/set value at grid point
    pub fn get(&self, ix: usize, iy: usize, iz: usize) -> Complex {
        self.psi[self.index(ix, iy, iz)]
    }

    pub fn set(&mut self, ix: usize, iy: usize, iz: usize, val: Complex) {
        let idx = self.index(ix, iy, iz);
        self.psi[idx] = val;
    }

    /// Grid spacing
    pub fn dx(&self) -> f32 {
        (self.bounds.1.x - self.bounds.0.x) / (self.nx - 1) as f32
    }

    pub fn dy(&self) -> f32 {
        (self.bounds.1.y - self.bounds.0.y) / (self.ny - 1) as f32
    }

    pub fn dz(&self) -> f32 {
        (self.bounds.1.z - self.bounds.0.z) / (self.nz - 1) as f32
    }

    /// Get position at grid indices
    pub fn position_at(&self, ix: usize, iy: usize, iz: usize) -> Vec3 {
        Vec3::new(
            self.bounds.0.x + ix as f32 * self.dx(),
            self.bounds.0.y + iy as f32 * self.dy(),
            self.bounds.0.z + iz as f32 * self.dz(),
        )
    }
}

/// Spherical harmonics Y_l^m(theta, phi)
pub fn spherical_harmonic(l: i32, m: i32, theta: f32, phi: f32) -> Complex {
    let m_abs = m.abs();

    // Associated Legendre polynomial P_l^|m|(cos(theta))
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    let plm = associated_legendre(l, m_abs, cos_theta, sin_theta);

    // Normalization factor
    let norm = spherical_harmonic_norm(l, m_abs);

    // Phase factor e^(i*m*phi)
    let phase = Complex::exp_i(m as f32 * phi);

    // Condon-Shortley phase for negative m
    let cs_phase = if m < 0 && m_abs % 2 == 1 { -1.0 } else { 1.0 };

    phase * (norm * plm * cs_phase)
}

/// Associated Legendre polynomial (simplified for low l)
fn associated_legendre(l: i32, m: i32, cos_theta: f32, sin_theta: f32) -> f32 {
    let sin_m = sin_theta.powi(m);

    match (l, m) {
        (0, 0) => 1.0,
        (1, 0) => cos_theta,
        (1, 1) => -sin_theta,
        (2, 0) => 0.5 * (3.0 * cos_theta * cos_theta - 1.0),
        (2, 1) => -3.0 * cos_theta * sin_theta,
        (2, 2) => 3.0 * sin_theta * sin_theta,
        (3, 0) => 0.5 * cos_theta * (5.0 * cos_theta * cos_theta - 3.0),
        (3, 1) => -1.5 * sin_theta * (5.0 * cos_theta * cos_theta - 1.0),
        (3, 2) => 15.0 * cos_theta * sin_theta * sin_theta,
        (3, 3) => -15.0 * sin_m * sin_theta,
        _ => sin_m, // Fallback
    }
}

/// Normalization constant for spherical harmonics
fn spherical_harmonic_norm(l: i32, m: i32) -> f32 {
    let l = l as f32;
    let m = m as f32;
    let num = (2.0 * l + 1.0) * factorial((l - m) as u32);
    let den = 4.0 * PI * factorial((l + m) as u32);
    (num / den).sqrt()
}

/// Simple factorial for small numbers
fn factorial(n: u32) -> f32 {
    (1..=n).map(|i| i as f32).product::<f32>().max(1.0)
}

/// Radial wavefunction for hydrogen-like atoms R_nl(r)
pub fn hydrogen_radial(n: u32, l: u32, r: f32, a0: f32) -> f32 {
    let rho = 2.0 * r / (n as f32 * a0);

    // Simplified radial functions for low n, l
    match (n, l) {
        (1, 0) => 2.0 * (-rho / 2.0).exp(), // 1s
        (2, 0) => (1.0 / (2.0 * 2.0_f32.sqrt())) * (1.0 - rho / 2.0) * (-rho / 2.0).exp(), // 2s
        (2, 1) => (1.0 / (2.0 * 6.0_f32.sqrt())) * rho * (-rho / 2.0).exp(), // 2p
        (3, 0) => (2.0 / (81.0 * 3.0_f32.sqrt())) * (27.0 - 18.0 * rho + 2.0 * rho * rho) * (-rho / 2.0).exp(), // 3s
        (3, 1) => (8.0 / (27.0 * 6.0_f32.sqrt())) * (1.0 - rho / 6.0) * rho * (-rho / 2.0).exp(), // 3p
        (3, 2) => (4.0 / (81.0 * 30.0_f32.sqrt())) * rho * rho * (-rho / 2.0).exp(), // 3d
        _ => (-r / a0).exp(), // Fallback
    }
}
