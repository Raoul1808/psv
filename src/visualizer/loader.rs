use std::{collections::HashSet, fmt::Write, ops::RangeInclusive, path::PathBuf};

use eframe::egui::{self, Widget};
use rand::seq::SliceRandom;

use crate::{common::SortingStrategy, sim::PushSwapInstruction, visualizer::exec::PushSwapExec};

#[derive(Debug, Clone, PartialEq)]
enum NumberGeneration {
    RandomNormalized(usize),
    RandomRanged(RangeInclusive<i32>, usize),
    UserInput(String),
}

impl NumberGeneration {
    fn get_numbers(&self) -> Result<Vec<i32>, String> {
        match self {
            Self::RandomNormalized(n) => {
                let mut nums: Vec<_> = (0..*n as i32).collect();
                nums.shuffle(&mut rand::rng());
                Ok(nums)
            }
            Self::RandomRanged(r, n) => {
                let len = *r.end() as i64 - *r.start() as i64 + 1;
                let len = (*n).min(len as usize);
                let mut set = HashSet::new();
                set.reserve(len);
                while set.len() < len {
                    set.insert(rand::random_range(r.clone()));
                }
                Ok(set.into_iter().collect())
            }
            Self::UserInput(s) => s
                .split_whitespace()
                .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
                .collect(),
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
    push_swap_path: PathBuf,
    number_preset: NumberGeneration,
    sorting_strategy: SortingStrategy,
    exec: Option<PushSwapExec>,
    run_msg: Option<String>,
    numbers: Vec<i32>,
    instructions: Option<Vec<PushSwapInstruction>>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            push_swap_path: PathBuf::new(),
            number_preset: NumberGeneration::RandomNormalized(10),
            sorting_strategy: SortingStrategy::default(),
            exec: None,
            run_msg: None,
            numbers: vec![],
            instructions: None,
        }
    }

    pub fn clear(&mut self) {
        if let Some(exec) = self.exec.take() {
            exec.cancel();
            let _ = exec.finish();
        }
        self.number_preset = NumberGeneration::RandomNormalized(10);
        self.sorting_strategy = SortingStrategy::default();
        self.run_msg = None;
        self.numbers.clear();
        self.instructions = None;
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.push_swap_path = path;
    }

    pub fn poll_exec(&mut self) {
        if let Some(exec) = self.exec.as_mut() {
            exec.poll_process();
        }
        if self.exec.as_ref().is_some_and(|e| e.is_finished()) {
            let exec = self.exec.take().unwrap();
            let res = exec.finish();
            match res {
                Ok(e) => {
                    self.instructions = Some(e.instructions);
                    self.numbers = e.numbers;
                    self.run_msg = Some(format!(
                        "Execution finished! Took {}s",
                        e.duration.as_secs_f64()
                    ));
                }
                Err(e) => {
                    log::error!("error during push_swap execution: {e}");
                    self.run_msg = Some(format!("Execution failed: {e}"));
                }
            }
        }
    }

    pub fn instructions_ready(&self) -> bool {
        self.instructions.is_some()
    }

    pub fn take_instructions(&mut self) -> Option<Vec<PushSwapInstruction>> {
        self.instructions.take()
    }

    pub fn numbers(&self) -> &[i32] {
        self.numbers.as_slice()
    }

    pub fn is_working(&self) -> bool {
        self.exec.is_some()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.poll_exec();

        let path_str = self.push_swap_path.display();
        egui::Label::new(format!("Executable path: {}", path_str))
            .truncate()
            .ui(ui);
        self.number_preset.ui(ui);
        self.sorting_strategy.ui(ui);
        match self.exec.as_ref() {
            Some(exec) => {
                if ui.button("Kill").clicked() {
                    exec.cancel();
                }
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Running push_swap...");
                });
            }
            None => {
                if ui.button("Visualize").clicked() {
                    let path = self.push_swap_path.clone();
                    match self.number_preset.get_numbers() {
                        Ok(n) => {
                            let exec = PushSwapExec::new(path, n, self.sorting_strategy);
                            self.exec = Some(exec);
                        }
                        Err(e) => {
                            log::error!("failed to get numbers: {e}");
                            self.run_msg = Some(e.to_string());
                        }
                    };
                }
                if !self.numbers.is_empty() && ui.button("Rerun with same numbers").clicked() {
                    let path = self.push_swap_path.clone();
                    let exec = PushSwapExec::new(path, self.numbers.clone(), self.sorting_strategy);
                    self.exec = Some(exec);
                }
                if let Some(msg) = self.run_msg.as_ref() {
                    ui.label(msg);
                }
            }
        }
        // TODO: reorder UI
        if !self.numbers.is_empty() && ui.button("Copy numbers to clipboard").clicked() {
            let nums = self.numbers.iter().fold(String::new(), |mut acc, n| {
                let _ = write!(acc, "{n} ");
                acc
            });
            let nums = nums.trim_end();
            ui.copy_text(nums.into());
            println!("Copied numbers to clipboard!");
        }
    }
}
