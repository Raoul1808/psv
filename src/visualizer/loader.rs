use std::{collections::HashSet, ops::RangeInclusive, path::PathBuf};

use eframe::egui::{self, Widget};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, PartialEq)]
enum NumberGeneration {
    RandomNormalized(usize),
    RandomRanged(RangeInclusive<i32>, usize),
    UserInput(String),
}

impl NumberGeneration {
    fn get_args(&self) -> Vec<String> {
        match self {
            Self::RandomNormalized(n) => {
                let mut nums: Vec<_> = (0..*n).collect();
                nums.shuffle(&mut rand::rng());
                nums.into_iter().map(|u| u.to_string()).collect()
            }
            Self::RandomRanged(r, n) => {
                let len = *r.end() as i64 - *r.start() as i64 + 1;
                let len = (*n).min(len as usize);
                let mut set = HashSet::new();
                set.reserve(len);
                while set.len() < len {
                    set.insert(rand::random_range(r.clone()));
                }
                set.into_iter().map(|i| i.to_string()).collect()
            }
            Self::UserInput(s) => s.split_whitespace().map(str::to_string).collect(),
        }
    }

    const fn name(&self) -> &str {
        match self {
            Self::RandomNormalized(_) => "Random Normalized",
            Self::RandomRanged(_, _) => "Random with Custom Range",
            Self::UserInput(_) => "User Input",
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Number Generation")
            .selected_text(self.name())
            .show_ui(ui, |ui| {
                let (range, numbers, input) = match self {
                    Self::RandomNormalized(n) => (-256..=256, *n, String::new()),
                    Self::RandomRanged(r, n) => (r.clone(), *n, String::new()),
                    Self::UserInput(u) => (-256..=256, 10, u.clone()),
                };
                ui.selectable_value(self, Self::RandomNormalized(numbers), "Random Normalized");
                ui.selectable_value(
                    self,
                    Self::RandomRanged(range, numbers),
                    "Random with Custom Range",
                );
                ui.selectable_value(self, Self::UserInput(input.to_string()), "User Input");
            });
        match self {
            Self::RandomNormalized(numbers) => {
                ui.horizontal(|ui| {
                    ui.label("Numbers from 0 to ");
                    egui::DragValue::new(numbers).range(1..=u16::MAX).ui(ui);
                    ui.label("(exclusive)");
                });
            }
            Self::RandomRanged(range, numbers) => {
                ui.horizontal(|ui| {
                    let mut start = *range.start();
                    let mut end = *range.end();
                    egui::DragValue::new(numbers).range(1..=u16::MAX).ui(ui);
                    ui.label(format!(
                        "random number{} from",
                        if *numbers == 1 { "" } else { "s" }
                    ));
                    egui::DragValue::new(&mut start)
                        .range(i32::MIN..=(end - 1))
                        .clamp_existing_to_range(true)
                        .ui(ui);
                    ui.label("to");
                    egui::DragValue::new(&mut end)
                        .range((start + 1)..=i32::MAX)
                        .clamp_existing_to_range(true)
                        .ui(ui);
                    *range = start..=end;
                    *numbers = ((end as i64 - start as i64 + 1) as usize).min(*numbers);
                });
            }
            Self::UserInput(input) => {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(input);
                    ui.label("Numbers");
                });
            }
        }
    }
}

pub struct Loader {
    push_swap_path: Option<PathBuf>,
    number_preset: NumberGeneration,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            push_swap_path: None,
            number_preset: NumberGeneration::RandomNormalized(10),
        }
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.push_swap_path = Some(path);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let path_str = self
            .push_swap_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or("None".into());
        egui::Label::new(format!("Executable path: {}", path_str))
            .truncate()
            .ui(ui);
        self.number_preset.ui(ui);
        if ui.button("Visualize").clicked() {
            let args = self.number_preset.get_args();
            println!("args: {args:#?}");
        }
    }
}
