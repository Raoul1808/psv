use std::time::Duration;

use eframe::egui::{self, Widget};
use egui_alignments::center_horizontal;

use crate::sim::Sim;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    PlayingForward,
    PlayingBackward,
}

pub struct Controls {
    need_redraw: bool,
    play_state: PlayState,
    instructions_per_second: usize,
}

impl Controls {
    pub fn new() -> Self {
        Controls {
            need_redraw: false,
            play_state: PlayState::Stopped,
            instructions_per_second: 60,
        }
    }

    pub fn clear(&mut self) {
        self.need_redraw = false;
        self.play_state = PlayState::Stopped;
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn play_state(&self) -> PlayState {
        self.play_state
    }

    pub fn playback_speed(&self) -> Duration {
        Duration::from_secs_f64(1. / self.instructions_per_second as f64)
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
                self.play_state = PlayState::Stopped;
                self.need_redraw |= sim.skip_to(counter);
            }
        });
        center_horizontal(ui, |ui| {
            let res = ui.scope(|ui| {
                let speed =
                    10_f64.powf(0_f64.max(self.instructions_per_second.ilog10() as f64 - 2.));
                egui::DragValue::new(&mut self.instructions_per_second)
                    .speed(speed)
                    .range(2..=(sim.instructions().len() / 4).max(60))
                    .ui(ui);
                ui.label("ips");
            });
            egui::Tooltip::for_enabled(&res.response)
                .at_pointer()
                .show(|ui| ui.label("playback speed in instructions per second"));
            ui.separator();
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
                let can_play = !sim.instructions().is_empty();
                let start_cond = program_counter > 0;
                let fwd_cond = self.play_state == PlayState::PlayingForward;
                let bwd_cond = self.play_state == PlayState::PlayingBackward;
                let end_cond = program_counter < instructions_len;
                if ui.add_enabled(start_cond, egui::Button::new("⏮")).clicked() {
                    self.need_redraw |= sim.skip_to(0);
                }
                if ui.add_enabled(start_cond, egui::Button::new("⏴")).clicked() {
                    self.need_redraw |= sim.undo();
                }
                if bwd_cond {
                    if ui.add_enabled(can_play, egui::Button::new("■")).clicked() {
                        self.play_state = PlayState::Stopped;
                    }
                } else {
                    if ui.add_enabled(can_play, egui::Button::new("◀")).clicked() {
                        self.play_state = PlayState::PlayingBackward;
                    }
                }
                if fwd_cond {
                    if ui.add_enabled(can_play, egui::Button::new("■")).clicked() {
                        self.play_state = PlayState::Stopped;
                    }
                } else {
                    if ui.add_enabled(can_play, egui::Button::new("▶")).clicked() {
                        self.play_state = PlayState::PlayingForward;
                    }
                }
                if ui.add_enabled(end_cond, egui::Button::new("⏵")).clicked() {
                    self.need_redraw |= sim.step();
                }
                if ui.add_enabled(end_cond, egui::Button::new("⏭")).clicked() {
                    self.need_redraw |= sim.skip_to(instructions_len);
                }
            });
            ui.separator();
            let mut program_counter = sim.program_counter();
            let instructions_len = sim.instructions().len();
            let res = egui::DragValue::new(&mut program_counter)
                .range(0..=instructions_len)
                .ui(ui);
            if res.changed() {
                self.play_state = PlayState::Stopped;
                self.need_redraw |= sim.skip_to(program_counter);
            }
            ui.label(format!("/ {instructions_len}"));
        });
    }
}
