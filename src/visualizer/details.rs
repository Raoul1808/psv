use std::cmp::Ordering;

use eframe::egui::{self, Widget};

use crate::sim::Sim;

pub struct Details {
    skip_to: Option<usize>,
    follow_instructions: bool,
    follow_this_frame: bool,
}

impl Details {
    pub fn new() -> Self {
        Self {
            skip_to: None,
            follow_instructions: true,
            follow_this_frame: false,
        }
    }

    pub fn can_skip(&self) -> bool {
        self.skip_to.is_some()
    }

    pub fn skip_to(&mut self) -> Option<usize> {
        self.skip_to.take()
    }

    pub fn mark_follow_this_frame(&mut self) {
        if self.follow_instructions {
            self.follow_this_frame = true;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, sim: &Sim) {
        ui.heading("Sorting Details");
        ui.separator();

        let instructions = sim.instructions();

        ui.label(format!("Instruction count: {}", instructions.len()));
        ui.label(format!("Stack sorted: {}", sim.is_sorted()));

        let (stack_a, stack_b) = sim.as_slices();
        ui.label(format!("Stack A current length: {}", stack_a.len()));
        ui.label(format!("Stack A current length: {}", stack_b.len()));

        ui.separator();

        ui.checkbox(&mut self.follow_instructions, "Auto-scroll");
        let scroll = ui.button("Scroll to current instruction").clicked();

        let available_height = ui.available_height();
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        let mut table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(egui::Sense::click())
            .animate_scrolling(false);

        if scroll || self.follow_this_frame {
            table = table.scroll_to_row(sim.program_counter() + 1, Some(egui::Align::Center));
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(" ");
                });
                header.col(|ui| {
                    ui.strong("nb");
                });
                header.col(|ui| {
                    ui.strong("Instruction");
                });
            })
            .body(|ui| {
                ui.rows(text_height, instructions.len() + 1, |mut row| {
                    let row_index = row.index();
                    let end = row_index == instructions.len();
                    row.set_selected(row_index == sim.program_counter());
                    row.col(|ui| {
                        let label = match row_index.cmp(&sim.program_counter()) {
                            Ordering::Less => "✔",
                            Ordering::Equal => {
                                if end {
                                    ""
                                } else {
                                    "➡"
                                }
                            }
                            Ordering::Greater => "⚪",
                        };
                        egui::Label::new(label).selectable(false).ui(ui);
                    });
                    row.col(|ui| {
                        if !end {
                            egui::Label::new((row_index + 1).to_string())
                                .selectable(false)
                                .ui(ui);
                        }
                    });
                    row.col(|ui| {
                        let str = if end {
                            "End of program".into()
                        } else {
                            instructions[row_index].to_string()
                        };
                        egui::Label::new(str).selectable(false).ui(ui);
                    });
                    if row.response().clicked() {
                        self.skip_to = Some(row_index);
                    }
                });
            });
        self.follow_this_frame = false;
    }
}
