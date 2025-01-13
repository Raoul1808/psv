use std::{
    collections::HashSet,
    fmt::{Display, Write},
    fs,
    num::ParseIntError,
    ops::RangeInclusive,
    path::PathBuf,
    process::Command,
};

use egui::{ComboBox, Context, DragValue, ScrollArea, Widget, Window};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::sim::PushSwapSim;

use super::NUMBER_PRESETS;

const RANGE_MIN: i64 = i16::MIN as i64;
const RANGE_MAX: i64 = i16::MAX as i64;

#[derive(PartialEq)]
enum NumberGeneration {
    Ordered(usize),
    Random(usize),
    RandomRanged(RangeInclusive<i64>, usize),
    Arbitrary(String),
    Preset(usize),
}

impl Display for NumberGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            NumberGeneration::Arbitrary(_) => "User Input",
            NumberGeneration::Random(_) => "Random Normalized",
            NumberGeneration::RandomRanged(..) => "Random from Custom Range",
            NumberGeneration::Ordered(_) => "Ordered",
            NumberGeneration::Preset(_) => "Preset",
        };
        write!(f, "{}", str)
    }
}

#[derive(PartialEq)]
enum InstructionsSource {
    Manual(String),
    Executable(Option<PathBuf>),
}

impl Display for InstructionsSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            InstructionsSource::Manual(_) => "User Input",
            InstructionsSource::Executable(_) => "Program Output",
        };
        write!(f, "{}", str)
    }
}

pub struct LoadingOptions {
    gen_opt: NumberGeneration,
    source_opt: InstructionsSource,
    number_args: String,
}

impl Default for LoadingOptions {
    fn default() -> Self {
        let push_swap = if let Ok(path) = fs::canonicalize("push_swap") {
            Some(path)
        } else {
            None
        };
        Self {
            gen_opt: NumberGeneration::Random(10),
            source_opt: InstructionsSource::Executable(push_swap),
            number_args: String::new(),
        }
    }
}

fn update_projection(projection: &mut cgmath::Matrix4<f32>, num_range: f32) {
    *projection = cgmath::ortho(0., num_range * 2., num_range, 0., -1., 1.);
}

impl LoadingOptions {
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
            NumberGeneration::Preset(i) => Ok(NUMBER_PRESETS[*i].1.to_vec()),
        }
    }

    pub fn get_instructions_and_numbers(&self) -> Result<(String, Vec<i64>), String> {
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

    fn load_sim(&mut self, sim: &mut PushSwapSim, projection: &mut cgmath::Matrix4<f32>) {
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
        match sim.load_random(&numbers, &instructions) {
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
        update_projection(projection, numbers.len() as f32);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ctx: &Context,
        open: &mut bool,
        sim: &mut PushSwapSim,
        regenerate_render_data: &mut bool,
        projection: &mut cgmath::Matrix4<f32>,
        playing_sim: &mut bool,
        show_playback: &mut bool,
    ) {
        Window::new("Loading Options").open(open).show(ctx, |ui| {
            ComboBox::from_label("Number Generation")
                .selected_text(self.gen_opt.to_string())
                .show_ui(ui, |ui| {
                    use NumberGeneration::*;
                    let (range, num_gen, str, i) = match &self.gen_opt {
                        Ordered(r) | Random(r) => (0..=(*r as i64 - 1), *r, String::new(), 0),
                        RandomRanged(r, n) => (r.clone(), *n, String::new(), 0),
                        Arbitrary(s) => (0..=9, 10, s.clone(), 0),
                        Preset(i) => (0..=9, 10, String::new(), *i),
                    };
                    ui.selectable_value(&mut self.gen_opt, Ordered(num_gen), "Ordered").on_hover_text("Numbers will be generated in order from 0 to n.");
                    ui.selectable_value(&mut self.gen_opt, Random(num_gen), "Random Normalized").on_hover_text("Numbers will be generated from 0 to n, then they will be shuffled.");
                    ui.selectable_value(
                        &mut self.gen_opt,
                        RandomRanged(range, num_gen),
                        "Random from Custom Range",
                    ).on_hover_text("Numbers will be picked randomly from the specified range. Visually, the numbers will appear normalized.");
                    ui.selectable_value(&mut self.gen_opt, Arbitrary(str), "User Input").on_hover_text("You will be able to input a list of numbers yourself.");
                    ui.selectable_value(&mut self.gen_opt, Preset(i), "Preset").on_hover_text("Numbers will be selected from a few hardcoded presets. This option was added just for fun, but some of the tests in here are known to break some programs.");
                });
            match &mut self.gen_opt {
                NumberGeneration::Ordered(r) | NumberGeneration::Random(r) => {
                    ui.horizontal(|ui| {
                        DragValue::new(r).ui(ui);
                        ui.label("Numbers to Generate");
                    });
                }
                NumberGeneration::RandomRanged(r, s) => {
                    ui.horizontal(|ui| {
                        DragValue::new(s).ui(ui);
                        ui.label("Numbers to Generate");
                    });
                    ui.horizontal(|ui| {
                        let mut start = *r.start();
                        let mut end = *r.end();
                        ui.label("Random numbers from");
                        DragValue::new(&mut start)
                            .range(RANGE_MIN..=(end - 1))
                            .clamp_existing_to_range(true)
                            .ui(ui);
                        ui.label("to");
                        DragValue::new(&mut end)
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
                NumberGeneration::Preset(i) => {
                    ComboBox::from_label("Number Preset")
                        .selected_text(NUMBER_PRESETS[*i].0)
                        .show_index(ui, i, NUMBER_PRESETS.len(), |i| {
                            NUMBER_PRESETS[i].0
                        });
                }
            };
            ComboBox::from_label("Instructions Source")
                .selected_text(self.source_opt.to_string())
                .show_ui(ui, |ui| {
                    use InstructionsSource::*;
                    let (ins, path) = match &self.source_opt {
                        Manual(i) => (i.clone(), None),
                        Executable(p) => (String::new(), p.clone()),
                    };
                    ui.selectable_value(&mut self.source_opt, Manual(ins), "User Input").on_hover_text("You will be able to input a list of push_swap instructions yourself.");
                    ui.selectable_value(&mut self.source_opt, Executable(path), "Program Output").on_hover_text("The selected program will be executed with the generated numbers above fed as input to the program. The output of the program will be interpreted as a list of push_swap instructions.");
                });
            match &mut self.source_opt {
                InstructionsSource::Manual(i) => {
                    ui.label("Type push_swap instructions below");
                    ScrollArea::vertical().show(ui, |ui| {
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
                        ui.label(format!("Selected Program: {}", path));
                    });
                }
            };
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Visualize").clicked() {
                    self.load_sim(sim, projection);
                    *regenerate_render_data = true;
                    *playing_sim = false;
                    *show_playback = true;
                }
                if ui.button("Clear").clicked() {
                    sim.clear();
                    self.number_args.clear();
                    *regenerate_render_data = true;
                    *playing_sim = false;
                    *show_playback = false;
                }
                if ui.button("Copy numbers to clipboard").on_hover_text("The list of generated numbers will be collapsed into a single line that can be pasted as program arguments. Useful if you want to debug a random sequence that was just generated.").clicked() {
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
    }
}
