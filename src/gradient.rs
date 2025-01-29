// NOTE: File copied from small side-project

use egui::Widget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A structure that represents a color gradient.
pub struct Gradient {
    steps: Vec<(f32, [f32; 4])>,
}

fn lerp(col1: [f32; 4], col2: [f32; 4], t: f32) -> [f32; 4] {
    let mut col = [0.0; 4];
    for i in 0..4 {
        col[i] = col1[i] + t * (col2[i] - col1[i]);
    }
    col
}

#[allow(dead_code)]
impl Gradient {
    /// Creates a new gradient from the given colors.
    pub fn new(col_a: [f32; 4], col_b: [f32; 4]) -> Gradient {
        Gradient {
            steps: vec![(0.0, col_a), (1.0, col_b)],
        }
    }

    /// Creates a new gradient from the given colors slice.
    ///
    /// # Panics
    ///
    /// This function panics if the given slice has less than 2 elements.
    pub fn from_slice(arr: &[[f32; 4]]) -> Gradient {
        assert!(
            arr.len() >= 2,
            "There must be at least 2 colors for a gradient"
        );
        let mut steps = vec![];
        for i in 0..arr.len() {
            let progress = i as f32 / (arr.len() - 1) as f32;
            steps.push((progress, arr[i]));
        }
        Gradient { steps }
    }

    pub fn steps_sorted(&self) -> Vec<(f32, [f32; 4])> {
        let mut gradient = self.steps.clone();
        if gradient.is_sorted_by(|(t1, _), (t2, _)| t1 <= t2) {
            gradient.sort_by(|(t1, _), (t2, _)| t1.total_cmp(t2));
        }
        gradient
    }

    pub fn color_at(&self, t: f32) -> [f32; 4] {
        let gradient = self.steps_sorted();
        let t = t.clamp(0.0, 1.0);
        for i in 0..(gradient.len() - 1) {
            let (t1, col1) = gradient[i];
            let (t2, col2) = gradient[i + 1];
            if t <= t1 {
                return col1;
            }
            if t == t2 {
                return col2;
            }
            if t > t2 {
                continue;
            }
            let t = (t - t1) / (t2 - t1);
            return lerp(col1, col2, t);
        }
        gradient.last().unwrap().1
    }

    // NOTE: This is new
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut dragging = false;
        ui.horizontal(|ui| {
            let enabled = self.steps.len() > 2;
            if ui.button("+").clicked() {
                self.steps.push((1.0, [0.0, 0.0, 0.0, 1.0]));
            }
            if ui.add_enabled(enabled, egui::Button::new("-")).clicked() {
                let _ = self.steps.pop();
            }
        });
        for (t, col) in self.steps.iter_mut() {
            ui.horizontal(|ui| {
                ui.color_edit_button_rgba_unmultiplied(col);
                dragging |= egui::Slider::new(t, 0.0..=1.0).ui(ui).dragged();
            });
        }
        if !dragging {
            self.steps.sort_by(|(t1, _), (t2, _)| t1.total_cmp(t2));
        }
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            steps: vec![(0.0, [0.0, 0.0, 0.0, 1.0]), (1.0, [1.0, 1.0, 1.0, 1.0])],
        }
    }
}

#[cfg(test)]
mod test {
    use super::Gradient;

    #[test]
    fn new() {
        let col_a = [1.0, 1.0, 0.0, 1.0];
        let col_b = [1.0, 0.0, 1.0, 1.0];
        let expected = Gradient {
            steps: vec![(0.0, [1.0, 1.0, 0.0, 1.0]), (1.0, [1.0, 0.0, 1.0, 1.0])],
        };
        let colors = Gradient::new(col_a, col_b);
        assert_eq!(colors, expected);
    }

    #[test]
    fn from_slice_2() {
        let colors = [[1.0, 1.0, 0.0, 1.0], [1.0, 0.0, 1.0, 1.0]];
        let expected = Gradient {
            steps: vec![(0.0, [1.0, 1.0, 0.0, 1.0]), (1.0, [1.0, 0.0, 1.0, 1.0])],
        };
        let colors = Gradient::from_slice(colors.as_slice());
        assert_eq!(colors, expected);
    }

    #[test]
    fn from_slice_3() {
        let colors = [
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
        ];
        let expected = Gradient {
            steps: vec![
                (0.0, [1.0, 0.0, 0.0, 1.0]),
                (0.5, [1.0, 0.0, 1.0, 1.0]),
                (1.0, [1.0, 1.0, 1.0, 1.0]),
            ],
        };
        let colors = Gradient::from_slice(colors.as_slice());
        assert_eq!(colors, expected);
    }

    #[test]
    fn from_slice_5() {
        let colors = [
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
        ];
        let expected = Gradient {
            steps: vec![
                (0.0, [1.0, 0.0, 0.0, 1.0]),
                (0.25, [1.0, 0.0, 1.0, 1.0]),
                (0.5, [1.0, 1.0, 0.0, 1.0]),
                (0.75, [0.0, 1.0, 1.0, 1.0]),
                (1.0, [1.0, 1.0, 1.0, 1.0]),
            ],
        };
        let colors = Gradient::from_slice(colors.as_slice());
        assert_eq!(colors, expected);
    }

    #[test]
    #[should_panic]
    fn from_slice_invalid_size() {
        let colors = [[1.0, 1.0, 1.0, 1.0]];
        let _ = Gradient::from_slice(colors.as_slice());
        // This should have panicked
    }

    #[test]
    fn color_at_simple() {
        let colors = Gradient::new([1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]);

        let expected = [0.5, 0.5, 0.0, 1.0];
        let color = colors.color_at(0.5);
        assert_eq!(color, expected);

        let expected = [0.25, 0.75, 0.0, 1.0];
        let color = colors.color_at(0.75);
        assert_eq!(color, expected);

        let expected = [0.875, 0.125, 0.0, 1.0];
        let color = colors.color_at(0.125);
        assert_eq!(color, expected);

        let expected = [1.0, 0.0, 0.0, 1.0];
        let color = colors.color_at(-4.2);
        assert_eq!(color, expected);

        let expected = [0.0, 1.0, 0.0, 1.0];
        let color = colors.color_at(1.0);
        assert_eq!(color, expected);
    }

    #[test]
    fn color_at_narrow() {
        let colors = Gradient {
            steps: vec![(0.3, [1.0, 0.0, 0.0, 1.0]), (0.6, [0.0, 1.0, 0.5, 1.0])],
        };

        // float inaccuracy but close enough
        let expected = [0.5000001, 0.4999999, 0.24999996, 1.0];
        let color = colors.color_at(0.45);
        assert_eq!(color, expected);

        let expected = [1.0, 0.0, 0.0, 1.0];
        let color = colors.color_at(0.2);
        assert_eq!(color, expected);

        let expected = [0.0, 1.0, 0.5, 1.0];
        let color = colors.color_at(0.9);
        assert_eq!(color, expected);
    }

    #[test]
    fn color_at_multiple() {
        let colors = Gradient::from_slice(&[
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]);

        let expected = [1.0, 0.25, 0.0, 1.0];
        let color = colors.color_at(0.0625);
        assert_eq!(color, expected);

        let expected = [0.0, 1.0, 0.0, 1.0];
        let color = colors.color_at(0.5);
        assert_eq!(color, expected);

        let expected = [0.0, 1.0, 0.5, 1.0];
        let color = colors.color_at(0.625);
        assert_eq!(color, expected);
    }
}
