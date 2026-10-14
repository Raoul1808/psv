use eframe::egui;

use crate::config::{ColorProfile, Config};

pub struct Visuals {
    need_redraw: bool,
}

impl Visuals {
    pub fn new() -> Self {
        Self { need_redraw: false }
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, config: &mut Config) {
        self.need_redraw = false;
        if ui.button("New Profile").clicked() {
            config.color_profiles.push(ColorProfile::new());
        }
        egui::ComboBox::from_label("Color Profile")
            .selected_text(&config.color_profiles[config.selected_color_profile].name)
            .show_index(
                ui,
                &mut config.selected_color_profile,
                config.color_profiles.len(),
                |i| &config.color_profiles[i].name,
            );
        let Some(profile) = config.try_get_current_profile_mut() else {
            return;
        };

        ui.add_enabled_ui(!profile.is_default(), |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut profile.name);
                ui.label("Profile name");
            });
            ui.horizontal(|ui| {
                let res = ui.color_edit_button_rgba_unmultiplied(&mut profile.clear_color);
                ui.label("Backing color");
                if res.changed() {
                    self.need_redraw = true;
                }
            });
            self.need_redraw |= profile.colors.ui(ui);
        });
        if ui.button("Save Config").clicked() {
            let _ = config.save();
        }
    }
}
