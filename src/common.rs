use eframe::egui;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortingStrategy {
    #[default]
    None,
    Simple,
    Medium,
    Complex,
    Adaptive,
}

impl SortingStrategy {
    const fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Simple => "Simple",
            Self::Medium => "Medium",
            Self::Complex => "Complex",
            Self::Adaptive => "Adaptive",
        }
    }

    pub const fn to_arg(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Simple => Some("--simple"),
            Self::Medium => Some("--medium"),
            Self::Complex => Some("--complex"),
            Self::Adaptive => Some("--adaptative"),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Sorting strategy")
            .selected_text(self.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::None, Self::None.name());
                ui.selectable_value(self, Self::Simple, Self::Simple.name());
                ui.selectable_value(self, Self::Medium, Self::Medium.name());
                ui.selectable_value(self, Self::Complex, Self::Complex.name());
                ui.selectable_value(self, Self::Adaptive, Self::Adaptive.name());
            });
    }
}
