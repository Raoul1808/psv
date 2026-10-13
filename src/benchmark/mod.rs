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

pub struct Benchmark {
    push_swap_path: PathBuf,
    job_manager: Option<JobManager>,
    job_results: Vec<JobResult>,
    numbers: usize,
    test_amount: usize,
    num_threads: usize,
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
        }
    }

    pub fn update_path(&mut self, path: PathBuf) {
        self.push_swap_path = path;
    }

    pub fn is_benchmarking(&self) -> bool {
        self.job_manager.is_some()
    }

    pub fn send_jobs(&mut self) {
        self.job_results.clear();
        let jobs = JobManager::run(
            self.push_swap_path.clone(),
            self.numbers,
            self.test_amount,
            self.num_threads,
        );
        self.job_manager = Some(jobs);
    }

    pub fn poll_jobs(&mut self) {
        if let Some(manager) = self.job_manager.as_mut() {
            while let Some(res) = manager.try_get_result() {
                self.job_results.push(res);
            }
        }
        if self.job_manager.as_ref().is_some_and(|m| m.is_done()) {
            let jobs = self.job_manager.take().unwrap();
            jobs.finalize();
        }
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

            for job in &self.job_results {
                let id = job.id;
                match &job.ty {
                    JobResultType::Success { instructions } => {
                        ui.label(format!("Job {id} finished successfully: {instructions}"));
                    }
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
