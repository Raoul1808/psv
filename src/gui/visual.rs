use egui::{
    pos2, Button, ComboBox, Context, Mesh, Rect, Rgba, Sense, Shape, Slider, Ui, Widget, Window,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    config::{ColorProfile, Config, SortColors},
    util,
};

pub struct VisualOptions {
    color_profile: ColorProfile,
    opacity: u8,
    preview_nums: Vec<u8>,
}

impl VisualOptions {
    pub fn new(config: &Config) -> Self {
        let color_profile = config
            .color_profiles
            .get(config.current_profile)
            .cloned()
            .unwrap_or(util::default_profile());
        let preview_nums: Vec<_> = (0..100).collect();
        Self {
            color_profile,
            opacity: config.egui_opacity,
            preview_nums,
        }
    }

    pub fn clear_color(&self) -> [f32; 3] {
        self.color_profile.clear_color
    }

    pub fn opacity(&self) -> u8 {
        self.opacity
    }

    fn default_subdivisions() -> Vec<[f32; 3]> {
        vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    pub fn color_at(&self, t: f32) -> [f32; 4] {
        match &self.color_profile.colors {
            SortColors::FromGradient(g) => g.color_at(t),
            SortColors::ColoredSubdisions(v) => {
                let len = v.len();
                for (i, col) in v.iter().enumerate().rev() {
                    let t2 = i as f32 / len as f32;
                    if t >= t2 {
                        return [col[0], col[1], col[2], 1.0];
                    }
                }
                [1.0; 4]
            }
        }
    }

    fn color_profile_gui(&mut self, ui: &mut Ui) {
        ui.label("Color Profile");
        if ui
            .text_edit_singleline(&mut self.color_profile.name)
            .changed()
        {
            self.color_profile.name.truncate(ColorProfile::NAME_MAX_LEN);
        }
        ui.horizontal(|ui| {
            ui.color_edit_button_rgb(&mut self.color_profile.clear_color);
            ui.label("Background Color");
        });
        ComboBox::from_label("Color Setting")
                .selected_text(self.color_profile.colors.to_string())
                .show_ui(ui, |ui| {
                    let (gradient, subdivisions) = match &self.color_profile.colors {
                        SortColors::FromGradient(g) => (g.clone(), Self::default_subdivisions()),
                        SortColors::ColoredSubdisions(s) => (util::default_gradient(), s.clone()),
                    };
                    ui.selectable_value(
                        &mut self.color_profile.colors,
                        SortColors::FromGradient(gradient),
                        "From Gradient",
                    ).on_hover_text("Sorted numbers' color will be determined by a color gradient.");
                    ui.selectable_value(
                        &mut self.color_profile.colors,
                        SortColors::ColoredSubdisions(subdivisions),
                        "Colored Subdivisions",
                    ).on_hover_ui(|ui| {
                        ui.label("Sorted numbers will have their color determined in groups.");
                        ui.label("e.g: if you have defined 3 colors for 300 numbers, the first 100 numbers will be colored with the first color, the second 100 with the second color, and the third 100 with the third color.");
                        ui.label("Useful to visualize sorting algorithms that group together numbers in big range groups.");
                    });
                });
        match &mut self.color_profile.colors {
            SortColors::FromGradient(g) => g.ui(ui),
            SortColors::ColoredSubdisions(s) => {
                let mut del = None;
                let enabled = s.len() > 1;
                for (i, col) in s.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Color {}", i + 1));
                        ui.color_edit_button_rgb(col);
                        if ui.add_enabled(enabled, Button::new("Remove")).clicked() {
                            del = Some(i);
                        }
                    });
                }
                if ui.button("Add color").clicked() {
                    s.push([1.0, 1.0, 1.0]);
                }
                if let Some(i) = del {
                    s.remove(i);
                }
            }
        };
    }

    fn preview_color(&mut self, ui: &mut Ui) {
        ui.label("Color preview");
        ui.horizontal(|ui| {
            if ui.button("Reorder").clicked() {
                self.preview_nums.sort();
            }
            if ui.button("Shuffle").clicked() {
                self.preview_nums.shuffle(&mut thread_rng());
            }
        });
        ui.scope(|ui| {
            ui.spacing_mut().slider_width = ui.available_width();
            let mut len = self.preview_nums.len() as u8;
            let slider = Slider::new(&mut len, 10..=u8::MAX).show_value(false).ui(ui);
            if slider.changed() {
                self.preview_nums = (0..len).collect();
            }
        });
        let size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(size, Sense::hover());
        let mut mesh = Mesh::default();
        {
            let [r, g, b] = self.color_profile.clear_color;
            mesh.add_colored_rect(rect, Rgba::from_rgba_unmultiplied(r, g, b, 1.).into());
        }
        let len = self.preview_nums.len() as f32;
        for (i, n) in self.preview_nums.iter().enumerate() {
            let i = i as f32;
            let n = *n as f32;
            let left = rect.left();
            let right = rect.left() + (rect.width() / (len / (n + 1.)));
            let top = rect.top() + i * (rect.height() / len);
            let bottom = top + (rect.height() / len);
            let [r, g, b, a] = self.color_at(n / (len - 1.));
            let rect = Rect {
                min: pos2(left, top),
                max: pos2(right, bottom),
            };
            mesh.add_colored_rect(rect, Rgba::from_rgba_unmultiplied(r, g, b, a).into());
        }
        ui.painter().add(Shape::mesh(mesh));
    }

    pub fn ui(&mut self, ctx: &Context, config: &mut Config, open: &mut bool) {
        Window::new("Visual Options").open(open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Floating window opacity");
                let slider = Slider::new(&mut self.opacity, 128..=255)
                    .show_value(false)
                    .ui(ui);
                if slider.drag_stopped() {
                    config.egui_opacity = self.opacity();
                    config.save();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Scale factor");
                if ui.button("-").clicked() {
                    config.scale_factor = (config.scale_factor - 0.1).max(0.3);
                    config.save();
                }
                ui.label(format!("{:.1}", config.scale_factor));
                if ui.button("+").clicked() {
                    config.scale_factor = (config.scale_factor + 0.1).min(3.0);
                    config.save();
                }
            });
            ui.separator();
            let can_switch_or_delete = config.color_profiles.len() > 1;
            ui.add_enabled_ui(can_switch_or_delete, |ui| {
                let profile = ComboBox::from_label("Color Profile").show_index(
                    ui,
                    &mut config.current_profile,
                    config.color_profiles.len(),
                    |index| &config.color_profiles[index].name,
                );
                if profile.changed() {
                    config.save();
                    self.color_profile = config.color_profiles[config.current_profile].clone();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    config.color_profiles.push(util::default_profile());
                    config.current_profile = config.color_profiles.len() - 1;
                    self.color_profile = config.color_profiles[config.current_profile].clone();
                }
                if ui.button("Save").clicked() {
                    config.color_profiles[config.current_profile] = self.color_profile.clone();
                    config.save();
                }
                if ui
                    .add_enabled(can_switch_or_delete, Button::new("Delete"))
                    .clicked()
                {
                    config.color_profiles.remove(config.current_profile);
                    config.current_profile =
                        config.current_profile.clamp(0, config.color_profiles.len());
                    self.color_profile = config.color_profiles[config.current_profile].clone();
                    config.save();
                }
            });
            ui.separator();
            ui.columns_const(|[ui_left, ui_right]| {
                self.preview_color(ui_left);
                self.color_profile_gui(ui_right);
            });
        });
    }
}
