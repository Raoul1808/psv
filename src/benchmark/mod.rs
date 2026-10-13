use std::path::PathBuf;

use eframe::egui::{self, Widget};

use crate::benchmark::jobs::JobManager;

mod jobs;

enum JobResultType {
    Success { instructions: usize },
    Failure { numbers: Vec<i32> },
    Error(String),
}

struct JobResult {
    id: usize,
    ty: JobResultType,
}

const THREADS_WARNING_THRESHOLD: usize = 8;
const MIN_THREADS: usize = 1;

#[derive(Debug, Clone, Copy)]
struct Metrics {
    min: usize,
    max: usize,
    avg: usize,
    med: f64,
    deviation: f64,
}

pub struct Benchmark {
    push_swap_path: PathBuf,
    job_manager: Option<JobManager>,
    job_results: Vec<JobResult>,
    numbers: usize,
    test_amount: usize,
    num_threads: usize,
    bench_result: Option<Metrics>,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            push_swap_path: PathBuf::new(),
            job_manager: None,
            job_results: vec![],
            numbers: 10,
            test_amount: 10,
            num_threads: 4,
            bench_result: None,
        }
    }

    pub fn update_path(&mut self, path: PathBuf) {
        self.push_swap_path = path;
    }

    pub fn is_benchmarking(&self) -> bool {
        self.job_manager.is_some()
    }

    fn send_jobs(&mut self) {
        self.job_results.clear();
        self.bench_result = None;
        let jobs = JobManager::run(
            self.push_swap_path.clone(),
            self.numbers,
            self.test_amount,
            self.num_threads,
        );
        self.job_manager = Some(jobs);
    }

    fn poll_jobs(&mut self) {
        if let Some(manager) = self.job_manager.as_mut() {
            while let Some(res) = manager.try_get_result() {
                self.job_results.push(res);
            }
        }
        if self.job_manager.as_ref().is_some_and(|m| m.is_done()) {
            let jobs = self.job_manager.take().unwrap();
            jobs.finalize();
            self.finished_jobs();
        }
    }

    // TODO: realtime statistics would be cool to have
    fn finished_jobs(&mut self) {
        let mut results: Vec<_> = self
            .job_results
            .iter()
            .filter_map(|j| {
                if let JobResultType::Success { instructions } = j.ty {
                    Some(instructions)
                } else {
                    None
                }
            })
            .collect();
        if results.is_empty() {
            return;
        }
        results.sort();
        let min = *results.iter().min().unwrap();
        let max = *results.iter().max().unwrap();
        let avg =
            (results.iter().map(|u| *u as u128).sum::<u128>() / results.len() as u128) as usize;
        let med = if results.len() % 2 == 1 {
            results[(results.len() - 1) / 2] as f64
        } else {
            (results[(results.len() - 1) / 2] + results[(results.len() - 1) / 2 + 1]) as f64 / 2.
        };
        let deviation = {
            let avg = avg as i128;
            let sum = results.iter().map(|u| *u as i128 - avg).sum::<i128>();
            let sq = sum * sum;
            let avg = sq as f64 / results.len() as f64;
            avg.sqrt()
        };
        let metrics = Metrics {
            min,
            max,
            avg,
            med,
            deviation,
        };
        self.bench_result = Some(metrics);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.job_manager.is_some() {
            self.poll_jobs();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Benchmarking");

            ui.label(format!(
                "Executable path: {}",
                self.push_swap_path.display()
            ));

            ui.horizontal(|ui| {
                egui::DragValue::new(&mut self.numbers).ui(ui);
                ui.label(format!(
                    "random number{} to sort repeatedly",
                    if self.numbers == 1 { "" } else { "s" }
                ));
            });
            ui.horizontal(|ui| {
                egui::DragValue::new(&mut self.test_amount).ui(ui);
                ui.label(format!(
                    "sorting pass{} to run",
                    if self.test_amount == 1 { "" } else { "es" }
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Run on");
                egui::DragValue::new(&mut self.num_threads)
                    .range(MIN_THREADS..=u16::MAX as usize)
                    .ui(ui);
                ui.label(format!(
                    "thread{}",
                    if self.num_threads == 1 { "" } else { "s" }
                ));
            });
            if self.num_threads > THREADS_WARNING_THRESHOLD {
                ui.label(
                    egui::RichText::new(
                        "⚠️ Setting this to a high amount can freeze your computer!",
                    )
                    .color(egui::Color32::from_rgb(255, 192, 0)),
                );
            }
            match self.job_manager.as_ref() {
                Some(jobs) => {
                    if ui.button("Stop").clicked() {
                        jobs.cancel();
                    }
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Jobs running...");
                    });
                    let progress = jobs.jobs_progress();
                    let queued = jobs.jobs_queued();
                    let finished = jobs.jobs_finished();
                    egui::ProgressBar::new(progress)
                        .animate(true)
                        .text(format!("{}/{}", finished, queued))
                        .ui(ui);
                }
                None => {
                    if ui.button("Start benchmark").clicked() {
                        self.send_jobs();
                    }
                }
            }

            if let Some(metrics) = self.bench_result.as_ref() {
                let Metrics {
                    min,
                    max,
                    avg,
                    med,
                    deviation,
                } = metrics;
                ui.label(format!("Min: {}", min));
                ui.label(format!("Max: {}", max));
                ui.label(format!("Average: {}", avg));
                ui.label(format!("Median: {}", med));
                ui.label(format!("Deviation: {}", deviation));
            }

            for job in &self.job_results {
                let id = job.id;
                match &job.ty {
                    JobResultType::Success { .. } => {}
                    JobResultType::Failure { numbers } => {
                        ui.label(format!("Job {id} failed. Had numbers {numbers:?}"));
                    }
                    JobResultType::Error(e) => {
                        ui.label(format!("Job {id} ran into an error: {e}"));
                    }
                }
            }
        });
    }
}
