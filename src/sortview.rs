use core::fmt;
use std::{
    collections::HashSet,
    fmt::Write,
    num::ParseIntError,
    ops::RangeInclusive,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use cgmath::{ortho, Matrix4, SquareMatrix};
use egui::Widget;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    gui::VisualOptions,
    sim::PushSwapSim,
    vertex::{Vertex, VertexIndexPair},
};

const RANGE_MIN: i64 = i16::MIN as i64;
const RANGE_MAX: i64 = i16::MAX as i64;

#[derive(PartialEq)]
enum NumberGeneration {
    Ordered(usize),
    Random(usize),
    RandomRanged(RangeInclusive<i64>, usize),
    Arbitrary(String),
}

impl fmt::Display for NumberGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            NumberGeneration::Arbitrary(_) => "User Input",
            NumberGeneration::Random(_) => "Random Normalized",
            NumberGeneration::RandomRanged(..) => "Random from Custom Range",
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
    show_visual: bool,
    visual: VisualOptions,
    sim: PushSwapSim,
    gen_opt: NumberGeneration,
    source_opt: InstructionsSource,
    playing_sim: bool,
    last_instant: Instant,
    exec_interval: Duration,
    duration_accumulated: Duration,
    number_args: String,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            projection: Matrix4::identity(),
            regenerate_render_data: false,
            show_visual: false,
            visual: VisualOptions::default(),
            sim: Default::default(),
            gen_opt: NumberGeneration::Random(10),
            source_opt: InstructionsSource::Executable(None),
            playing_sim: false,
            last_instant: Instant::now(),
            exec_interval: Duration::from_millis(16),
            duration_accumulated: Duration::ZERO,
            number_args: String::new(),
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

    pub fn clear_color(&self) -> [f32; 3] {
        self.visual.clear_color()
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
            let o = if offset { num_range as f32 } else { 0. };
            let t = num / num_range as f32;
            let color = self.visual.color_at(t);
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

    fn get_numbers(&self) -> Result<Vec<i64>, ParseIntError> {
        match &self.gen_opt {
            NumberGeneration::Ordered(r) => Ok((0..(*r as i64)).collect()),
            NumberGeneration::Random(r) => {
                let mut nums: Vec<_> = (0..(*r as i64)).collect();
                nums.shuffle(&mut thread_rng());
                Ok(nums)
            }
            NumberGeneration::RandomRanged(r, n) => {
                let mut map = HashSet::new();
                while map.len() < *n {
                    map.insert(thread_rng().gen_range(r.clone()));
                }
                Ok(map.into_iter().collect())
            }
            NumberGeneration::Arbitrary(s) => s.split_whitespace().map(|s| s.parse()).collect(),
        }
    }

    fn get_instructions_and_numbers(&self) -> Result<(String, Vec<i64>), String> {
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
        self.number_args =
            numbers
                .iter()
                .map(|n| n.to_string())
                .fold(String::new(), |mut acc, n| {
                    let _ = write!(acc, "{} ", n);
                    acc
                });
        match self.sim.load_random(&numbers, &instructions) {
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
        ui.style_mut(|ui| {
            ui.visuals.window_fill =
                egui::Color32::from_rgba_unmultiplied(0x1b, 0x1b, 0x1b, self.visual.opacity());
        });
        egui::Window::new("Visualization Loader")
            .resizable(true)
            .movable(true)
            .collapsible(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.show_visual, "Show Visual Options Window");
                egui::ComboBox::from_label("Number Generation")
                    .selected_text(self.gen_opt.to_string())
                    .show_ui(ui, |ui| {
                        use NumberGeneration::*;
                        let (range, num_gen, str) = match &self.gen_opt {
                            Ordered(r) | Random(r) => (0..=(*r as i64 - 1), *r, String::new()),
                            RandomRanged(r, n) => (r.clone(), *n, String::new()),
                            Arbitrary(s) => (0..=9, 10, s.clone()),
                        };
                        ui.selectable_value(&mut self.gen_opt, Ordered(num_gen), "Ordered");
                        ui.selectable_value(
                            &mut self.gen_opt,
                            Random(num_gen),
                            "Random Normalized",
                        );
                        ui.selectable_value(
                            &mut self.gen_opt,
                            RandomRanged(range, num_gen),
                            "Random from Custom Range",
                        );
                        ui.selectable_value(&mut self.gen_opt, Arbitrary(str), "User Input");
                    });
                match &mut self.gen_opt {
                    NumberGeneration::Ordered(r) | NumberGeneration::Random(r) => {
                        ui.horizontal(|ui| {
                            egui::DragValue::new(r).ui(ui);
                            ui.label("Numbers to Generate");
                        });
                    }
                    NumberGeneration::RandomRanged(r, s) => {
                        ui.horizontal(|ui| {
                            egui::DragValue::new(s).ui(ui);
                            ui.label("Numbers to Generate");
                        });
                        ui.horizontal(|ui| {
                            let mut start = *r.start();
                            let mut end = *r.end();
                            ui.label("Random numbers from");
                            egui::DragValue::new(&mut start)
                                .range(RANGE_MIN..=(end - 1))
                                .clamp_existing_to_range(true)
                                .ui(ui);
                            ui.label("to");
                            egui::DragValue::new(&mut end)
                                .range((start + 1)..=RANGE_MAX)
                                .clamp_existing_to_range(true)
                                .ui(ui);
                            *r = start..=end;
                            *s = ((end - start + 1) as usize).min(*s);
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
                        self.number_args.clear();
                        self.regenerate_render_data = true;
                    }
                    if ui.button("Copy number arguments").clicked() {
                        let copy = self.number_args.clone();
                        ui.output_mut(move |o| o.copied_text = copy);
                    }
                    if ui.button("Benchmark").clicked() {
                        rfd::MessageDialog::new()
                            .set_title("Benchmarking not available")
                            .set_level(rfd::MessageLevel::Info)
                            .set_description("Benchmarking can only be done by running psv with the argument `benchmark` (aliases: `bench` `b`)")
                            .set_buttons(rfd::MessageButtons::OkCustom("Got it".into()))
                            .show();
                    }
                });
            });
        self.visual.ui(ui, &mut self.show_visual);
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
