use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HelpSection {
    Start,
    Visualizer,
    Benchmarking,
    ReadingBenchmarks,
}

impl HelpSection {
    const ALL: &[Self] = &[
        Self::Start,
        Self::Visualizer,
        Self::Benchmarking,
        Self::ReadingBenchmarks,
    ];

    pub fn section_name(&self) -> &str {
        match self {
            Self::Start => "Start",
            Self::Visualizer => "Visualizer",
            Self::Benchmarking => "Benchmarking",
            Self::ReadingBenchmarks => "Reading benchmark results",
        }
    }

    pub fn section_ui(&self, ui: &mut egui::Ui) {
        match self {
            Self::Start => {
                ui.label("Welcome to the help section!");
                ui.label("Select a section on the left to get started.");
            }
            Self::Visualizer => {}
            Self::Benchmarking => {}
            Self::ReadingBenchmarks => {}
        }
    }
}

pub struct Help {
    current_section: usize,
    is_open: bool,
}

impl Help {
    pub fn new() -> Self {
        Self {
            current_section: 0,
            is_open: false,
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

        egui::Window::new("Help")
            .open(&mut self.is_open)
            .order(egui::Order::Foreground)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .title_frame(
                egui::Frame::window(ui.style()).fill(ui.visuals().widgets.open.weak_bg_fill),
            )
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .fixed_size((
                size.width as f32 - margin * 2.,
                size.height as f32 - margin * 2.,
            ))
            .fixed_pos(egui::pos2(size.width as f32 / 2., size.height as f32 / 2.))
            .show(ui, |ui| {
                egui::Panel::left("help_section").show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, s) in HelpSection::ALL.iter().enumerate() {
                            ui.selectable_value(&mut self.current_section, i, s.section_name());
                        }
                    });
                });
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        HelpSection::ALL[self.current_section].section_ui(ui);
                    });
                });
            })
            .unwrap();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, eframe: &mut eframe::Frame) {
        // I stole this from egui's Modal::show
        let area = egui::Area::new("help_backdrop".into())
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
