//! Equations sidebar UI for solar voyage simulation
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
// Solar Voyage Equations - Orbital Mechanics & Relativity
// ============================================================================

pub const SOLAR_VOYAGE_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Kepler's Third Law",
        formula: "T² = (4π²/GM)·a³",
        description: "Orbital period from semi-major axis",
    },
    Equation {
        name: "Vis-viva Equation",
        formula: "v² = GM(2/r - 1/a)",
        description: "Orbital velocity at any point",
    },
    Equation {
        name: "Escape Velocity",
        formula: "v_esc = √(2GM/r)",
        description: "Minimum speed to escape gravity",
    },
    Equation {
        name: "Lorentz Factor",
        formula: "γ = 1/√(1 - v²/c²)",
        description: "Relativistic time dilation factor",
    },
    Equation {
        name: "Time Dilation",
        formula: "Δt' = γ·Δt",
        description: "Moving clocks run slow",
    },
    Equation {
        name: "Length Contraction",
        formula: "L = L₀/γ",
        description: "Moving objects contract",
    },
    Equation {
        name: "Relativistic Momentum",
        formula: "p = γmv",
        description: "Momentum at high speeds",
    },
    Equation {
        name: "Schwarzschild Radius",
        formula: "rₛ = 2GM/c²",
        description: "Black hole event horizon",
    },
    Equation {
        name: "Gravitational Time Dilation",
        formula: "τ = t√(1 - rₛ/r)",
        description: "Clocks slow near massive objects",
    },
];

pub const SOLAR_VOYAGE_VARIABLES: &[(&str, &str)] = &[
    ("G", "Gravitational constant"),
    ("M", "Central mass (Sun, etc.)"),
    ("r", "Radial distance"),
    ("a", "Semi-major axis"),
    ("T", "Orbital period"),
    ("v", "Velocity"),
    ("c", "Speed of light"),
    ("γ", "Lorentz factor"),
    ("τ", "Proper time"),
    ("rₛ", "Schwarzschild radius"),
    ("L₀", "Rest length"),
    ("m", "Rest mass"),
];
