use std::time::{Duration, Instant};

use eframe::egui::{self, Widget};

use crate::{help::HelpSection, visualizer::PlayState};

pub struct PlaybackControlsSection {
    counter: u16,
    speed: u16,
    play_state: PlayState,
    last_instant: Instant,
    duration_accumulated: Duration,
}

const SPEED_MAX: u16 = u16::MAX / 4;

impl PlaybackControlsSection {
    pub fn new() -> Self {
        Self {
            counter: rand::random(),
            speed: 60,
            play_state: PlayState::Stopped,
            last_instant: Instant::now(),
            duration_accumulated: Duration::ZERO,
        }
    }

    fn update_playback(&mut self, ui: &mut egui::Ui) {
        match self.play_state {
            PlayState::PlayingForward => {
                let exec_interval = Duration::from_secs_f64(1. / self.speed as f64);
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration && self.counter < u16::MAX {
                    self.counter += 1;
                    self.duration_accumulated += exec_interval;
                }
                if self.counter == u16::MAX {
                    self.play_state = PlayState::Stopped;
                    self.duration_accumulated = Duration::ZERO;
                } else {
                    self.duration_accumulated -= catching_duration;
                }
                ui.request_repaint();
            }
            PlayState::PlayingBackward => {
                let exec_interval = Duration::from_secs_f64(1. / self.speed as f64);
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration && self.counter > 0 {
                    self.counter -= 1;
                    self.duration_accumulated += exec_interval;
                }
                if self.counter == 0 {
                    self.play_state = PlayState::Stopped;
                    self.duration_accumulated = Duration::ZERO;
                } else {
                    self.duration_accumulated -= catching_duration;
                }
                ui.request_repaint();
            }
            _ => {
                self.duration_accumulated = Duration::ZERO;
            }
        }
        self.last_instant = Instant::now();
    }
}

impl HelpSection for PlaybackControlsSection {
    fn section_name(&self) -> &str {
        "Playback controls"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        self.update_playback(ui);
        ui.label("Once instructions are loaded, you can use the playback controls at the bottom of the screen.");
        ui.add_space(10.);
        ui.label("First is a slider. It will show the playback progress of all instructions. You can click and drag the slider head to move through instructions. It's like watching a video!");
        ui.add_space(5.);
        ui.scope(|ui| {
            let available_width = ui.available_width();
            ui.spacing_mut().slider_width = available_width;
            egui::Slider::new(&mut self.counter, 0..=u16::MAX)
                .show_value(false)
                .ui(ui);
        });
        ui.add_space(5.);
        ui.label("Playback buttons allow you to play, pause, skip or step through instructions. Each button does, in order:");
        ui.label("- Skip to the beginning");
        ui.label("- Step one frame backward");
        ui.label("- Play in reverse");
        ui.label("- Play normally");
        ui.label("- Step one frame forward");
        ui.label("- Skip to the end");
        ui.add_space(5.);
        ui.scope(|ui| {
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(24., egui::epaint::FontFamily::Proportional),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(24., egui::epaint::FontFamily::Proportional),
            );
            ui.horizontal(|ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let start_cond = self.counter > 0;
                    let fwd_cond = self.play_state == PlayState::PlayingForward;
                    let bwd_cond = self.play_state == PlayState::PlayingBackward;
                    let end_cond = self.counter < u16::MAX;
                    if ui.add_enabled(start_cond, egui::Button::new("⏮")).clicked() {
                        self.counter = 0;
                    }
                    if ui.add_enabled(start_cond, egui::Button::new("⏴")).clicked() {
                        self.counter -= 1;
                    }
                    if bwd_cond {
                        if ui.button("■").clicked() {
                            self.play_state = PlayState::Stopped;
                        }
                    } else {
                        if ui.button("◀").clicked() {
                            self.play_state = PlayState::PlayingBackward;
                        }
                    }
                    if fwd_cond {
                        if ui.button("■").clicked() {
                            self.play_state = PlayState::Stopped;
                        }
                    } else {
                        if ui.button("▶").clicked() {
                            self.play_state = PlayState::PlayingForward;
                        }
                    }
                    if ui.add_enabled(end_cond, egui::Button::new("⏵")).clicked() {
                        self.counter += 1;
                    }
                    if ui.add_enabled(end_cond, egui::Button::new("⏭")).clicked() {
                        self.counter = u16::MAX;
                    }
                });
            });
        });
        ui.add_space(5.);
        ui.label("On the left of the controls, you can adjust the playback speed, from 2 instructions per second to either 60 or a quarter of the amount of instructions, whichever is higher.");
        ui.label("You can click and drag, or double click to edit the value.");

        ui.add_space(5.);
        ui.horizontal(|ui| {
            let speed = 10_f64.powf(0_f64.max(self.speed.ilog10() as f64 - 2.));
            egui::DragValue::new(&mut self.speed)
                .speed(speed)
                .range(2..=SPEED_MAX.max(60))
                .ui(ui);
            ui.label("ips");
        });
        ui.add_space(5.);

        ui.label("Finally, you can see how many instructions were loaded, and which instruction you're currently viewing with the last label.");
        ui.label("Just like the speed control, you can click and drag, or double click the instruction number to jump around the program.");

        ui.add_space(5.);
        ui.horizontal(|ui| {
            egui::DragValue::new(&mut self.counter)
                .range(0..=u16::MAX)
                .ui(ui);
            ui.label(format!("/ {}", u16::MAX));
        });
    }
}
