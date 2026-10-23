use std::fmt::Write;

use eframe::egui;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Legend, Span, VLine};

use crate::benchmark::{JobResult, JobResultType};

#[derive(Debug, Default, Clone, Copy)]
struct Metrics {
    min: usize,
    max: usize,
    avg: usize,
    quart1: f64,
    med: f64,
    quart3: f64,
    deviation: f64,
}

pub struct Plot {
    metrics: Metrics,
    results: Vec<usize>,
    errors: Vec<JobResult>,
    dirty: bool,
}

impl Plot {
    pub fn new() -> Self {
        Self {
            metrics: Metrics::default(),
            results: vec![],
            errors: vec![],
            dirty: false,
        }
    }

    pub fn clear(&mut self) {
        self.metrics = Metrics::default();
        self.results.clear();
        self.errors.clear();
        self.dirty = false;
    }

    fn update_metrics(&mut self) {
        self.results.sort();
        let results = &self.results;
        if results.is_empty() {
            return;
        }
        let min = *results.iter().min().unwrap();
        let max = *results.iter().max().unwrap();
        let avg =
            (results.iter().map(|u| *u as u128).sum::<u128>() / results.len() as u128) as usize;
        let med = if results.len() % 2 == 1 {
            results[(results.len() - 1) / 2] as f64
        } else {
            (results[(results.len() - 1) / 2] + results[(results.len() - 1) / 2 + 1]) as f64 / 2.
        };
        let quart1 = {
            let i = (results.len() as f64 + 3.) / 4.;
            let low = i as usize;
            let high = i as usize + 1;
            if high >= results.len() || low >= results.len() {
                f64::NAN
            } else {
                let dec = (i.fract() * 4.).round() as usize;
                match dec {
                    0 => results[low] as f64,
                    1 => (3. * results[low] as f64 + results[high] as f64) / 4.,
                    2 => (results[low] as f64 + results[high] as f64) / 2.,
                    3 => (results[low] as f64 + 3. * results[high] as f64) / 4.,
                    _ => unreachable!(),
                }
            }
        };
        let quart3 = {
            let i = (3. * results.len() as f64 + 1.) / 4.;
            let low = i as usize;
            let high = i as usize + 1;
            if high >= results.len() || low >= results.len() {
                f64::NAN
            } else {
                let dec = (i.fract() * 4.).round() as usize;
                match dec {
                    0 => results[low] as f64,
                    1 => (3. * results[low] as f64 + results[high] as f64) / 4.,
                    2 => (results[low] as f64 + results[high] as f64) / 2.,
                    3 => (results[low] as f64 + 3. * results[high] as f64) / 4.,
                    _ => unreachable!(),
                }
            }
        };
        let deviation = {
            let avg = avg as i128;
            let sum = results
                .iter()
                .map(|u| (*u as i128 - avg).pow(2))
                .sum::<i128>();
            let avg = sum as f64 / results.len() as f64;
            avg.sqrt()
        };
        let metrics = Metrics {
            min,
            max,
            avg,
            quart1,
            med,
            quart3,
            deviation,
        };
        self.metrics = metrics;
    }

    pub fn add_result(&mut self, result: JobResult) {
        match &result.ty {
            JobResultType::Success { instructions } => {
                self.results.push(*instructions);
            }
            _ => {
                self.errors.push(result);
            }
        }
        self.dirty = true;
    }

    fn draw_plot(&mut self, ui: &mut egui::Ui) {
        let plot = BoxPlot::new(
            "Instructions",
            vec![BoxElem::new(
                0.0,
                BoxSpread::new(
                    self.metrics.min as f64,
                    self.metrics.quart1,
                    self.metrics.med,
                    self.metrics.quart3,
                    self.metrics.max as f64,
                ),
            )],
        )
        .horizontal();
        let dev_start = self.metrics.avg as f64 - self.metrics.deviation / 2.;
        let dev_end = self.metrics.avg as f64 + self.metrics.deviation / 2.;
        let dev_span = Span::new("Deviation", dev_start..=dev_end);

        let avg_line = VLine::new("Average", self.metrics.avg as f64);

        let range = self.metrics.max - self.metrics.min;
        let min_x = self.metrics.min as f64 - range as f64 * 0.2;
        let max_x = self.metrics.max as f64 + range as f64 * 0.2;
        egui_plot::Plot::new("Instructions Distribution")
            .legend(Legend::default())
            .allow_zoom(egui::Vec2b::new(true, false))
            .allow_drag(egui::Vec2b::new(true, false))
            .allow_scroll(egui::Vec2b::new(true, false))
            .auto_bounds(egui::Vec2b::new(true, false))
            .show_axes(egui::Vec2b::new(true, false))
            .center_y_axis(true)
            .include_x(min_x)
            .include_x(max_x)
            .default_y_bounds(-0.5, 1.5)
            .show(ui, |plot_ui| {
                plot_ui.box_plot(plot);
                plot_ui.span(dev_span);
                plot_ui.vline(avg_line);
            });
    }

    fn show_failed_jobs(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(" ");
                });
                header.col(|ui| {
                    ui.strong("Job ID");
                });
                header.col(|ui| {
                    ui.strong("Context");
                });
            })
            .body(|ui| {
                ui.rows(text_height, self.errors.len(), |mut row| {
                    let row_index = row.index();
                    let res = &self.errors[row_index];
                    row.col(|ui| {
                        let label = match &res.ty {
                            JobResultType::Failure { .. } => "⚠️",
                            JobResultType::Error(_) => "❌",
                            JobResultType::Success { .. } => "",
                        };
                        ui.label(label);
                    });
                    row.col(|ui| {
                        ui.label(res.id.to_string());
                    });
                    row.col(|ui| match &res.ty {
                        JobResultType::Failure { numbers } => {
                            ui.label("did not sort numbers");
                            if ui.button("Copy numbers").clicked() {
                                let nums = numbers.iter().fold(String::new(), |mut acc, n| {
                                    let _ = write!(acc, "{n} ");
                                    acc
                                });
                                let nums = nums.trim_end();
                                ui.copy_text(nums.into());
                            }
                        }
                        JobResultType::Error(e) => {
                            ui.label(format!("ran into an error: {e}"));
                        }
                        JobResultType::Success { .. } => {}
                    });
                });
            });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.results.is_empty() {
            return;
        }
        if self.dirty {
            self.update_metrics();
            self.dirty = false;
        }
        ui.columns(2, |ui_cols| {
            let ui = &mut ui_cols[0];
            ui.label(format!("Average: {}", self.metrics.avg));
            ui.separator();
            ui.label(format!("Max = {}", self.metrics.max));
            ui.label(format!("Quartile 3 = {}", self.metrics.quart3));
            ui.label(format!("Median = {}", self.metrics.med));
            ui.label(format!("Quartile 1 = {}", self.metrics.quart1));
            ui.label(format!("Min = {}", self.metrics.min));
            ui.separator();
            ui.label(format!("Deviation: {}", self.metrics.deviation));

            if !self.errors.is_empty() {
                ui.separator();
                let success_rate = 100. * self.results.len() as f64
                    / (self.results.iter().len() + self.errors.len()) as f64;
                ui.label(format!("Program success rate: {success_rate}%"));
                ui.collapsing("Errors", |ui| {
                    self.show_failed_jobs(ui);
                });
            }

            let ui = &mut ui_cols[1];
            self.draw_plot(ui);
        });
    }
}
