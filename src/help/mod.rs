use eframe::egui;

mod app_modes;
mod benchmarking;
mod benchmarking_stats;
mod details_view;
mod playback_controls;
mod visualizer;
mod visuals_view;

trait HelpSection {
    fn section_name(&self) -> &str;
    fn section_ui(&mut self, ui: &mut egui::Ui);
}

struct StartSection;

impl HelpSection for StartSection {
    fn section_name(&self) -> &str {
        "Start"
    }

    fn section_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Welcome to the help section!");
        ui.label("Select a section on the left to get started.");
    }
}

pub struct Help {
    current_section: usize,
    is_open: bool,
    sections: Vec<Box<dyn HelpSection>>,
}

impl Help {
    pub fn new() -> Self {
        let sections: Vec<Box<dyn HelpSection>> = vec![
            Box::new(StartSection),
            Box::new(app_modes::AppModesSection),
            Box::new(visualizer::VisualizerSection),
            Box::new(playback_controls::PlaybackControlsSection::new()),
            Box::new(details_view::DetailsViewSection),
            Box::new(visuals_view::VisualsViewSection),
            Box::new(benchmarking::BenchmarkingSection),
            Box::new(benchmarking_stats::BenchmarkingStatsSection),
        ];
        Self {
            current_section: 0,
            is_open: false,
            sections,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn window_ui(&mut self, ui: &mut egui::Ui, eframe: &mut eframe::Frame) {
        let size = eframe.winit_window().expect("no window???").inner_size();
        let margin = 60.;

        let size = egui::vec2(
            size.width as f32 - margin * 2.,
            size.height as f32 - margin * 2.,
        );

        egui::Window::new("Help Manual")
            .open(&mut self.is_open)
            .order(egui::Order::Foreground)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .title_frame(
                egui::Frame::window(ui.style()).fill(ui.visuals().widgets.open.weak_bg_fill),
            )
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .min_size(size)
            .max_size(size)
            .fixed_pos(egui::pos2(size.x / 2., size.y / 2.))
            .show(ui, |ui| {
                egui::Panel::left("help_section").show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, s) in self.sections.iter().enumerate() {
                            ui.selectable_value(&mut self.current_section, i, s.section_name());
                        }
                    });
                });
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Min).with_main_wrap(true),
                            |ui| {
                                self.sections[self.current_section].section_ui(ui);
                            },
                        );
                    });
                });
            })
            .unwrap();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, eframe: &mut eframe::Frame) {
        // I stole this from egui's Modal::show
        let area = egui::Area::new(egui::Id::unique("help_backdrop"))
            .kind(egui::UiKind::Modal)
            .sense(egui::Sense::hover())
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .interactable(true);
        let backdrop_color = egui::Color32::from_black_alpha(100);

        let egui::InnerResponse {
            inner: backdrop_response,
            ..
        } = area.show(ui, |ui| {
            let bg_rect = ui.content_rect();
            let bg_sense = egui::Sense::CLICK | egui::Sense::DRAG;
            let mut backdrop =
                ui.new_child(egui::UiBuilder::new().sense(bg_sense).max_rect(bg_rect));
            backdrop.set_min_size(bg_rect.size());
            ui.painter().rect_filled(bg_rect, 0.0, backdrop_color);
            let backdrop_response = backdrop.response();

            ui.scope_builder(
                egui::UiBuilder::new().sense(egui::Sense::CLICK | egui::Sense::DRAG),
                |ui| self.window_ui(ui, eframe),
            );

            backdrop_response
        });

        if backdrop_response.clicked() {
            self.is_open = false;
        }
    }
}
