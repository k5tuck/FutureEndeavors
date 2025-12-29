//! Quark and Hadron Simulation
//!
//! Visualizes the strong force, quark confinement, and hadron structure.
//! Demonstrates color charge, gluon exchange, and asymptotic freedom.

use glam::Vec3;
use rand::Rng;
use std::f32::consts::PI;

/// Color charge (quantum chromodynamics)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCharge {
    Red,
    Green,
    Blue,
    AntiRed,    // Cyan
    AntiGreen,  // Magenta
    AntiBlue,   // Yellow
}

impl ColorCharge {
    /// Get the anticolor
    pub fn anti(&self) -> Self {
        match self {
            ColorCharge::Red => ColorCharge::AntiRed,
            ColorCharge::Green => ColorCharge::AntiGreen,
            ColorCharge::Blue => ColorCharge::AntiBlue,
            ColorCharge::AntiRed => ColorCharge::Red,
            ColorCharge::AntiGreen => ColorCharge::Green,
            ColorCharge::AntiBlue => ColorCharge::Blue,
        }
    }

    /// Display color for rendering
    pub fn render_color(&self) -> [f32; 4] {
        match self {
            ColorCharge::Red => [1.0, 0.2, 0.2, 1.0],
            ColorCharge::Green => [0.2, 1.0, 0.2, 1.0],
            ColorCharge::Blue => [0.2, 0.2, 1.0, 1.0],
            ColorCharge::AntiRed => [0.0, 1.0, 1.0, 1.0],
            ColorCharge::AntiGreen => [1.0, 0.0, 1.0, 1.0],
            ColorCharge::AntiBlue => [1.0, 1.0, 0.0, 1.0],
        }
    }

    /// Check if this color plus another makes a color-neutral combination
    pub fn neutralizes(&self, other: &ColorCharge) -> bool {
        matches!(
            (self, other),
            (ColorCharge::Red, ColorCharge::AntiRed)
                | (ColorCharge::AntiRed, ColorCharge::Red)
                | (ColorCharge::Green, ColorCharge::AntiGreen)
                | (ColorCharge::AntiGreen, ColorCharge::Green)
                | (ColorCharge::Blue, ColorCharge::AntiBlue)
                | (ColorCharge::AntiBlue, ColorCharge::Blue)
        )
    }
}

/// Quark flavor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarkFlavor {
    Up,      // +2/3 charge
    Down,    // -1/3 charge
    Charm,   // +2/3 charge
    Strange, // -1/3 charge
    Top,     // +2/3 charge
    Bottom,  // -1/3 charge
}

impl QuarkFlavor {
    /// Electric charge in units of e/3
    pub fn charge_thirds(&self) -> i32 {
        match self {
            QuarkFlavor::Up | QuarkFlavor::Charm | QuarkFlavor::Top => 2,
            QuarkFlavor::Down | QuarkFlavor::Strange | QuarkFlavor::Bottom => -1,
        }
    }

    /// Approximate mass in MeV/c² (scaled for visualization)
    pub fn mass(&self) -> f32 {
        match self {
            QuarkFlavor::Up => 2.2,
            QuarkFlavor::Down => 4.7,
            QuarkFlavor::Charm => 1275.0,
            QuarkFlavor::Strange => 95.0,
            QuarkFlavor::Top => 173000.0,
            QuarkFlavor::Bottom => 4180.0,
        }
    }

    /// Symbol for display
    pub fn symbol(&self) -> &'static str {
        match self {
            QuarkFlavor::Up => "u",
            QuarkFlavor::Down => "d",
            QuarkFlavor::Charm => "c",
            QuarkFlavor::Strange => "s",
            QuarkFlavor::Top => "t",
            QuarkFlavor::Bottom => "b",
        }
    }
}

/// A quark particle
#[derive(Debug, Clone)]
pub struct Quark {
    pub flavor: QuarkFlavor,
    pub color: ColorCharge,
    pub is_antiquark: bool,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Quark {
    pub fn new(flavor: QuarkFlavor, color: ColorCharge, is_antiquark: bool, position: Vec3) -> Self {
        let color = if is_antiquark { color.anti() } else { color };
        Self {
            flavor,
            color,
            is_antiquark,
            position,
            velocity: Vec3::ZERO,
        }
    }

    /// Effective radius for visualization (scaled)
    pub fn radius(&self) -> f32 {
        0.2 + (self.flavor.mass().ln() / 12.0).clamp(0.0, 0.5)
    }

    /// Display symbol
    pub fn symbol(&self) -> String {
        if self.is_antiquark {
            format!("{}̄", self.flavor.symbol())
        } else {
            self.flavor.symbol().to_string()
        }
    }
}

/// Gluon (force carrier)
#[derive(Debug, Clone)]
pub struct Gluon {
    pub color: ColorCharge,
    pub anticolor: ColorCharge,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
}

impl Gluon {
    pub fn new(color: ColorCharge, anticolor: ColorCharge, position: Vec3) -> Self {
        Self {
            color,
            anticolor,
            position,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
        }
    }
}

/// Flux tube connecting quarks (string-like confinement)
#[derive(Debug, Clone)]
pub struct FluxTube {
    pub quark_a: usize,
    pub quark_b: usize,
    pub tension: f32,        // String tension
    pub width: f32,          // Visual width
    pub color_flow: [f32; 4], // Visual color
}

/// Hadron types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HadronType {
    Proton,     // uud
    Neutron,    // udd
    PionPlus,   // ud̄
    PionMinus,  // ūd
    PionZero,   // (uū - dd̄)/√2
    Kaon,       // us̄ or ds̄
    Jpsi,       // cc̄
}

impl HadronType {
    pub fn name(&self) -> &'static str {
        match self {
            HadronType::Proton => "Proton",
            HadronType::Neutron => "Neutron",
            HadronType::PionPlus => "π+",
            HadronType::PionMinus => "π-",
            HadronType::PionZero => "π0",
            HadronType::Kaon => "K",
            HadronType::Jpsi => "J/ψ",
        }
    }
}

/// QCD simulation
pub struct QuarkSimulation {
    /// All quarks in the simulation
    pub quarks: Vec<Quark>,
    /// Gluons being exchanged
    pub gluons: Vec<Gluon>,
    /// Flux tubes connecting quarks
    pub flux_tubes: Vec<FluxTube>,
    /// Current hadron configuration
    pub hadron_type: Option<HadronType>,
    /// String tension (confinement strength)
    pub string_tension: f32,
    /// Coupling constant
    pub alpha_s: f32,
    /// Simulation time
    pub time: f32,
    /// Confinement radius
    pub confinement_radius: f32,
}

impl QuarkSimulation {
    pub fn new() -> Self {
        Self {
            quarks: Vec::new(),
            gluons: Vec::new(),
            flux_tubes: Vec::new(),
            hadron_type: None,
            string_tension: 1.0,
            alpha_s: 0.5, // Strong coupling
            time: 0.0,
            confinement_radius: 1.0,
        }
    }

    /// Create a proton (uud)
    pub fn init_proton(&mut self) {
        self.quarks.clear();
        self.gluons.clear();

        let r = 0.5;
        let angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];

        self.quarks.push(Quark::new(
            QuarkFlavor::Up,
            ColorCharge::Red,
            false,
            Vec3::new(r * angles[0].cos(), r * angles[0].sin(), 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Up,
            ColorCharge::Green,
            false,
            Vec3::new(r * angles[1].cos(), r * angles[1].sin(), 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Down,
            ColorCharge::Blue,
            false,
            Vec3::new(r * angles[2].cos(), r * angles[2].sin(), 0.0),
        ));

        self.hadron_type = Some(HadronType::Proton);
        self.update_flux_tubes();
    }

    /// Create a neutron (udd)
    pub fn init_neutron(&mut self) {
        self.quarks.clear();
        self.gluons.clear();

        let r = 0.5;
        let angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];

        self.quarks.push(Quark::new(
            QuarkFlavor::Up,
            ColorCharge::Red,
            false,
            Vec3::new(r * angles[0].cos(), r * angles[0].sin(), 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Down,
            ColorCharge::Green,
            false,
            Vec3::new(r * angles[1].cos(), r * angles[1].sin(), 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Down,
            ColorCharge::Blue,
            false,
            Vec3::new(r * angles[2].cos(), r * angles[2].sin(), 0.0),
        ));

        self.hadron_type = Some(HadronType::Neutron);
        self.update_flux_tubes();
    }

    /// Create a pion (quark-antiquark meson)
    pub fn init_pion_plus(&mut self) {
        self.quarks.clear();
        self.gluons.clear();

        self.quarks.push(Quark::new(
            QuarkFlavor::Up,
            ColorCharge::Red,
            false,
            Vec3::new(-0.4, 0.0, 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Down,
            ColorCharge::Red, // Will become AntiRed
            true,
            Vec3::new(0.4, 0.0, 0.0),
        ));

        self.hadron_type = Some(HadronType::PionPlus);
        self.update_flux_tubes();
    }

    /// Create J/ψ (charmonium)
    pub fn init_jpsi(&mut self) {
        self.quarks.clear();
        self.gluons.clear();

        self.quarks.push(Quark::new(
            QuarkFlavor::Charm,
            ColorCharge::Blue,
            false,
            Vec3::new(-0.3, 0.0, 0.0),
        ));
        self.quarks.push(Quark::new(
            QuarkFlavor::Charm,
            ColorCharge::Blue,
            true,
            Vec3::new(0.3, 0.0, 0.0),
        ));

        self.hadron_type = Some(HadronType::Jpsi);
        self.update_flux_tubes();
    }

    /// Update flux tubes based on current quark positions
    fn update_flux_tubes(&mut self) {
        self.flux_tubes.clear();

        let n = self.quarks.len();
        if n < 2 {
            return;
        }

        // For baryons (3 quarks): Y-shaped flux tube
        // For mesons (2 quarks): single tube
        if n == 3 {
            // Connect to center (simplified Y topology)
            let center: Vec3 = self.quarks.iter().map(|q| q.position).sum::<Vec3>() / 3.0;
            for i in 0..3 {
                let j = (i + 1) % 3;
                let dist = (self.quarks[i].position - self.quarks[j].position).length();
                self.flux_tubes.push(FluxTube {
                    quark_a: i,
                    quark_b: j,
                    tension: self.string_tension * dist,
                    width: 0.1,
                    color_flow: blend_colors(
                        self.quarks[i].color.render_color(),
                        self.quarks[j].color.render_color(),
                    ),
                });
            }
        } else if n == 2 {
            let dist = (self.quarks[0].position - self.quarks[1].position).length();
            self.flux_tubes.push(FluxTube {
                quark_a: 0,
                quark_b: 1,
                tension: self.string_tension * dist,
                width: 0.15,
                color_flow: blend_colors(
                    self.quarks[0].color.render_color(),
                    self.quarks[1].color.render_color(),
                ),
            });
        }
    }

    /// Simulate one timestep
    pub fn step(&mut self, dt: f32) {
        self.time += dt;

        let n = self.quarks.len();
        if n == 0 {
            return;
        }

        // Compute forces from confinement (linear potential)
        let mut forces = vec![Vec3::ZERO; n];

        // String force: F = -σ * r̂ (linear confinement)
        let center: Vec3 = self.quarks.iter().map(|q| q.position).sum::<Vec3>() / n as f32;

        for i in 0..n {
            let to_center = center - self.quarks[i].position;
            let dist = to_center.length();

            if dist > 0.01 {
                // Linear confining force
                let confine_force = self.string_tension * to_center.normalize();

                // Short-range repulsion (Coulomb-like at small distances)
                let repel = if dist < 0.3 {
                    -self.alpha_s * to_center.normalize() / (dist * dist + 0.01)
                } else {
                    Vec3::ZERO
                };

                forces[i] += confine_force + repel;
            }

            // Pairwise interactions
            for j in (i + 1)..n {
                let r = self.quarks[j].position - self.quarks[i].position;
                let dist = r.length();

                if dist > 0.01 {
                    let dir = r / dist;

                    // One-gluon exchange (Coulomb-like, but with running coupling)
                    let coulomb = -self.alpha_s / (dist * dist + 0.1);

                    // Confining linear term
                    let confine = self.string_tension * dist;

                    let force_mag = coulomb + confine * 0.1;
                    let force = dir * force_mag;

                    forces[i] += force;
                    forces[j] -= force;
                }
            }
        }

        // Update velocities and positions
        for (i, quark) in self.quarks.iter_mut().enumerate() {
            let mass = quark.flavor.mass().max(1.0);
            quark.velocity += forces[i] * dt / mass;
            quark.velocity *= 0.98; // Damping
            quark.position += quark.velocity * dt;
        }

        // Spawn virtual gluons for visualization
        self.update_gluons(dt);

        // Update flux tube visualization
        self.update_flux_tubes();

        // Color exchange (quantum fluctuation visualization)
        if rand::thread_rng().gen::<f32>() < 0.02 && n >= 2 {
            self.color_exchange();
        }
    }

    /// Update virtual gluon visualization
    fn update_gluons(&mut self, dt: f32) {
        // Update existing gluons
        self.gluons.retain_mut(|g| {
            g.lifetime += dt;
            g.position += g.velocity * dt;
            g.lifetime < 0.5 // Short lifetime
        });

        // Spawn new gluons between quarks
        let mut rng = rand::thread_rng();
        if self.quarks.len() >= 2 && rng.gen::<f32>() < 0.1 {
            let i = rng.gen_range(0..self.quarks.len());
            let j = (i + 1) % self.quarks.len();

            let pos = (self.quarks[i].position + self.quarks[j].position) / 2.0;
            let vel = (self.quarks[j].position - self.quarks[i].position).normalize() * 2.0;

            let gluon = Gluon::new(self.quarks[i].color, self.quarks[j].color.anti(), pos);
            self.gluons.push(Gluon {
                velocity: vel,
                ..gluon
            });
        }
    }

    /// Simulate color exchange between quarks
    fn color_exchange(&mut self) {
        if self.quarks.len() < 2 {
            return;
        }

        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.quarks.len());
        let j = rng.gen_range(0..self.quarks.len());

        if i != j {
            // Swap colors (simplified - real QCD is more complex)
            let temp = self.quarks[i].color;
            self.quarks[i].color = self.quarks[j].color;
            self.quarks[j].color = temp;
        }
    }

    /// Attempt to separate quarks (demonstrates confinement)
    pub fn apply_separation_force(&mut self, quark_index: usize, force: Vec3) {
        if quark_index < self.quarks.len() {
            self.quarks[quark_index].velocity += force * 0.1;
        }
    }

    /// Get render data
    pub fn get_quark_data(&self) -> Vec<(Vec3, f32, [f32; 4], String)> {
        self.quarks
            .iter()
            .map(|q| {
                (q.position, q.radius(), q.color.render_color(), q.symbol())
            })
            .collect()
    }

    /// Check if hadron is color-neutral
    pub fn is_color_neutral(&self) -> bool {
        if self.quarks.len() == 3 {
            // Baryon: need RGB or anti-RGB
            let colors: Vec<_> = self.quarks.iter().map(|q| q.color).collect();
            let has_r = colors.contains(&ColorCharge::Red) || colors.contains(&ColorCharge::AntiRed);
            let has_g = colors.contains(&ColorCharge::Green) || colors.contains(&ColorCharge::AntiGreen);
            let has_b = colors.contains(&ColorCharge::Blue) || colors.contains(&ColorCharge::AntiBlue);
            has_r && has_g && has_b
        } else if self.quarks.len() == 2 {
            // Meson: need color-anticolor pair
            self.quarks[0].color.neutralizes(&self.quarks[1].color)
        } else {
            false
        }
    }
}

impl Default for QuarkSimulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend two colors for flux tube visualization
fn blend_colors(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
        (a[3] + b[3]) / 2.0,
    ]
}
