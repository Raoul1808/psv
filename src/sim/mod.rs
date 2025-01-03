use std::{collections::VecDeque, time::Duration};

use egui::Widget;
use parser::parse_push_swap;

mod parser;

pub type Stack = VecDeque<u32>;

#[derive(Debug)]
pub enum PushSwapInstruction {
    SwapA,
    SwapB,
    SwapBoth,
    PushA,
    PushB,
    RotateA,
    RotateB,
    RotateBoth,
    ReverseRotateA,
    ReverseRotateB,
    ReverseRotateBoth,
}

impl PushSwapInstruction {
    pub fn execute(&self, stack_a: &mut Stack, stack_b: &mut Stack) {
        use PushSwapInstruction::*;
        match *self {
            SwapA => {
                if stack_a.len() >= 2 {
                    (stack_a[0], stack_a[1]) = (stack_a[1], stack_a[0]);
                }
            }
            SwapB => {
                if stack_b.len() >= 2 {
                    (stack_b[0], stack_b[1]) = (stack_b[1], stack_b[0]);
                }
            }
            SwapBoth => {
                SwapA.execute(stack_a, stack_b);
                SwapB.execute(stack_a, stack_b);
            }
            PushA => {
                if let Some(value) = stack_b.pop_front() {
                    stack_a.push_front(value);
                }
            }
            PushB => {
                if let Some(value) = stack_a.pop_front() {
                    stack_b.push_front(value);
                }
            }
            RotateA => {
                if let Some(value) = stack_a.pop_front() {
                    stack_a.push_back(value);
                }
            }
            RotateB => {
                if let Some(value) = stack_b.pop_front() {
                    stack_b.push_back(value);
                }
            }
            RotateBoth => {
                RotateA.execute(stack_a, stack_b);
                RotateB.execute(stack_a, stack_b);
            }
            ReverseRotateA => {
                if let Some(value) = stack_a.pop_back() {
                    stack_a.push_front(value);
                }
            }
            ReverseRotateB => {
                if let Some(value) = stack_b.pop_back() {
                    stack_b.push_front(value);
                }
            }
            ReverseRotateBoth => {
                ReverseRotateA.execute(stack_a, stack_b);
                ReverseRotateB.execute(stack_a, stack_b);
            }
        }
    }

    pub fn undo(&self, stack_a: &mut Stack, stack_b: &mut Stack) {
        use PushSwapInstruction::*;
        match *self {
            SwapA => SwapA.execute(stack_a, stack_b),
            SwapB => SwapB.execute(stack_a, stack_b),
            SwapBoth => SwapBoth.execute(stack_a, stack_b),
            PushA => PushB.execute(stack_a, stack_b),
            PushB => PushA.execute(stack_a, stack_b),
            RotateA => ReverseRotateA.execute(stack_a, stack_b),
            RotateB => ReverseRotateB.execute(stack_a, stack_b),
            RotateBoth => ReverseRotateBoth.execute(stack_a, stack_b),
            ReverseRotateA => RotateA.execute(stack_a, stack_b),
            ReverseRotateB => RotateB.execute(stack_a, stack_b),
            ReverseRotateBoth => RotateBoth.execute(stack_a, stack_b),
        }
    }
}

#[derive(Default)]
pub struct PushSwapSim {
    instructions: Vec<PushSwapInstruction>,
    program_counter: usize,
    stack_a: Stack,
    stack_b: Stack,
}

fn normalized_vec(numbers: &[i64]) -> Vec<u32> {
    let mut numbers: Vec<_> = numbers.iter().enumerate().collect();
    numbers.sort_by(|(_, i1), (_, i2)| i1.cmp(i2));
    let mut numbers: Vec<_> = numbers
        .into_iter()
        .enumerate()
        .map(|(i, (ii, _))| (i, ii))
        .collect();
    numbers.sort_by(|(_, i1), (_, i2)| i1.cmp(i2));
    numbers.into_iter().map(|(i, _)| i as u32).collect()
}

impl PushSwapSim {
    pub fn load_normalized(&mut self, numbers: Vec<u32>, text: &str) -> Result<(), usize> {
        self.instructions = parse_push_swap(text)?;
        self.program_counter = 0;
        self.stack_a = VecDeque::from(numbers);
        self.stack_b = VecDeque::new();
        Ok(())
    }

    pub fn load_random(&mut self, numbers: &[i64], text: &str) -> Result<(), usize> {
        let numbers = normalized_vec(numbers);
        self.load_normalized(numbers, text)
    }

    pub fn make_contiguous(&mut self) {
        let _ = self.stack_a.make_contiguous();
        let _ = self.stack_b.make_contiguous();
    }

    pub fn stack_a(&self) -> &[u32] {
        self.stack_a.as_slices().0
    }

    pub fn stack_b(&self) -> &[u32] {
        self.stack_b.as_slices().0
    }

    pub fn step(&mut self) -> bool {
        if self.program_counter >= self.instructions.len() {
            return false;
        }
        self.instructions[self.program_counter].execute(&mut self.stack_a, &mut self.stack_b);
        self.program_counter += 1;
        true
    }

    pub fn undo(&mut self) -> bool {
        if self.program_counter == 0 {
            return false;
        }
        self.program_counter -= 1;
        self.instructions[self.program_counter].undo(&mut self.stack_a, &mut self.stack_b);
        true
    }

    pub fn skip_to(&mut self, counter: usize) -> bool {
        let mut needs_redraw = false;
        let counter = counter.clamp(0, self.instructions.len());
        while self.program_counter < counter {
            needs_redraw |= self.step();
        }
        while self.program_counter > counter {
            needs_redraw |= self.undo();
        }
        needs_redraw
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        play_sim: &mut bool,
        exec_duration: &mut Duration,
    ) -> bool {
        let mut needs_redraw = false;
        egui::Window::new("Visualization Playback").show(ctx, |ui| {
            ui.label(format!("Instructions loaded: {}", self.instructions.len()));
            ui.label(format!("Program Counter: {}", self.program_counter));
            ui.scope(|ui| {
                ui.style_mut().spacing.slider_width = ui.available_width();
                let mut counter = self.program_counter;
                let max = if self.instructions.is_empty() {
                    0
                } else {
                    self.instructions.len()
                };
                let slider = ui.add_enabled(
                    !self.instructions.is_empty(),
                    egui::Slider::new(&mut counter, 0..=max).show_value(false),
                );
                if slider.changed() {
                    self.skip_to(counter);
                }
            });
            ui.horizontal(|ui| {
                let start_cond = self.program_counter > 0;
                let end_cond = self.program_counter < self.instructions.len();
                let undo_cond = !*play_sim && start_cond;
                let step_cond = !*play_sim && end_cond;
                if ui
                    .add_enabled(start_cond, egui::Button::new("<<"))
                    .clicked()
                {
                    while self.undo() {}
                    needs_redraw = true;
                }
                if ui.add_enabled(undo_cond, egui::Button::new("<")).clicked() {
                    needs_redraw = self.undo();
                }
                if *play_sim {
                    if ui.button("Pause").clicked() {
                        *play_sim = false;
                    }
                } else if ui.button("Play").clicked() {
                    *play_sim = true;
                }
                if ui.add_enabled(step_cond, egui::Button::new(">")).clicked() {
                    needs_redraw = self.step();
                }
                if ui.add_enabled(end_cond, egui::Button::new(">>")).clicked() {
                    while self.step() {}
                    needs_redraw = true;
                }
            });
            ui.horizontal(|ui| {
                let mut millis = exec_duration.as_millis() as u64;
                egui::Slider::new(&mut millis, 1..=100)
                    .show_value(false)
                    .ui(ui);
                *exec_duration = Duration::from_millis(millis);
                ui.label(format!("{}ms exec rate", millis));
            });
        });
        needs_redraw
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
        self.stack_a.clear();
        self.stack_b.clear();
        self.program_counter = 0;
    }
}

#[cfg(test)]
mod test {
    use super::normalized_vec;

    #[test]
    fn test_normalizer() {
        let chaotic = &[39512, -727, 1116, -525, 0, 32457, -42, -9837, 69, 52];
        let expected = &[9, 1, 7, 2, 4, 8, 3, 0, 6, 5];

        let res = normalized_vec(chaotic);
        assert_eq!(res.as_slice(), expected);
    }
}
