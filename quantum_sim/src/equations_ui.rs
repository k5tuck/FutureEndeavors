//! Equations sidebar UI for quantum simulations
//!
//! Provides egui-based sidebars displaying relevant physics equations

use egui::{Context, RichText, Color32, FontId, FontFamily};

/// Equation entry with label and formula
pub struct Equation {
    pub name: &'static str,
    pub formula: &'static str,
    pub description: &'static str,
}

/// Draw a styled equation sidebar
pub fn draw_equations_sidebar(
    ctx: &Context,
    title: &str,
    equations: &[Equation],
    variables: &[(&str, &str)],
) {
    egui::SidePanel::right("equations_panel")
        .min_width(280.0)
        .max_width(350.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new(title).color(Color32::from_rgb(100, 200, 255)));
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Equations section
            ui.label(RichText::new("Equations").strong().color(Color32::from_rgb(255, 200, 100)));
            ui.add_space(5.0);

            for eq in equations {
                draw_equation(ui, eq);
                ui.add_space(8.0);
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Variables section
            ui.label(RichText::new("Variables").strong().color(Color32::from_rgb(255, 200, 100)));
            ui.add_space(5.0);

            for (symbol, meaning) in variables {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*symbol).color(Color32::from_rgb(150, 255, 150))
                        .font(FontId::new(14.0, FontFamily::Monospace)));
                    ui.label(RichText::new("=").color(Color32::GRAY));
                    ui.label(RichText::new(*meaning).color(Color32::LIGHT_GRAY));
                });
            }
        });
}

/// Draw a single equation with name, formula, and description
fn draw_equation(ui: &mut egui::Ui, eq: &Equation) {
    ui.group(|ui| {
        ui.label(RichText::new(eq.name).strong().color(Color32::WHITE));
        ui.label(
            RichText::new(eq.formula)
                .font(FontId::new(16.0, FontFamily::Monospace))
                .color(Color32::from_rgb(200, 220, 255))
        );
        ui.label(RichText::new(eq.description).small().color(Color32::GRAY));
    });
}

// ============================================
// Tunneling Equations
// ============================================

pub const TUNNELING_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Time-Dependent Schrödinger",
        formula: "iℏ ∂ψ/∂t = Ĥψ",
        description: "Governs wavefunction evolution",
    },
    Equation {
        name: "Hamiltonian",
        formula: "Ĥ = -ℏ²/2m ∇² + V(x)",
        description: "Kinetic + potential energy",
    },
    Equation {
        name: "Probability Density",
        formula: "ρ(x,t) = |ψ(x,t)|²",
        description: "Probability of finding particle",
    },
    Equation {
        name: "Transmission Coefficient",
        formula: "T ≈ e^(-2κa)",
        description: "Tunneling probability through barrier",
    },
    Equation {
        name: "Decay Constant",
        formula: "κ = √(2m(V₀-E))/ℏ",
        description: "Exponential decay in barrier",
    },
];

pub const TUNNELING_VARIABLES: &[(&str, &str)] = &[
    ("ψ", "Wavefunction"),
    ("ℏ", "Reduced Planck constant"),
    ("m", "Particle mass"),
    ("V(x)", "Potential energy"),
    ("E", "Particle energy"),
    ("V₀", "Barrier height"),
    ("a", "Barrier width"),
    ("T", "Transmission coefficient"),
];

// ============================================
// Orbital Equations
// ============================================

pub const ORBITAL_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Hydrogen Wavefunction",
        formula: "ψ_nlm = R_nl(r) Y_l^m(θ,φ)",
        description: "Radial × angular parts",
    },
    Equation {
        name: "Radial Equation",
        formula: "R_nl(r) = ρ^l e^(-ρ/2) L(ρ)",
        description: "Laguerre polynomials",
    },
    Equation {
        name: "Spherical Harmonics",
        formula: "Y_l^m = P_l^m(cosθ) e^(imφ)",
        description: "Angular momentum eigenfunctions",
    },
    Equation {
        name: "Energy Levels",
        formula: "E_n = -13.6 eV / n²",
        description: "Hydrogen energy spectrum",
    },
    Equation {
        name: "Bohr Radius",
        formula: "a₀ = ℏ²/(m_e e²) ≈ 0.529 Å",
        description: "Characteristic atomic length",
    },
];

pub const ORBITAL_VARIABLES: &[(&str, &str)] = &[
    ("n", "Principal quantum number"),
    ("l", "Angular momentum (0 to n-1)"),
    ("m", "Magnetic quantum number"),
    ("r,θ,φ", "Spherical coordinates"),
    ("R_nl", "Radial wavefunction"),
    ("Y_l^m", "Spherical harmonic"),
    ("a₀", "Bohr radius"),
];

// ============================================
// Teleportation Equations
// ============================================

pub const TELEPORTATION_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Qubit State",
        formula: "|ψ⟩ = α|0⟩ + β|1⟩",
        description: "General superposition state",
    },
    Equation {
        name: "Bell State (Φ⁺)",
        formula: "|Φ⁺⟩ = (|00⟩ + |11⟩)/√2",
        description: "Maximally entangled pair",
    },
    Equation {
        name: "CNOT Gate",
        formula: "CNOT|a,b⟩ = |a, a⊕b⟩",
        description: "Controlled-NOT operation",
    },
    Equation {
        name: "Hadamard Gate",
        formula: "H|0⟩ = (|0⟩+|1⟩)/√2",
        description: "Creates superposition",
    },
    Equation {
        name: "Teleportation Fidelity",
        formula: "F = |⟨ψ_in|ψ_out⟩|²",
        description: "State transfer quality",
    },
];

pub const TELEPORTATION_VARIABLES: &[(&str, &str)] = &[
    ("|0⟩,|1⟩", "Computational basis"),
    ("α, β", "Complex amplitudes"),
    ("|α|²+|β|²", "= 1 (normalization)"),
    ("⊕", "XOR operation"),
    ("H", "Hadamard gate"),
    ("CNOT", "Controlled-NOT gate"),
];

// ============================================
// Quark/QCD Equations
// ============================================

pub const QUARK_EQUATIONS: &[Equation] = &[
    Equation {
        name: "QCD Lagrangian",
        formula: "ℒ = ψ̄(iγᵘDᵤ - m)ψ - ¼GᵃᵤᵥGᵃᵘᵛ",
        description: "Strong force field theory",
    },
    Equation {
        name: "Color Confinement",
        formula: "V(r) = -4αₛ/3r + σr",
        description: "Coulomb + linear potential",
    },
    Equation {
        name: "Running Coupling",
        formula: "αₛ(Q²) = 12π/((33-2nf)ln(Q²/Λ²))",
        description: "Asymptotic freedom",
    },
    Equation {
        name: "Color Singlet",
        formula: "RGB → White (colorless)",
        description: "Baryons are color-neutral",
    },
    Equation {
        name: "Meson Structure",
        formula: "qq̄ (quark-antiquark)",
        description: "Color-anticolor pair",
    },
];

pub const QUARK_VARIABLES: &[(&str, &str)] = &[
    ("αₛ", "Strong coupling constant"),
    ("σ", "String tension (~1 GeV/fm)"),
    ("R,G,B", "Color charges"),
    ("q", "Quark field"),
    ("Gᵃᵤᵥ", "Gluon field tensor"),
    ("Λ_QCD", "QCD scale (~200 MeV)"),
];

// ============================================
// Quantum Hall Equations
// ============================================

pub const HALL_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Cyclotron Frequency",
        formula: "ωc = eB/m*",
        description: "Electron orbital frequency",
    },
    Equation {
        name: "Landau Levels",
        formula: "E_n = ℏωc(n + ½)",
        description: "Quantized energy levels",
    },
    Equation {
        name: "Magnetic Length",
        formula: "l_B = √(ℏ/eB)",
        description: "Characteristic length scale",
    },
    Equation {
        name: "Hall Conductance",
        formula: "σ_xy = ν × e²/h",
        description: "Quantized in units of e²/h",
    },
    Equation {
        name: "Filling Factor",
        formula: "ν = n_e × h/(eB)",
        description: "Electrons per Landau level",
    },
];

pub const HALL_VARIABLES: &[(&str, &str)] = &[
    ("B", "Magnetic field strength"),
    ("m*", "Effective electron mass"),
    ("n", "Landau level index"),
    ("ν", "Filling factor"),
    ("σ_xy", "Hall conductance"),
    ("e²/h", "Conductance quantum"),
    ("l_B", "Magnetic length"),
];

// ============================================
// 4D Hypercube Equations
// ============================================

pub const HYPERCUBE_EQUATIONS: &[Equation] = &[
    Equation {
        name: "4D Point",
        formula: "P = (x, y, z, w)",
        description: "Position in 4-space",
    },
    Equation {
        name: "4D Rotation (XW plane)",
        formula: "x' = x cos θ - w sin θ\nw' = x sin θ + w cos θ",
        description: "Rotation mixing x and w",
    },
    Equation {
        name: "Perspective Projection",
        formula: "P₃ = P₄ × d/(d - w)",
        description: "4D → 3D projection",
    },
    Equation {
        name: "Tesseract Vertices",
        formula: "2⁴ = 16 vertices",
        description: "All (±1,±1,±1,±1)",
    },
    Equation {
        name: "Tesseract Edges",
        formula: "32 edges, 24 faces, 8 cells",
        description: "Hypercube structure",
    },
];

pub const HYPERCUBE_VARIABLES: &[(&str, &str)] = &[
    ("x,y,z", "Spatial dimensions"),
    ("w", "Fourth dimension"),
    ("d", "Projection distance"),
    ("θ", "Rotation angle"),
    ("P₃", "Projected 3D point"),
    ("P₄", "Original 4D point"),
];
