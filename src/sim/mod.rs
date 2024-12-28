use std::collections::VecDeque;

mod parser;

pub type Stack = VecDeque<u32>;

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
}

pub struct PushSwapSim {
    instructions: Vec<PushSwapInstruction>,
    program_counter: usize,
}

impl PushSwapSim {}
