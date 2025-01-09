use std::time::Duration;

use egui::{Button, Slider, Widget, Window};

use crate::sim::PushSwapSim;

pub struct PlaybackControls;

impl PlaybackControls {
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        sim: &mut PushSwapSim,
        play_sim: &mut bool,
        exec_duration: &mut Duration,
        regenerate_render_data: &mut bool,
    ) {
        let program_counter = sim.program_counter();
        Window::new("Playback Controls")
            .open(open)
            .show(ctx, move |ui| {
                ui.label(format!("Instructions loaded: {}", sim.instructions().len()));
                ui.label(format!("Program Counter: {}", sim.program_counter()));
                ui.scope(|ui| {
                    let instructions = sim.instructions();
                    ui.style_mut().spacing.slider_width = ui.available_width();
                    let mut counter = sim.program_counter();
                    let max = if instructions.is_empty() {
                        0
                    } else {
                        instructions.len()
                    };
                    let str: String = if *play_sim && program_counter < instructions.len() {
                        let total_duration = *exec_duration * instructions.len() as u32;
                        let t = program_counter as f32 / instructions.len() as f32;
                        let time_elapsed = total_duration.mul_f32(t);
                        let time_left = total_duration - time_elapsed;
                        let secs = time_left.as_secs();
                        if secs >= 60 {
                            format!("Estimated Time Remaining: {}m{}s", secs / 60, secs % 60)
                        } else {
                            format!("Estimated Time Remaining: {}s", secs)
                        }
                    } else {
                        "Estimated Time Remaining: N/A".into()
                    };
                    ui.label(str);
                    let slider = ui.add_enabled(
                        !instructions.is_empty(),
                        egui::Slider::new(&mut counter, 0..=max).show_value(false),
                    );
                    if slider.changed() {
                        sim.skip_to(counter);
                        *regenerate_render_data = true;
                    }
                });
                ui.horizontal(|ui| {
                    let instructions = sim.instructions();
                    let start_cond = program_counter > 0;
                    let end_cond = program_counter < instructions.len();
                    let undo_cond = !*play_sim && start_cond;
                    let step_cond = !*play_sim && end_cond;
                    if ui.add_enabled(start_cond, Button::new("<<")).clicked() {
                        while sim.undo() {}
                        *regenerate_render_data = true;
                    }
                    if ui.add_enabled(undo_cond, Button::new("<")).clicked() {
                        *regenerate_render_data = sim.undo();
                    }
                    if *play_sim {
                        if ui.button("Pause").clicked() {
                            *play_sim = false;
                        }
                    } else if ui.button("Play").clicked() {
                        *play_sim = true;
                    }
                    if ui.add_enabled(step_cond, Button::new(">")).clicked() {
                        *regenerate_render_data = sim.step();
                    }
                    if ui.add_enabled(end_cond, Button::new(">>")).clicked() {
                        while sim.step() {}
                        *regenerate_render_data = true;
                    }
                });
                ui.horizontal(|ui| {
                    let mut millis = exec_duration.as_millis() as u64;
                    Slider::new(&mut millis, 1..=50).show_value(false).ui(ui);
                    *exec_duration = Duration::from_millis(millis);
                    ui.label(format!("{}ms exec rate", millis));
                });
            });
    }
}
