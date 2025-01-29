use egui::{
    lerp, pos2, vec2, Button, ComboBox, Context, Mesh, Rect, Rgba, Sense, Shape, Slider, Ui,
    Widget, Window,
};

use crate::{
    config::{Config, SortColors},
    util,
};

pub struct VisualOptions {
    clear_color: [f32; 3],
    sort_colors: SortColors,
    opacity: u8,
}

impl VisualOptions {
    pub fn new(config: &Config) -> Self {
        Self {
            clear_color: config.clear_color,
            sort_colors: config.sort_colors.clone(),
            opacity: config.egui_opacity,
        }
    }

    pub fn clear_color(&self) -> [f32; 3] {
        self.clear_color
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
        match &self.sort_colors {
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

    fn preview_color(&self, ui: &mut Ui) {
        let width = ui.available_width();
        let height = ui.spacing().slider_rail_height + ui.spacing().interact_size.y;
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let mut mesh = Mesh::default();
            match &self.sort_colors {
                SortColors::FromGradient(g) => {
                    let steps = g.steps_sorted();
                    let n = steps.len();
                    if let Some((t, color)) = steps.first() {
                        let x = lerp(rect.x_range(), *t);
                        if *t > 0.0 {
                            let rect = Rect {
                                min: rect.left_top(),
                                max: pos2(x, rect.bottom()),
                            };
                            let [r, g, b, a] = *color;
                            let color = Rgba::from_rgba_unmultiplied(r, g, b, a);
                            mesh.add_colored_rect(rect, color.into());
                        }
                    }
                    for (i, (t, color)) in steps.iter().copied().enumerate() {
                        let [r, g, b, a] = color;
                        let color = Rgba::from_rgba_unmultiplied(r, g, b, a);
                        let x = lerp(rect.x_range(), t);
                        mesh.colored_vertex(pos2(x, rect.top()), color.into());
                        mesh.colored_vertex(pos2(x, rect.bottom()), color.into());
                        if i < n - 1 {
                            let i = i as u32;
                            mesh.add_triangle(2 * i, 2 * i + 1, 2 * i + 2);
                            mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                        }
                    }
                    if let Some((t, color)) = steps.last() {
                        let x = lerp(rect.x_range(), *t);
                        if *t > 0.0 {
                            let rect = Rect {
                                min: pos2(x, rect.top()),
                                max: rect.right_bottom(),
                            };
                            let [r, g, b, a] = *color;
                            let color = Rgba::from_rgba_unmultiplied(r, g, b, a);
                            mesh.add_colored_rect(rect, color.into());
                        }
                    }
                }
                SortColors::ColoredSubdisions(s) => {
                    let n = s.len() as f32;
                    for (i, color) in s.iter().copied().enumerate() {
                        let [r, g, b] = color;
                        let color = Rgba::from_rgba_unmultiplied(r, g, b, 1.0);
                        let tmin = i as f32 / n;
                        let tmax = (i + 1) as f32;
                        let x1 = lerp(rect.x_range(), tmin);
                        let x2 = lerp(rect.x_range(), tmax);
                        let rect = Rect {
                            min: pos2(x1, rect.top()),
                            max: pos2(x2, rect.bottom()),
                        };
                        mesh.add_colored_rect(rect, color.into());
                    }
                }
            };
            ui.painter().add(Shape::mesh(mesh));
        }
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
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut self.clear_color);
                ui.label("Background Color");
                if ui.button("Save").clicked() {
                    config.clear_color = self.clear_color();
                    config.save();
                }
            });
            ui.label("Color preview");
            self.preview_color(ui);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    self.sort_colors = util::default_gradient().into();
                }
                if ui.button("Load").clicked() {
                    self.sort_colors = config.sort_colors.clone();
                }
                if ui.button("Save").clicked() {
                    config.sort_colors = self.sort_colors.clone();
                    config.save();
                }
            });
            ui.separator();
            ComboBox::from_label("Color Setting")
                .selected_text(self.sort_colors.to_string())
                .show_ui(ui, |ui| {
                    let (gradient, subdivisions) = match &self.sort_colors {
                        SortColors::FromGradient(g) => (g.clone(), Self::default_subdivisions()),
                        SortColors::ColoredSubdisions(s) => (util::default_gradient(), s.clone()),
                    };
                    ui.selectable_value(
                        &mut self.sort_colors,
                        SortColors::FromGradient(gradient),
                        "From Gradient",
                    ).on_hover_text("Sorted numbers' color will be determined by a color gradient.");
                    ui.selectable_value(
                        &mut self.sort_colors,
                        SortColors::ColoredSubdisions(subdivisions),
                        "Colored Subdivisions",
                    ).on_hover_ui(|ui| {
                        ui.label("Sorted numbers will have their color determined in groups.");
                        ui.label("e.g: if you have defined 3 colors for 300 numbers, the first 100 numbers will be colored with the first color, the second 100 with the second color, and the third 100 with the third color.");
                        ui.label("Useful to visualize sorting algorithms that group together numbers in big range groups.");
                    });
                });
            match &mut self.sort_colors {
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
        });
    }
}
