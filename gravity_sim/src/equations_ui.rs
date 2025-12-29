//! Equations sidebar UI for gravity simulations
//!
//! Displays relevant physics equations using egui.

use egui::{Context, RichText, Color32};

/// An equation with its name and formula
pub struct Equation {
    pub name: &'static str,
    pub formula: &'static str,
    pub description: &'static str,
}

/// Draw the equations sidebar
pub fn draw_equations_sidebar(
    ctx: &Context,
    title: &str,
    equations: &[Equation],
    variables: &[(&str, &str)],
) {
    egui::SidePanel::right("equations_panel")
        .resizable(true)
        .default_width(280.0)
        .show(ctx, |ui| {
            ui.heading(RichText::new(title).color(Color32::LIGHT_BLUE));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing(RichText::new("📐 Equations").strong(), |ui| {
                    for eq in equations {
                        ui.group(|ui| {
                            ui.label(RichText::new(eq.name).strong().color(Color32::YELLOW));
                            ui.label(RichText::new(eq.formula).monospace().color(Color32::WHITE));
                            ui.label(RichText::new(eq.description).small().italics());
                        });
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(8.0);

                ui.collapsing(RichText::new("📖 Variables").strong(), |ui| {
                    egui::Grid::new("variables_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            for (symbol, meaning) in variables {
                                ui.label(RichText::new(*symbol).monospace().color(Color32::LIGHT_GREEN));
                                ui.label(*meaning);
                                ui.end_row();
                            }
                        });
                });
            });
        });
}

// ============================================================================
// Gravity Simulation Equations
// ============================================================================

pub const GRAVITY_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Newton's Law of Gravitation",
        formula: "F = G·m₁·m₂ / r²",
        description: "Force between two masses",
    },
    Equation {
        name: "Gravitational Acceleration",
        formula: "a = G·M / r²",
        description: "Acceleration due to gravity",
    },
    Equation {
        name: "Orbital Velocity",
        formula: "v = √(G·M / r)",
        description: "Circular orbit velocity",
    },
    Equation {
        name: "Escape Velocity",
        formula: "v_esc = √(2·G·M / r)",
        description: "Velocity to escape gravity",
    },
    Equation {
        name: "Gravitational Potential Energy",
        formula: "U = -G·m₁·m₂ / r",
        description: "Energy stored in gravitational field",
    },
    Equation {
        name: "Kepler's Third Law",
        formula: "T² = (4π²/GM)·a³",
        description: "Orbital period relation",
    },
];

pub const GRAVITY_VARIABLES: &[(&str, &str)] = &[
    ("G", "Gravitational constant"),
    ("M, m", "Mass of bodies"),
    ("r", "Distance between centers"),
    ("F", "Gravitational force"),
    ("a", "Acceleration / Semi-major axis"),
    ("v", "Velocity"),
    ("T", "Orbital period"),
    ("U", "Potential energy"),
];

// ============================================================================
// 3D Gravity Simulation Equations (includes additional orbital mechanics)
// ============================================================================

pub const GRAVITY_3D_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Newton's Law of Gravitation",
        formula: "F⃗ = -G·m₁·m₂ / |r⃗|² · r̂",
        description: "Vector force between masses",
    },
    Equation {
        name: "N-Body Acceleration",
        formula: "a⃗ᵢ = Σⱼ G·mⱼ·(r⃗ⱼ-r⃗ᵢ) / |r⃗ⱼ-r⃗ᵢ|³",
        description: "Sum of all gravitational influences",
    },
    Equation {
        name: "Orbital Angular Momentum",
        formula: "L⃗ = r⃗ × p⃗ = m·r⃗ × v⃗",
        description: "Conserved in central force",
    },
    Equation {
        name: "Vis-viva Equation",
        formula: "v² = GM(2/r - 1/a)",
        description: "Orbital speed at any point",
    },
    Equation {
        name: "Specific Orbital Energy",
        formula: "ε = v²/2 - GM/r = -GM/2a",
        description: "Total energy per unit mass",
    },
    Equation {
        name: "Hill Sphere Radius",
        formula: "r_H ≈ a·(m/3M)^(1/3)",
        description: "Gravitational sphere of influence",
    },
];

pub const GRAVITY_3D_VARIABLES: &[(&str, &str)] = &[
    ("G", "Gravitational constant (6.674×10⁻¹¹)"),
    ("M, m", "Mass of bodies"),
    ("r⃗", "Position vector"),
    ("r̂", "Unit vector toward mass"),
    ("v⃗", "Velocity vector"),
    ("a", "Semi-major axis"),
    ("L⃗", "Angular momentum"),
    ("ε", "Specific orbital energy"),
    ("r_H", "Hill sphere radius"),
];
