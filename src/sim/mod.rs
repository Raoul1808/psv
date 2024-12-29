use std::collections::VecDeque;

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

impl PushSwapSim {
    pub fn load(&mut self, numbers: &[u32], text: &str) -> Result<(), usize> {
        self.instructions = parse_push_swap(text)?;
        self.program_counter = 0;
        let vec = Vec::from(numbers);
        self.stack_a = VecDeque::from(vec);
        self.stack_b = VecDeque::new();
        Ok(())
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

    pub fn ui(&mut self, ctx: &egui::Context) -> bool {
        let mut needs_redraw = false;
        egui::Window::new("Visualization Playback").show(ctx, |ui| {
            ui.label(format!("Instructions loaded: {}", self.instructions.len()));
            ui.label(format!("Program Counter: {}", self.program_counter));
            ui.horizontal(|ui| {
                let undo_cond = self.program_counter > 0;
                let step_cond = self.program_counter < self.instructions.len();
                if ui.add_enabled(undo_cond, egui::Button::new("<<")).clicked() {
                    while self.undo() {}
                    needs_redraw = true;
                }
                if ui.add_enabled(undo_cond, egui::Button::new("<")).clicked() {
                    needs_redraw = self.undo();
                }
                if ui.add_enabled(step_cond, egui::Button::new(">")).clicked() {
                    needs_redraw = self.step();
                }
                if ui.add_enabled(step_cond, egui::Button::new(">>")).clicked() {
                    while self.step() {}
                    needs_redraw = true;
                }
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
