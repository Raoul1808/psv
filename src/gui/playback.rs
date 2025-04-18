use std::{cmp::Ordering, time::Duration};

use egui::{Align, Button, DragValue, Layout, Sense, Ui, Widget, Window};

use crate::sim::PushSwapSim;

pub struct PlaybackControls {
    auto_scroll_table: bool,
    force_scroll: bool,
}

impl Default for PlaybackControls {
    fn default() -> Self {
        Self {
            auto_scroll_table: true,
            force_scroll: false,
        }
    }
}

impl PlaybackControls {
    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        sim: &mut PushSwapSim,
        play_sim: &mut bool,
        temp_stop_sim: &mut bool,
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
                        if secs >= 3600 {
                            format!(
                                "Estimated Time Remaining: {}h{}m{}s",
                                secs / 3600,
                                (secs / 60) % 60,
                                secs % 60
                            )
                        } else if secs >= 60 {
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
                    *temp_stop_sim = slider.is_pointer_button_down_on();
                });
                ui.horizontal(|ui| {
                    let instructions_len = sim.instructions().len();
                    let start_cond = program_counter > 0;
                    let end_cond = program_counter < instructions_len;
                    let reached_end = program_counter == instructions_len;
                    let undo_cond = !*play_sim && start_cond;
                    let step_cond = !*play_sim && end_cond;
                    if reached_end {
                        *play_sim = false;
                    }
                    ui.scope(|ui| {
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Button,
                            egui::FontId::new(24., egui::epaint::FontFamily::Proportional),
                        );
                        if ui.add_enabled(start_cond, Button::new("⏮")).clicked() {
                            *regenerate_render_data = sim.skip_to(0);
                        }
                        if ui
                            .add_enabled(undo_cond, Button::new("⏴"))
                            .on_hover_text("You can press the right arrow key to step backwards.")
                            .clicked()
                        {
                            *regenerate_render_data = sim.undo();
                        }
                        if instructions_len == 0 {
                            ui.add_enabled(false, Button::new("⏹"));
                        } else if reached_end {
                            if ui
                                .button("↺")
                                .on_hover_text("You can press spacebar to replay the simulation.")
                                .clicked()
                            {
                                sim.skip_to(0);
                                *play_sim = true;
                            }
                        } else {
                            let button_text = if *play_sim { "⏸" } else { "▶" };
                            if ui
                                .button(button_text)
                                .on_hover_text(
                                    "You can press spacebar to play/pause the simulation.",
                                )
                                .clicked()
                            {
                                *play_sim = !*play_sim;
                            }
                        }
                        if ui
                            .add_enabled(step_cond, Button::new("⏵"))
                            .on_hover_text("You can press the left arrow key to step forward.")
                            .clicked()
                        {
                            *regenerate_render_data = sim.step();
                        }
                        if ui.add_enabled(end_cond, Button::new("⏭")).clicked() {
                            *regenerate_render_data = sim.skip_to(instructions_len);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    let mut exec_rate = (1. / exec_duration.as_secs_f64()).round() as u32;
                    let speed_cap = 60.max(sim.instructions().len() / 2);
                    let speed = 10_f64.powf(0_f64.max(exec_rate.ilog10() as f64 - 2.));
                    DragValue::new(&mut exec_rate)
                        .speed(speed)
                        .range(1..=speed_cap)
                        .ui(ui);
                    ui.label("instructions per second");
                    *exec_duration = Duration::from_secs_f64(1. / exec_rate as f64);
                });
                ui.separator();
                ui.collapsing("push_swap instruction flow", |ui| {
                    self.instructions_table_ui(ui, sim, *play_sim, regenerate_render_data);
                });
            });
    }

    fn instructions_table_ui(
        &mut self,
        ui: &mut Ui,
        sim: &mut PushSwapSim,
        playing_sim: bool,
        regenerate_render_data: &mut bool,
    ) {
        use egui_extras::{Column, TableBuilder};

        let mut scroll_now = false;
        if ui.button("Focus current instruction").clicked() {
            scroll_now = true;
        }
        ui.checkbox(
            &mut self.auto_scroll_table,
            "Auto-scroll to current instruction when playing",
        );
        ui.add_enabled(
            self.auto_scroll_table,
            egui::Checkbox::new(
                &mut self.force_scroll,
                "Always scroll to current instruction at all times",
            ),
        );
        if !self.auto_scroll_table {
            self.force_scroll = false;
        }
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(Sense::click())
            .animate_scrolling(false);

        if scroll_now || self.force_scroll || (playing_sim && self.auto_scroll_table) {
            table = table.scroll_to_row(sim.program_counter() + 1, Some(Align::Center));
        }

        let total_instructions = sim.instructions().len();
        let mut skip = None;
        table
            .header(10.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.strong("Index");
                });
                header.col(|ui| {
                    ui.strong("Instruction");
                });
            })
            .body(|body| {
                body.rows(text_height, total_instructions + 1, |mut row| {
                    let row_index = row.index();
                    let reached_end = row_index == total_instructions;
                    row.set_selected(row_index == sim.program_counter());
                    row.col(|ui| match row_index.cmp(&sim.program_counter()) {
                        Ordering::Less => {
                            ui.label("✔");
                        }
                        Ordering::Equal => {
                            if !reached_end {
                                ui.label("➡");
                            }
                        }
                        Ordering::Greater => {
                            ui.label("⚪");
                        }
                    });
                    row.col(|ui| {
                        if !reached_end {
                            ui.label(format!("{}", row_index + 1));
                        }
                    });
                    row.col(|ui| {
                        if reached_end {
                            ui.label("End of program");
                        } else {
                            ui.label(sim.instructions()[row_index].to_string());
                        }
                    });
                    if row.response().clicked() {
                        skip = Some(row_index);
                    }
                });
            });
        if let Some(index) = skip {
            sim.skip_to(index);
            *regenerate_render_data = true;
        }
    }
}
