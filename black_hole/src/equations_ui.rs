//! Equations sidebar UI for black hole simulations
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
// Black Hole (3D) Equations - Schwarzschild Geometry
// ============================================================================

pub const BLACK_HOLE_3D_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Schwarzschild Radius",
        formula: "rₛ = 2GM/c²",
        description: "Event horizon radius",
    },
    Equation {
        name: "Schwarzschild Metric",
        formula: "ds² = -(1-rₛ/r)c²dt² + (1-rₛ/r)⁻¹dr² + r²dΩ²",
        description: "Spacetime interval in Schwarzschild coords",
    },
    Equation {
        name: "Photon Sphere",
        formula: "r_ph = 3GM/c² = 1.5rₛ",
        description: "Radius where light can orbit",
    },
    Equation {
        name: "Gravitational Redshift",
        formula: "z = (1 - rₛ/r)^(-1/2) - 1",
        description: "Frequency shift near black hole",
    },
    Equation {
        name: "Innermost Stable Orbit",
        formula: "r_ISCO = 6GM/c² = 3rₛ",
        description: "Closest stable circular orbit",
    },
    Equation {
        name: "Hawking Temperature",
        formula: "T = ℏc³/(8πGMk_B)",
        description: "Black hole thermal radiation",
    },
];

pub const BLACK_HOLE_3D_VARIABLES: &[(&str, &str)] = &[
    ("G", "Gravitational constant"),
    ("M", "Black hole mass"),
    ("c", "Speed of light"),
    ("rₛ", "Schwarzschild radius"),
    ("r", "Radial coordinate"),
    ("dΩ²", "Angular element (dθ² + sin²θ dφ²)"),
    ("z", "Redshift factor"),
    ("ℏ", "Reduced Planck constant"),
    ("k_B", "Boltzmann constant"),
];

// ============================================================================
// Black Hole (2D) Equations - Light Deflection
// ============================================================================

pub const BLACK_HOLE_2D_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Light Deflection Angle",
        formula: "Δφ = 4GM/(c²b)",
        description: "Bending angle for distant light",
    },
    Equation {
        name: "Impact Parameter",
        formula: "b = r·sin(ψ)",
        description: "Perpendicular distance to trajectory",
    },
    Equation {
        name: "Effective Potential",
        formula: "V_eff = (1 - rₛ/r)(L²/r² + κc²)",
        description: "Governs photon trajectories",
    },
    Equation {
        name: "Critical Impact Parameter",
        formula: "b_crit = 3√3 GM/c²",
        description: "Photons with b < b_crit fall in",
    },
    Equation {
        name: "Einstein Ring Radius",
        formula: "θ_E = √(4GM·D_LS/(c²D_L·D_S))",
        description: "Apparent ring from perfect alignment",
    },
    Equation {
        name: "Geodesic Equation",
        formula: "d²xᵘ/dλ² + Γᵘ_αβ (dxᵅ/dλ)(dxᵝ/dλ) = 0",
        description: "Path of light in curved spacetime",
    },
];

pub const BLACK_HOLE_2D_VARIABLES: &[(&str, &str)] = &[
    ("Δφ", "Deflection angle"),
    ("b", "Impact parameter"),
    ("ψ", "Angle at closest approach"),
    ("L", "Angular momentum"),
    ("κ", "0 for photons, 1 for massive"),
    ("D_L", "Distance to lens"),
    ("D_S", "Distance to source"),
    ("D_LS", "Lens-source distance"),
    ("Γᵘ_αβ", "Christoffel symbols"),
    ("λ", "Affine parameter"),
];
