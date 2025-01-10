use std::{collections::VecDeque, fmt::Display};

use parser::parse_push_swap;

mod parser;

pub type Stack = VecDeque<u32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Display for PushSwapInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PushSwapInstruction::*;
        let str = match self {
            SwapA => "sa",
            SwapB => "sb",
            SwapBoth => "ss",
            PushA => "pa",
            PushB => "pb",
            RotateA => "ra",
            RotateB => "rb",
            RotateBoth => "rr",
            ReverseRotateA => "rra",
            ReverseRotateB => "rrb",
            ReverseRotateBoth => "rrr",
        };
        write!(f, "{}", str)
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

    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    pub fn instructions(&self) -> &[PushSwapInstruction] {
        &self.instructions
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
