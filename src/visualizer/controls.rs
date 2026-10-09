use eframe::egui::{self, Widget};
use egui_alignments::center_horizontal;

use crate::sim::Sim;

pub struct Controls {
    need_redraw: bool,
}

impl Controls {
    pub fn new() -> Self {
        Controls { need_redraw: false }
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, sim: &mut Sim) {
        self.need_redraw = false;
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = ui.available_width();

            let mut counter = sim.program_counter();

            let instructions = sim.instructions();
            let max = if instructions.is_empty() {
                0
            } else {
                instructions.len()
            };

            let slider = egui::Slider::new(&mut counter, 0..=max)
                .show_value(false)
                .ui(ui);
            if slider.changed() {
                sim.skip_to(counter);
                self.need_redraw = true;
            }
        });
        center_horizontal(ui, |ui| {
            ui.scope(|ui| {
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(24., egui::epaint::FontFamily::Proportional),
                );
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(24., egui::epaint::FontFamily::Proportional),
                );

                let program_counter = sim.program_counter();
                let instructions_len = sim.instructions().len();
                let start_cond = program_counter > 0;
                let end_cond = program_counter < instructions_len;
                if ui.add_enabled(start_cond, egui::Button::new("⏮")).clicked() {
                    self.need_redraw |= sim.skip_to(0);
                }
                if ui.add_enabled(start_cond, egui::Button::new("⏴")).clicked() {
                    self.need_redraw |= sim.undo();
                }
                if ui.add_enabled(end_cond, egui::Button::new("⏵")).clicked() {
                    self.need_redraw |= sim.step();
                }
                if ui.add_enabled(end_cond, egui::Button::new("⏭")).clicked() {
                    self.need_redraw |= sim.skip_to(instructions_len);
                }
            });
        });
    }
}
