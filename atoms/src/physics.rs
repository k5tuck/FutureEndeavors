//! Atomic and molecular physics simulation
//!
//! Simulates atoms using:
//! - Coulomb's law for electrostatic interactions
//! - Lennard-Jones potential for van der Waals forces
//! - Simple covalent bonding model

use glam::Vec2;
use rand::Rng;
use std::f32::consts::PI;

/// Coulomb constant (scaled for visualization)
pub const K_COULOMB: f32 = 100.0;

/// Lennard-Jones parameters (scaled)
pub const LJ_EPSILON: f32 = 1.0;
pub const LJ_SIGMA: f32 = 0.5;

/// Bond formation distance threshold
pub const BOND_DISTANCE: f32 = 1.2;
pub const BOND_STRENGTH: f32 = 50.0;

/// Element types with their properties
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Hydrogen,
    Carbon,
    Nitrogen,
    Oxygen,
    Sodium,
    Chlorine,
}

impl Element {
    pub fn charge(&self) -> f32 {
        match self {
            Element::Hydrogen => 0.0,
            Element::Carbon => 0.0,
            Element::Nitrogen => 0.0,
            Element::Oxygen => 0.0,
            Element::Sodium => 1.0,   // Na+
            Element::Chlorine => -1.0, // Cl-
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            Element::Hydrogen => 1.0,
            Element::Carbon => 12.0,
            Element::Nitrogen => 14.0,
            Element::Oxygen => 16.0,
            Element::Sodium => 23.0,
            Element::Chlorine => 35.5,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            Element::Hydrogen => 0.25,
            Element::Carbon => 0.35,
            Element::Nitrogen => 0.32,
            Element::Oxygen => 0.30,
            Element::Sodium => 0.45,
            Element::Chlorine => 0.40,
        }
    }

    pub fn color(&self) -> [f32; 4] {
        match self {
            Element::Hydrogen => [1.0, 1.0, 1.0, 1.0],   // White
            Element::Carbon => [0.3, 0.3, 0.3, 1.0],     // Dark gray
            Element::Nitrogen => [0.2, 0.2, 0.8, 1.0],   // Blue
            Element::Oxygen => [0.8, 0.2, 0.2, 1.0],     // Red
            Element::Sodium => [0.8, 0.5, 0.8, 1.0],     // Purple
            Element::Chlorine => [0.2, 0.8, 0.2, 1.0],   // Green
        }
    }

    pub fn max_bonds(&self) -> usize {
        match self {
            Element::Hydrogen => 1,
            Element::Carbon => 4,
            Element::Nitrogen => 3,
            Element::Oxygen => 2,
            Element::Sodium => 1,
            Element::Chlorine => 1,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Element::Hydrogen => "H",
            Element::Carbon => "C",
            Element::Nitrogen => "N",
            Element::Oxygen => "O",
            Element::Sodium => "Na",
            Element::Chlorine => "Cl",
        }
    }
}

/// An atom in the simulation
#[derive(Debug, Clone)]
pub struct Atom {
    pub element: Element,
    pub position: Vec2,
    pub velocity: Vec2,
    pub bonds: Vec<usize>, // Indices of bonded atoms
    pub id: usize,
}

impl Atom {
    pub fn new(element: Element, position: Vec2, id: usize) -> Self {
        Self {
            element,
            position,
            velocity: Vec2::ZERO,
            bonds: Vec::new(),
            id,
        }
    }

    pub fn mass(&self) -> f32 {
        self.element.mass()
    }

    pub fn charge(&self) -> f32 {
        self.element.charge()
    }

    pub fn radius(&self) -> f32 {
        self.element.radius()
    }

    pub fn color(&self) -> [f32; 4] {
        self.element.color()
    }

    pub fn can_bond(&self) -> bool {
        self.bonds.len() < self.element.max_bonds()
    }
}

/// A bond between two atoms
#[derive(Debug, Clone, Copy)]
pub struct Bond {
    pub atom_a: usize,
    pub atom_b: usize,
    pub order: u8, // 1 = single, 2 = double, 3 = triple
}

impl Bond {
    pub fn new(atom_a: usize, atom_b: usize) -> Self {
        Self {
            atom_a,
            atom_b,
            order: 1,
        }
    }
}

/// The physics simulation
pub struct Simulation {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
    pub temperature: f32,
    pub damping: f32,
    next_id: usize,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
            temperature: 300.0, // Room temperature in Kelvin
            damping: 0.98,
            next_id: 0,
        }
    }

    pub fn add_atom(&mut self, element: Element, position: Vec2) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.atoms.push(Atom::new(element, position, id));
        id
    }

    /// Initialize with water molecules (H2O)
    pub fn init_water(&mut self, count: usize) {
        self.clear();
        let mut rng = rand::thread_rng();

        for i in 0..count {
            let angle = (i as f32 / count as f32) * 2.0 * PI;
            let radius = 3.0 + rng.gen::<f32>() * 5.0;
            let center = Vec2::new(angle.cos() * radius, angle.sin() * radius);

            // Create H2O molecule
            let o_pos = center;
            let h1_pos = center + Vec2::new(-0.6, 0.4);
            let h2_pos = center + Vec2::new(0.6, 0.4);

            let o_id = self.add_atom(Element::Oxygen, o_pos);
            let h1_id = self.add_atom(Element::Hydrogen, h1_pos);
            let h2_id = self.add_atom(Element::Hydrogen, h2_pos);

            self.create_bond(o_id, h1_id);
            self.create_bond(o_id, h2_id);
        }
    }

    /// Initialize with salt (NaCl) ions
    pub fn init_salt(&mut self, count: usize) {
        self.clear();
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let pos = Vec2::new(
                (rng.gen::<f32>() - 0.5) * 15.0,
                (rng.gen::<f32>() - 0.5) * 15.0,
            );

            self.add_atom(Element::Sodium, pos);

            let pos2 = Vec2::new(
                (rng.gen::<f32>() - 0.5) * 15.0,
                (rng.gen::<f32>() - 0.5) * 15.0,
            );

            self.add_atom(Element::Chlorine, pos2);
        }
    }

    /// Initialize with organic molecules (simple hydrocarbons)
    pub fn init_organic(&mut self, count: usize) {
        self.clear();
        let mut rng = rand::thread_rng();

        for i in 0..count {
            let angle = (i as f32 / count as f32) * 2.0 * PI;
            let radius = 4.0 + rng.gen::<f32>() * 4.0;
            let center = Vec2::new(angle.cos() * radius, angle.sin() * radius);

            // Create CH4 (methane) molecule
            let c_pos = center;
            let c_id = self.add_atom(Element::Carbon, c_pos);

            for j in 0..4 {
                let h_angle = (j as f32 / 4.0) * 2.0 * PI + rng.gen::<f32>() * 0.2;
                let h_pos = center + Vec2::new(h_angle.cos() * 0.8, h_angle.sin() * 0.8);
                let h_id = self.add_atom(Element::Hydrogen, h_pos);
                self.create_bond(c_id, h_id);
            }
        }
    }

    /// Initialize with random atoms
    pub fn init_random(&mut self, count: usize) {
        self.clear();
        let mut rng = rand::thread_rng();

        let elements = [
            Element::Hydrogen,
            Element::Carbon,
            Element::Nitrogen,
            Element::Oxygen,
        ];

        for _ in 0..count {
            let pos = Vec2::new(
                (rng.gen::<f32>() - 0.5) * 15.0,
                (rng.gen::<f32>() - 0.5) * 15.0,
            );

            let element = elements[rng.gen_range(0..elements.len())];
            self.add_atom(element, pos);
        }
    }

    pub fn clear(&mut self) {
        self.atoms.clear();
        self.bonds.clear();
        self.next_id = 0;
    }

    pub fn create_bond(&mut self, a: usize, b: usize) -> bool {
        // Check if bond already exists
        for bond in &self.bonds {
            if (bond.atom_a == a && bond.atom_b == b) || (bond.atom_a == b && bond.atom_b == a) {
                return false;
            }
        }

        // Check if atoms can form more bonds
        let atom_a = &self.atoms[a];
        let atom_b = &self.atoms[b];

        if !atom_a.can_bond() || !atom_b.can_bond() {
            return false;
        }

        self.bonds.push(Bond::new(a, b));
        self.atoms[a].bonds.push(b);
        self.atoms[b].bonds.push(a);

        true
    }

    /// Step the simulation forward
    pub fn step(&mut self, dt: f32) {
        let n = self.atoms.len();
        if n == 0 {
            return;
        }

        // Calculate forces
        let mut forces = vec![Vec2::ZERO; n];

        // Coulomb forces between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.atoms[j].position - self.atoms[i].position;
                let dist = r.length().max(0.5);
                let r_hat = r / dist;

                // Coulomb force
                let q1 = self.atoms[i].charge();
                let q2 = self.atoms[j].charge();
                if q1.abs() > 0.01 && q2.abs() > 0.01 {
                    let coulomb_force = K_COULOMB * q1 * q2 / (dist * dist);
                    forces[i] -= r_hat * coulomb_force;
                    forces[j] += r_hat * coulomb_force;
                }

                // Lennard-Jones potential (short-range repulsion/attraction)
                let sigma = (self.atoms[i].radius() + self.atoms[j].radius()) * LJ_SIGMA;
                let sr6 = (sigma / dist).powi(6);
                let sr12 = sr6 * sr6;
                let lj_force = 24.0 * LJ_EPSILON * (2.0 * sr12 - sr6) / dist;

                forces[i] -= r_hat * lj_force;
                forces[j] += r_hat * lj_force;
            }
        }

        // Bond forces (spring-like)
        for bond in &self.bonds {
            let a = bond.atom_a;
            let b = bond.atom_b;

            let r = self.atoms[b].position - self.atoms[a].position;
            let dist = r.length().max(0.01);
            let r_hat = r / dist;

            // Target bond length
            let eq_dist = self.atoms[a].radius() + self.atoms[b].radius();
            let displacement = dist - eq_dist;

            let spring_force = BOND_STRENGTH * displacement * r_hat;
            forces[a] += spring_force;
            forces[b] -= spring_force;
        }

        // Update velocities and positions
        for (i, atom) in self.atoms.iter_mut().enumerate() {
            let accel = forces[i] / atom.mass();
            atom.velocity += accel * dt;
            atom.velocity *= self.damping;
            atom.position += atom.velocity * dt;

            // Boundary conditions (soft walls)
            let bound = 12.0;
            if atom.position.x.abs() > bound {
                atom.position.x = atom.position.x.signum() * bound;
                atom.velocity.x *= -0.5;
            }
            if atom.position.y.abs() > bound {
                atom.position.y = atom.position.y.signum() * bound;
                atom.velocity.y *= -0.5;
            }
        }

        // Try to form new bonds (simple proximity-based bonding)
        self.try_form_bonds();
    }

    fn try_form_bonds(&mut self) {
        let n = self.atoms.len();

        for i in 0..n {
            if !self.atoms[i].can_bond() {
                continue;
            }

            for j in (i + 1)..n {
                if !self.atoms[j].can_bond() {
                    continue;
                }

                // Don't bond ions of the same charge
                let q1 = self.atoms[i].charge();
                let q2 = self.atoms[j].charge();
                if q1 * q2 > 0.0 {
                    continue;
                }

                let dist = (self.atoms[j].position - self.atoms[i].position).length();
                let bond_threshold = (self.atoms[i].radius() + self.atoms[j].radius()) * BOND_DISTANCE;

                if dist < bond_threshold {
                    // Check relative velocity (only bond if approaching slowly)
                    let rel_vel = (self.atoms[j].velocity - self.atoms[i].velocity).length();
                    if rel_vel < 2.0 {
                        self.create_bond(i, j);
                    }
                }
            }
        }
    }

    /// Calculate total kinetic energy
    pub fn kinetic_energy(&self) -> f32 {
        self.atoms
            .iter()
            .map(|a| 0.5 * a.mass() * a.velocity.length_squared())
            .sum()
    }

    /// Calculate average temperature from kinetic energy
    pub fn measured_temperature(&self) -> f32 {
        if self.atoms.is_empty() {
            return 0.0;
        }

        // T = 2 * KE / (k_B * N * dimensions)
        // Simplified with k_B = 1
        let ke = self.kinetic_energy();
        2.0 * ke / (self.atoms.len() as f32 * 2.0)
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
