use std::{
    collections::HashSet,
    fmt::{Display, Write},
    fs,
    io::Read,
    num::ParseIntError,
    ops::RangeInclusive,
    os::fd::AsRawFd,
    path::PathBuf,
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

use egui::{ComboBox, Context, DragValue, ScrollArea, Widget, Window};
use rand::{Rng, rng, seq::SliceRandom};
use tokio::sync::oneshot::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;

use crate::{config::Config, sim::PushSwapSim};

use super::NUMBER_PRESETS;

const RANGE_MIN: i64 = i16::MIN as i64;
const RANGE_MAX: i64 = i16::MAX as i64;

#[derive(PartialEq, Clone)]
enum NumberGeneration {
    Ordered(usize),
    ReverseOrdered(usize),
    Random(usize),
    RandomRanged(RangeInclusive<i64>, usize),
    Arbitrary(String),
    Preset(usize),
}

fn compute_disorder(stack: &[u32]) -> f64 {
    let mut mistakes = 0;
    let mut total_pairs = 0;
    for i in 0..(stack.len() - 1) {
        for j in (i + 1)..(stack.len() - 1) {
            total_pairs += 1;
            if stack[i] > stack[j] {
                mistakes += 1;
            }
        }
    }
    return mistakes as f64 / total_pairs as f64;
}

impl NumberGeneration {
    pub fn get_numbers(&self) -> Result<Vec<i64>, ParseIntError> {
        match &self {
            NumberGeneration::Ordered(r) => Ok((0..(*r as i64)).collect()),
            NumberGeneration::ReverseOrdered(r) => Ok((0..(*r as i64)).rev().collect()),
            NumberGeneration::Random(r) => {
                let mut nums: Vec<_> = (0..(*r as i64)).collect();
                nums.shuffle(&mut rng());
                Ok(nums)
            }
            NumberGeneration::RandomRanged(r, n) => {
                let mut map = HashSet::new();
                while map.len() < *n {
                    map.insert(rng().random_range(r.clone()));
                }
                Ok(map.into_iter().collect())
            }
            NumberGeneration::Arbitrary(s) => s.split_whitespace().map(|s| s.parse()).collect(),
            NumberGeneration::Preset(i) => Ok(NUMBER_PRESETS[*i].1.to_vec()),
        }
    }
}

impl Display for NumberGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            NumberGeneration::Arbitrary(_) => "User Input",
            NumberGeneration::Random(_) => "Random Normalized",
            NumberGeneration::RandomRanged(..) => "Random from Custom Range",
            NumberGeneration::Ordered(_) => "Ordered",
            NumberGeneration::ReverseOrdered(_) => "Reverse Ordered",
            NumberGeneration::Preset(_) => "Preset",
        };
        write!(f, "{}", str)
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum SortingStrategy {
    #[default]
    None,
    Simple,
    Medium,
    Complex,
    Adaptive,
}

impl SortingStrategy {
    pub const ALL: [SortingStrategy; 5] = [
        SortingStrategy::None,
        SortingStrategy::Simple,
        SortingStrategy::Medium,
        SortingStrategy::Complex,
        SortingStrategy::Adaptive,
    ];
    pub fn to_arg(self) -> String {
        match self {
            SortingStrategy::None => "",
            SortingStrategy::Simple => "--simple",
            SortingStrategy::Medium => "--medium",
            SortingStrategy::Complex => "--complex",
            SortingStrategy::Adaptive => "--adaptive",
        }
        .to_string()
    }
}

impl Display for SortingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            SortingStrategy::None => "None",
            SortingStrategy::Simple => "Simple",
            SortingStrategy::Medium => "Medium",
            SortingStrategy::Complex => "Complex",
            SortingStrategy::Adaptive => "Adaptive",
        };
        write!(f, "{str}")
    }
}

#[derive(PartialEq, Clone)]
enum InstructionsSource {
    Manual(String),
    File(Option<PathBuf>),
    Executable {
        path: Option<PathBuf>,
        mode: SortingStrategy,
    },
}

impl Display for InstructionsSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            InstructionsSource::Manual(_) => "User Input",
            InstructionsSource::File(_) => "From File",
            InstructionsSource::Executable { .. } => "Program Output",
        };
        write!(f, "{}", str)
    }
}

struct AsyncWorker {
    receiver: Receiver<Result<PushSwapSim, String>>,
    token: CancellationToken,
    start_time: Instant,
}

enum ExecutionTimeInfo {
    None,
    Finished(Duration),
    Killed(Duration),
    Error(String),
}

pub struct LoadingOptions {
    gen_opt: NumberGeneration,
    source_opt: InstructionsSource,
    worker: Option<AsyncWorker>,
    gen_time: ExecutionTimeInfo,
    disorder: Option<f64>,
    number_args: String,
}

fn update_projection(projection: &mut cgmath::Matrix4<f32>, num_range: f32) {
    *projection = cgmath::ortho(0., num_range * 2., num_range, 0., -1., 1.);
}

// Taken from https://stackoverflow.com/a/68174244
pub fn change_blocking_fd(fd: std::os::unix::io::RawFd, blocking: bool) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(
            fd,
            libc::F_SETFL,
            if blocking {
                flags & !libc::O_NONBLOCK
            } else {
                flags | libc::O_NONBLOCK
            },
        );
    }
}

impl LoadingOptions {
    pub fn new(config: &Config) -> LoadingOptions {
        LoadingOptions {
            gen_opt: NumberGeneration::Random(10),
            source_opt: InstructionsSource::Executable {
                path: config.push_swap_path.clone(),
                mode: Default::default(),
            },
            worker: None,
            gen_time: ExecutionTimeInfo::None,
            disorder: None,
            number_args: String::new(),
        }
    }

    async fn get_instructions_and_numbers(
        token: CancellationToken,
        gen_opt: NumberGeneration,
        source_opt: InstructionsSource,
    ) -> Result<(String, Vec<i64>), String> {
        let (instructions, numbers) = match &source_opt {
            InstructionsSource::Executable { path, mode } => {
                let path = path.as_ref().ok_or("No executable selected".to_string())?;
                let numbers = gen_opt
                    .get_numbers()
                    .map_err(|err| format!("error while parsing numbers: {}", err))?;
                let args: Vec<_> = numbers.iter().map(|n| n.to_string()).collect();
                let mut cmd = Command::new(path);
                if *mode != SortingStrategy::None {
                    cmd.arg(mode.to_arg());
                }
                let mut child = cmd
                    .args(args)
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(|err| format!("error while running program: {}", err))?;
                let mut stdout = child.stdout.take().unwrap();
                change_blocking_fd(stdout.as_raw_fd(), false);
                let mut output = vec![];
                loop {
                    let wait = child.try_wait();
                    if let Ok(Some(_)) = wait {
                        change_blocking_fd(stdout.as_raw_fd(), true);
                        let _ = stdout.read_to_end(&mut output);
                        break;
                    }
                    if token.is_cancelled() {
                        let _ = child.kill();
                        break;
                    }
                    let _ = stdout.read_to_end(&mut output);
                    sleep(Duration::from_millis(10));
                }
                let instructions = String::from_utf8(output)
                    .map_err(|err| format!("failed to convert byte array to string: {}", err))?;
                (instructions, numbers)
            }
            InstructionsSource::File(path) => {
                let path = path.as_ref().ok_or("No file selected".to_string())?;
                let numbers = gen_opt
                    .get_numbers()
                    .map_err(|err| format!("error while parsing numbers: {}", err))?;
                let instructions = fs::read_to_string(path)
                    .map_err(|err| format!("failed to read from file: {}", err))?;
                (instructions, numbers)
            }
            InstructionsSource::Manual(instructions) => (
                instructions.clone(),
                gen_opt
                    .get_numbers()
                    .map_err(|err| format!("error while parsing numbers: {}", err))?,
            ),
        };
        Ok((instructions, numbers))
    }

    async fn load_sim(
        sender: Sender<Result<PushSwapSim, String>>,
        token: CancellationToken,
        gen_opt: NumberGeneration,
        source_opt: InstructionsSource,
    ) {
        let (instructions, numbers) =
            match Self::get_instructions_and_numbers(token, gen_opt, source_opt).await {
                Ok(pair) => pair,
                Err(s) => {
                    rfd::AsyncMessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Information Error")
                        .set_description(format!("An error occurred:\n{}", s))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show()
                        .await;
                    sender
                        .send(Err(format!("Loading Error: {}", s)))
                        .expect("failed to send message through channel");
                    return;
                }
            };
        let number_args =
            numbers
                .iter()
                .map(|n| n.to_string())
                .fold(String::new(), |mut acc, n| {
                    let _ = write!(acc, "{} ", n);
                    acc
                });
        let mut sim = PushSwapSim::default();
        match sim.load_random(&numbers, &instructions) {
            Ok(_) => {
                sender
                    .send(Ok(sim))
                    .expect("failed to send message through channel");
            }
            Err(line) => {
                eprintln!("Error while loading instructions!");
                eprintln!("Failed at instruction {}", line);
                eprintln!("Numbers: [{}]", number_args);
                eprintln!("List of instructions: {}", instructions);
                rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Instruction parsing error")
                    .set_description(format!(
                        "Instruction {} is not a valid push_swap instruction",
                        line
                    ))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show()
                    .await;
                sender
                    .send(Err(format!("Parsing Error at instruction {}", line)))
                    .expect("failed to send message through channel");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ctx: &Context,
        config: &mut Config,
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
                        Ordered(r) | ReverseOrdered(r) | Random(r) => (0..=(*r as i64 - 1), *r, String::new(), 0),
                        RandomRanged(r, n) => (r.clone(), *n, String::new(), 0),
                        Arbitrary(s) => (0..=9, 10, s.clone(), 0),
                        Preset(i) => (0..=9, 10, String::new(), *i),
                    };
                    ui.selectable_value(&mut self.gen_opt, Ordered(num_gen), "Ordered").on_hover_text("Numbers will be generated in order from 0 to n.");
                    ui.selectable_value(&mut self.gen_opt, ReverseOrdered(num_gen), "Reverse Ordered").on_hover_text("Numbers will be generated in reverse order from n to 0.");
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
                NumberGeneration::Ordered(r) | NumberGeneration::ReverseOrdered(r) | NumberGeneration::Random(r) => {
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
                    let (ins, file_path, exe_path) = match &self.source_opt {
                        Manual(i) => (i.clone(), None, None),
                        File(p) => (String::new(), p.clone(), None),
                        Executable { path, .. } => (String::new(), None, path.clone()),
                    };
                    ui.selectable_value(&mut self.source_opt, Manual(ins), "User Input").on_hover_text("You will be able to input a list of push_swap instructions yourself.");
                    ui.selectable_value(&mut self.source_opt, File(file_path), "From File").on_hover_text("The selected file's contents will be interpreted as a list of push_swap instructions.");
                    ui.selectable_value(&mut self.source_opt, Executable { path: exe_path, mode: Default::default() }, "Program Output").on_hover_text("The selected program will be executed with the generated numbers above fed as input to the program. The output of the program will be interpreted as a list of push_swap instructions.");
                });
            match &mut self.source_opt {
                InstructionsSource::Manual(i) => {
                    ui.label("Type push_swap instructions below");
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.add_sized([300., 5.], egui::TextEdit::multiline(i));
                    });
                }
                InstructionsSource::File(p) => {
                    ui.horizontal(|ui| {
                        if ui.button("Browse").clicked() {
                            let path = rfd::FileDialog::new()
                                .set_title("Select file")
                                .pick_file();
                            if let Some(path) = path {
                                *p = Some(path);
                            }
                        }
                        let path = p
                            .clone()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or("None".into());
                        ui.label(format!("Selected File: {}", path));
                    });
                }
                InstructionsSource::Executable { path, mode } => {
                    ui.horizontal(|ui| {
                        if ui.button("Browse").clicked() {
                            let p = rfd::FileDialog::new()
                                .set_title("Select push_swap executable")
                                .pick_file();
                            if let Some(p) = p {
                                *path = Some(p.clone());
                                config.push_swap_path = Some(p);
                                config.save()
                            }
                        }
                        let path = path
                            .clone()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or("None".into());
                        ui.label(format!("Selected Program: {}", path));
                    });
                    ComboBox::from_label("Sorting Strategy")
                        .selected_text(mode.to_string())
                        .show_ui(ui, |ui| {
                            for m in SortingStrategy::ALL {
                                ui.selectable_value(mode, m, m.to_string());
                            }
                        });
                }
            };
            ui.separator();
            let mut clear_worker = false;
            if let Some(worker) = self.worker.as_mut() && let Ok(res) = worker.receiver.try_recv() {
                let now = Instant::now();
                let duration = now - worker.start_time;
                clear_worker = true;
                match res {
                    Ok(res) => {
                        *sim = res;
                        self.disorder = Some(compute_disorder(sim.stack_a()));
                        self.gen_time = if worker.token.is_cancelled() {
                            ExecutionTimeInfo::Killed(duration)
                        } else {
                            ExecutionTimeInfo::Finished(duration)
                        };
                        *regenerate_render_data = true;
                        *show_playback = true;
                        update_projection(projection, sim.amount() as f32);
                    }
                    Err(e) => {
                        self.gen_time = ExecutionTimeInfo::Error(e);
                    }
                }
            }
            if clear_worker {
                let _ = self.worker.take();
            }
            ui.horizontal(|ui| {
                match self.worker.as_ref() {
                    None => {
                        if ui.button("Visualize").clicked() {
                            let (sender, receiver) = channel();
                            let token = CancellationToken::new();
                            let token_clone = token.clone();
                            let gen_clone = self.gen_opt.clone();
                            let source_clone = self.source_opt.clone();
                            let start_time = Instant::now();
                            self.worker = Some(AsyncWorker { receiver, token, start_time });
                            tokio::spawn(async move {
                                Self::load_sim(sender, token_clone, gen_clone, source_clone).await;
                            });
                        }
                    }
                    Some(worker) => {
                        ui.spinner();
                        if ui.button("Kill").clicked() {
                            worker.token.cancel();
                        }
                    }
                }
                if ui.button("Clear").clicked() {
                    sim.clear();
                    self.number_args.clear();
                    *regenerate_render_data = true;
                    *playing_sim = false;
                    *show_playback = false;
                    self.gen_time = ExecutionTimeInfo::None;
                    self.disorder = None;
                }
                if ui.button("Copy numbers to clipboard").on_hover_text("The list of generated numbers will be collapsed into a single line that can be pasted as program arguments. Useful if you want to debug a random sequence that was just generated.").clicked() {
                    let copy = self.number_args.clone();
                    ui.ctx().copy_text(copy);
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
            if let Some(worker) = self.worker.as_ref() {
                let now = Instant::now();
                let duration = now - worker.start_time;
                ui.label(format!("push_swap is running. Execution time: {:.3} seconds", duration.as_secs_f64()));
            } else {
                match &self.gen_time {
                    ExecutionTimeInfo::None => {},
                    ExecutionTimeInfo::Finished(d) => {
                        ui.label(format!("Execution finished. Took {:.3} seconds", d.as_secs_f64()));
                    }
                    ExecutionTimeInfo::Killed(d) => {
                        ui.label(format!("Execution aborted. Killed after {:.3} seconds", d.as_secs_f64()));
                    }
                    ExecutionTimeInfo::Error(e) => {
                        ui.label(e);
                    }
                }
            }
            if let Some(dis) = self.disorder {
                ui.label(format!("Disorder: {:.2}%", dis * 100.));
            }
        });
    }
}
