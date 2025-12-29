//! Equations sidebar UI for atomic simulations
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
// Atomic / Molecular Dynamics Equations
// ============================================================================

pub const ATOMS_EQUATIONS: &[Equation] = &[
    Equation {
        name: "Coulomb's Law",
        formula: "F = k·q₁·q₂ / r²",
        description: "Electrostatic force between charges",
    },
    Equation {
        name: "Lennard-Jones Potential",
        formula: "V(r) = 4ε[(σ/r)¹² - (σ/r)⁶]",
        description: "Van der Waals interaction",
    },
    Equation {
        name: "Lennard-Jones Force",
        formula: "F(r) = 24ε/r[2(σ/r)¹² - (σ/r)⁶]",
        description: "Derivative of LJ potential",
    },
    Equation {
        name: "Harmonic Bond",
        formula: "V(r) = ½k(r - r₀)²",
        description: "Covalent bond spring potential",
    },
    Equation {
        name: "Kinetic Energy",
        formula: "KE = ½mv²",
        description: "Energy from atomic motion",
    },
    Equation {
        name: "Temperature Relation",
        formula: "⟨KE⟩ = (3/2)k_B·T",
        description: "Average kinetic energy per atom",
    },
];

pub const ATOMS_VARIABLES: &[(&str, &str)] = &[
    ("k", "Coulomb constant (8.99×10⁹ N·m²/C²)"),
    ("q", "Electric charge"),
    ("r", "Distance between atoms"),
    ("ε", "LJ well depth (binding energy)"),
    ("σ", "LJ collision diameter"),
    ("r₀", "Equilibrium bond length"),
    ("k_spring", "Bond force constant"),
    ("k_B", "Boltzmann constant"),
    ("T", "Temperature"),
    ("m", "Atomic mass"),
];
