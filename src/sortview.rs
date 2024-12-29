use core::fmt;
use std::{
    cmp::Ordering,
    num::ParseIntError,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use cgmath::{ortho, Matrix4, SquareMatrix};
use egui::Widget;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    sim::PushSwapSim,
    vertex::{Vertex, VertexIndexPair},
};

#[derive(PartialEq)]
enum NumberGeneration {
    Ordered(usize),
    Random(usize),
    Arbitrary(String),
}

impl fmt::Display for NumberGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            NumberGeneration::Arbitrary(_) => "User Input",
            NumberGeneration::Random(_) => "Random",
            NumberGeneration::Ordered(_) => "Ordered",
        };
        write!(f, "{}", str)
    }
}

#[derive(PartialEq)]
enum InstructionsSource {
    Manual(String),
    Executable(Option<PathBuf>),
}

impl fmt::Display for InstructionsSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            InstructionsSource::Manual(_) => "User Input",
            InstructionsSource::Executable(_) => "Program Output",
        };
        write!(f, "{}", str)
    }
}

pub struct SortView {
    projection: Matrix4<f32>,
    regenerate_render_data: bool,
    sim: PushSwapSim,
    gen_opt: NumberGeneration,
    source_opt: InstructionsSource,
    playing_sim: bool,
    last_instant: Instant,
    exec_interval: Duration,
    duration_accumulated: Duration,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            projection: Matrix4::identity(),
            regenerate_render_data: false,
            sim: Default::default(),
            gen_opt: NumberGeneration::Random(10),
            source_opt: InstructionsSource::Executable(None),
            playing_sim: false,
            last_instant: Instant::now(),
            exec_interval: Duration::from_millis(16),
            duration_accumulated: Duration::ZERO,
        }
    }

    pub fn get_tris_data(&mut self) -> Option<VertexIndexPair> {
        if self.regenerate_render_data {
            self.regenerate_render_data = false;
            self.sim.make_contiguous();
            let stack_a = self.sim.stack_a();
            let stack_b = self.sim.stack_b();
            let num_range = stack_a.len() as u32 + stack_b.len() as u32;
            let mut data = self.generate_tris_data(num_range, stack_a, false);
            data.extend(self.generate_tris_data(num_range, stack_b, true));
            Some(data)
        } else {
            None
        }
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    fn update_projection(&mut self, num_range: u32) {
        self.projection = ortho(0., num_range as f32 * 2., num_range as f32, 0., -1., 1.);
    }

    fn generate_tris_data(&self, num_range: u32, stack: &[u32], offset: bool) -> VertexIndexPair {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut next_index = 0;
        for (i, num) in stack.iter().enumerate() {
            let i = i as f32;
            let num = (*num) as f32;
            let half_range = num_range as f32 / 2.;
            let color = match num.partial_cmp(&half_range) {
                Some(Ordering::Equal) => [1.0, 1.0, 0.0],
                Some(Ordering::Less) => {
                    let half = num_range as f32 / 2.;
                    [1.0, num / half, 0.0]
                }
                Some(Ordering::Greater) | None => {
                    let half = num_range as f32 / 2.;
                    [1. - (num - half) / half, 1.0, 0.0]
                }
            };
            let o = if offset { num_range as f32 } else { 0. };
            vertices.push(Vertex {
                position: [0.0 + o, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0 + o, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0 + o, i + 1.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [0.0 + o, i + 1.0, 0.0],
                color,
            });
            indices.extend_from_slice(&[
                next_index,
                next_index + 1,
                next_index + 2,
                next_index + 2,
                next_index + 3,
                next_index,
            ]);
            next_index += 4;
        }
        VertexIndexPair { vertices, indices }
    }

    fn get_numbers(&self) -> Result<Vec<u32>, ParseIntError> {
        match &self.gen_opt {
            NumberGeneration::Ordered(r) => Ok((0..(*r as u32)).collect()),
            NumberGeneration::Random(r) => {
                let mut nums: Vec<_> = (0..(*r as u32)).collect();
                nums.shuffle(&mut thread_rng());
                Ok(nums)
            }
            NumberGeneration::Arbitrary(s) => s.split_whitespace().map(|s| s.parse()).collect(),
        }
    }

    fn get_instructions_and_numbers(&self) -> Result<(String, Vec<u32>), String> {
        let (instructions, numbers) = match &self.source_opt {
            InstructionsSource::Executable(path) => {
                let path = path.as_ref().ok_or("No executable selected".to_string())?;
                let numbers = self
                    .get_numbers()
                    .map_err(|err| format!("error while parsing numbers: {}", err))?;
                let args: Vec<_> = numbers.iter().map(|n| n.to_string()).collect();
                let output = Command::new(path)
                    .args(args)
                    .output()
                    .map_err(|err| format!("error while running program: {}", err))?
                    .stdout;
                let instructions = String::from_utf8(output)
                    .map_err(|err| format!("failed to convert byte array to string: {}", err))?;
                (instructions, numbers)
            }
            InstructionsSource::Manual(instructions) => (
                instructions.clone(),
                self.get_numbers()
                    .map_err(|err| format!("error while parsing numbers: {}", err))?,
            ),
        };
        Ok((instructions, numbers))
    }

    fn load_sim(&mut self) {
        let (instructions, numbers) = match self.get_instructions_and_numbers() {
            Ok(pair) => pair,
            Err(s) => {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Information Error")
                    .set_description(format!("An error occurred:\n{}", s))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
                return;
            }
        };
        match self.sim.load(&numbers, &instructions) {
            Ok(_) => {}
            Err(line) => {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Instruction parsing error")
                    .set_description(format!(
                        "Instruction {} is not a valid push_swap instruction",
                        line
                    ))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            }
        };
        self.update_projection(numbers.len() as u32);
        self.regenerate_render_data = true;
        self.playing_sim = false;
    }

    pub fn egui_menu(&mut self, ui: &egui::Context) {
        egui::Window::new("Visualization Loader")
            .resizable(true)
            .movable(true)
            .collapsible(true)
            .show(ui, |ui| {
                egui::ComboBox::from_label("Number Generation")
                    .selected_text(self.gen_opt.to_string())
                    .show_ui(ui, |ui| {
                        use NumberGeneration::*;
                        let (num_gen, str) = match &self.gen_opt {
                            Ordered(r) | Random(r) => (*r, String::new()),
                            Arbitrary(s) => (10, s.clone()),
                        };
                        ui.selectable_value(&mut self.gen_opt, Ordered(num_gen), "Ordered");
                        ui.selectable_value(&mut self.gen_opt, Random(num_gen), "Random");
                        ui.selectable_value(&mut self.gen_opt, Arbitrary(str), "User Input");
                    });
                match &mut self.gen_opt {
                    NumberGeneration::Ordered(r) | NumberGeneration::Random(r) => {
                        ui.horizontal(|ui| {
                            egui::DragValue::new(r).ui(ui);
                            ui.label("Numbers to Generate");
                        });
                    }
                    NumberGeneration::Arbitrary(s) => {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(s);
                            ui.label("Numbers");
                        });
                    }
                };
                egui::ComboBox::from_label("Instructions Source")
                    .selected_text(self.source_opt.to_string())
                    .show_ui(ui, |ui| {
                        use InstructionsSource::*;
                        let (ins, path) = match &self.source_opt {
                            Manual(i) => (i.clone(), None),
                            Executable(p) => (String::new(), p.clone()),
                        };
                        ui.selectable_value(&mut self.source_opt, Manual(ins), "User Input");
                        ui.selectable_value(
                            &mut self.source_opt,
                            Executable(path),
                            "Program Output",
                        );
                    });
                match &mut self.source_opt {
                    InstructionsSource::Manual(i) => {
                        ui.label("Type push_swap instructions below");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_sized([300., 5.], egui::TextEdit::multiline(i));
                        });
                    }
                    InstructionsSource::Executable(p) => {
                        ui.horizontal(|ui| {
                            if ui.button("Browse").clicked() {
                                let path = rfd::FileDialog::new()
                                    .set_title("Select push_swap executable")
                                    .pick_file();
                                if let Some(path) = path {
                                    *p = Some(path);
                                }
                            }
                            let path = p
                                .clone()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or("None".into());
                            ui.label(format!("Selected: {}", path));
                        });
                    }
                };
                ui.horizontal(|ui| {
                    if ui.button("Visualize").clicked() {
                        self.load_sim();
                    }
                    if ui.button("Clear").clicked() {
                        self.sim.clear();
                        self.regenerate_render_data = true;
                    }
                    // TODO: Add benchmarking option
                });
            });
        if self
            .sim
            .ui(ui, &mut self.playing_sim, &mut self.exec_interval)
        {
            self.regenerate_render_data = true;
        }
        if self.playing_sim {
            let current_instant = Instant::now();
            let catching_duration = current_instant.duration_since(self.last_instant);
            while self.duration_accumulated <= catching_duration {
                self.sim.step();
                self.regenerate_render_data = true;
                self.duration_accumulated += self.exec_interval;
            }
            self.duration_accumulated -= catching_duration;
        } else {
            self.duration_accumulated = Duration::ZERO;
        }
        self.last_instant = Instant::now();
    }
}
