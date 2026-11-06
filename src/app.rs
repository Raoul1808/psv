use eframe::egui::{self, Widget};
use egui_alignments::center_horizontal;

use crate::{
    benchmark::Benchmark,
    config::{AppMode, Config, VisualizerSideViewMode},
    help::Help,
    visualizer::Visualizer,
};

pub struct App {
    app_mode: AppMode,
    visualizer: Visualizer,
    benchmark: Benchmark,
    help: Help,
    config: Config,
    show_about: bool,
    show_settings: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext, config: Config) -> Self {
        let mut visualizer = Visualizer::new(cc, &config);
        let mut benchmark = Benchmark::new();
        if let Some(path) = config.push_swap_path.clone() {
            visualizer.update_path(path.clone());
            benchmark.update_path(path.clone());
        }
        let app_mode = config.preferred_app_mode;
        Self {
            config,
            app_mode,
            visualizer,
            benchmark,
            help: Help::new(),
            show_about: false,
            show_settings: false,
        }
    }

    fn select_executable(&mut self) {
        let file = rfd::FileDialog::new().pick_file();
        if let Some(file) = file {
            self.visualizer.update_path(file.clone());
            self.benchmark.update_path(file.clone());
            self.config.push_swap_path = Some(file);
            self.config.save_with_log();
        }
    }

    fn first_time_ui(&mut self, ui: &mut egui::Ui) {
        egui::Modal::new(egui::Id::unique("missing_path")).show(ui, |ui| {
            ui.horizontal(|ui| {
                center_horizontal(ui, |ui| ui.heading("Setup"));
            });
            ui.separator();
            ui.label("This program requires a push_swap executable to run!");
            ui.label("Hint: you can change this later with File -> Select push_swap executable");
            ui.add_space(7.);
            center_horizontal(ui, |ui| {
                if egui::Button::new("Select push_swap executable")
                    .min_size([ui.available_width(), 0.0].into())
                    .ui(ui)
                    .clicked()
                {
                    self.select_executable();
                }
            });
        });
    }

    fn show_about(&mut self, ui: &mut egui::Ui) {
        let res = egui::Modal::new(egui::Id::unique("about")).show(ui, |ui| {
            ui.horizontal(|ui| {
                center_horizontal(ui, |ui| {
                    ui.heading("psv");
                })
            });
            ui.separator();
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Yet another push_swap visualizer");
            ui.label("Made in Rust using eframe/egui");

            ui.add_space(10.);

            let width = ui.available_width();
            ui.horizontal(|ui| {
                if egui::Button::new("Close")
                    .min_size([width, 0.0].into())
                    .ui(ui)
                    .clicked()
                {
                    self.show_about = false;
                }
            });
        });
        if res.should_close() {
            self.show_about = false;
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        let res = egui::Modal::new(egui::Id::unique("settings")).show(ui, |ui| {
            ui.horizontal(|ui| {
                center_horizontal(ui, |ui| {
                    ui.heading("Settings");
                })
            });
            ui.separator();
            egui::ComboBox::from_label("Preferred app mode on startup")
                .selected_text(self.config.preferred_app_mode.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.config.preferred_app_mode,
                        AppMode::Visualize,
                        AppMode::Visualize.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.config.preferred_app_mode,
                        AppMode::Benchmark,
                        AppMode::Benchmark.to_string(),
                    );
                });

            egui::ComboBox::from_label("Preferred visualizer side view on startup")
                .selected_text(self.config.preferred_side_panel.to_string())
                .show_ui(ui, |ui| {
                    for mode in VisualizerSideViewMode::ALL {
                        ui.selectable_value(
                            &mut self.config.preferred_side_panel,
                            mode,
                            mode.to_string(),
                        );
                    }
                });

            ui.add_space(10.);

            ui.horizontal(|ui| {
                let width = ui.available_width();
                if egui::Button::new("Close")
                    .min_size([width, 0.0].into())
                    .ui(ui)
                    .clicked()
                {
                    self.show_settings = false;
                    self.config.save_with_log();
                }
            })
        });
        if res.should_close() {
            self.show_settings = false;
            self.config.save_with_log();
        }
    }

    fn app_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Select push_swap executable").clicked() {
                        self.select_executable();
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                match self.app_mode {
                    AppMode::Visualize => {
                        self.visualizer.menu_ui(ui);
                    }
                    AppMode::Benchmark => {
                        self.benchmark.menu_ui(ui);
                    }
                }
                ui.menu_button("Help", |ui| {
                    ui.add_enabled(
                        false,
                        egui::Label::new(format!("psv v{}", env!("CARGO_PKG_VERSION"))),
                    );
                    ui.separator();
                    if ui.button("Help Manual").clicked() {
                        self.help.open();
                    }
                    ui.separator();
                    if ui.button("About psv").clicked() {
                        self.show_about = true;
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_change_app_mode = match self.app_mode {
                        AppMode::Visualize => !self.visualizer.should_lock_app_mode(),
                        AppMode::Benchmark => !self.benchmark.is_benchmarking(),
                    };
                    let res = ui.add_enabled_ui(can_change_app_mode, |ui| {
                        egui::ComboBox::new("app_mode", "")
                            .selected_text(self.app_mode.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.app_mode,
                                    AppMode::Visualize,
                                    AppMode::Visualize.to_string(),
                                );
                                ui.selectable_value(
                                    &mut self.app_mode,
                                    AppMode::Benchmark,
                                    AppMode::Benchmark.to_string(),
                                );
                            });
                        ui.label("App Mode:");
                    });
                    egui::Tooltip::for_disabled(&res.response)
                        .at_pointer()
                        .show(|ui| {
                            ui.label(
                                "Cannot change app mode while push_swap processes are running",
                            );
                        });
                });
            });
        });

        match self.app_mode {
            AppMode::Visualize => {
                self.visualizer.ui(ui, &mut self.config);
            }
            AppMode::Benchmark => {
                self.benchmark.ui(ui);
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        let path_set = self.config.push_swap_path.is_some();
        if path_set {
            self.app_ui(ui);
            if self.show_about {
                self.show_about(ui);
            }
            if self.show_settings {
                self.show_settings(ui);
            }
            if self.help.is_open() {
                self.help.ui(ui, frame);
            }
        } else {
            self.first_time_ui(ui);
        }
    }
}
