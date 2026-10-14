use std::{collections::VecDeque, fmt::Display, ops::Not, str::FromStr};

use crate::sim::error::PushSwapError;

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
    pub fn run<T>(self, sa: &mut VecDeque<T>, sb: &mut VecDeque<T>) {
        use PushSwapInstruction::*;
        match self {
            SwapA => {
                if sa.len() >= 2 {
                    sa.swap(0, 1);
                }
            }
            SwapB => {
                if sb.len() >= 2 {
                    sb.swap(0, 1);
                }
            }
            SwapBoth => {
                SwapA.run(sa, sb);
                SwapB.run(sa, sb);
            }
            PushA => {
                if let Some(t) = sb.pop_front() {
                    sa.push_front(t);
                }
            }
            PushB => {
                if let Some(t) = sa.pop_front() {
                    sb.push_front(t);
                }
            }
            RotateA => {
                if !sa.is_empty() {
                    sa.rotate_left(1);
                }
            }
            RotateB => {
                if !sb.is_empty() {
                    sb.rotate_left(1);
                }
            }
            RotateBoth => {
                RotateA.run(sa, sb);
                RotateB.run(sa, sb);
            }
            ReverseRotateA => {
                if !sa.is_empty() {
                    sa.rotate_right(1);
                }
            }
            ReverseRotateB => {
                if !sb.is_empty() {
                    sb.rotate_right(1);
                }
            }
            ReverseRotateBoth => {
                ReverseRotateA.run(sa, sb);
                ReverseRotateB.run(sa, sb);
            }
        }
    }
}

impl Not for PushSwapInstruction {
    type Output = PushSwapInstruction;

    fn not(self) -> Self::Output {
        use PushSwapInstruction::*;
        match self {
            SwapA => SwapA,
            SwapB => SwapB,
            SwapBoth => SwapBoth,
            PushA => PushB,
            PushB => PushA,
            RotateA => ReverseRotateA,
            RotateB => ReverseRotateB,
            RotateBoth => ReverseRotateBoth,
            ReverseRotateA => RotateA,
            ReverseRotateB => RotateB,
            ReverseRotateBoth => RotateBoth,
        }
    }
}

impl FromStr for PushSwapInstruction {
    type Err = PushSwapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PushSwapInstruction::*;
        let ins = match s {
            "sa" => SwapA,
            "sb" => SwapB,
            "ss" => SwapBoth,
            "pa" => PushA,
            "pb" => PushB,
            "ra" => RotateA,
            "rb" => RotateB,
            "rr" => RotateBoth,
            "rra" => ReverseRotateA,
            "rrb" => ReverseRotateB,
            "rrr" => ReverseRotateBoth,
            _ => return Err(PushSwapError::InvalidInstruction(s.to_string())),
        };
        Ok(ins)
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
        write!(f, "{str}")
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::sim::{error::PushSwapError, instructions::PushSwapInstruction};

    #[test]
    fn valid_instructions() {
        use super::PushSwapInstruction::*;

        let instructions_str = "pa pb ss ra ra rrb sa sb rb rra rr rrr";
        let expected = vec![
            PushA,
            PushB,
            SwapBoth,
            RotateA,
            RotateA,
            ReverseRotateB,
            SwapA,
            SwapB,
            RotateB,
            ReverseRotateA,
            RotateBoth,
            ReverseRotateBoth,
        ];

        let instructions = instructions_str
            .split_whitespace()
            .map(|i| i.parse::<PushSwapInstruction>())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(instructions, expected);
    }

    #[test]
    fn invalid_instruction() {
        let res = "pp".parse::<PushSwapInstruction>();
        assert!(res.is_err());

        let err = res.unwrap_err();
        let expected = PushSwapError::InvalidInstruction("pp".to_string());
        assert_eq!(err, expected);
    }

    macro_rules! assert_stack {
        ($stack:expr, $nums:expr) => {
            $stack.make_contiguous();
            let slc = $stack.as_slices().0;
            assert_eq!(slc, $nums.as_slice() as &[i32]);
        };
    }

    #[test]
    fn run_instructions() {
        let mut stack_a = VecDeque::from([1, 2, 3, 4]);
        let mut stack_b = VecDeque::new();

        let sa = &mut stack_a;
        let sb = &mut stack_b;

        use PushSwapInstruction::*;

        PushB.run(sa, sb);
        assert_stack!(sa, [2, 3, 4]);
        assert_stack!(sb, [1]);

        PushB.run(sa, sb);
        assert_stack!(sa, [3, 4]);
        assert_stack!(sb, [2, 1]);

        SwapA.run(sa, sb);
        assert_stack!(sa, [4, 3]);

        SwapB.run(sa, sb);
        assert_stack!(sb, [1, 2]);

        SwapBoth.run(sa, sb);
        assert_stack!(sa, [3, 4]);
        assert_stack!(sb, [2, 1]);

        PushA.run(sa, sb);
        assert_stack!(sa, [2, 3, 4]);
        assert_stack!(sb, [1]);

        SwapA.run(sa, sb);
        assert_stack!(sa, [3, 2, 4]);

        RotateA.run(sa, sb);
        assert_stack!(sa, [2, 4, 3]);

        ReverseRotateA.run(sa, sb);
        assert_stack!(sa, [3, 2, 4]);

        PushB.run(sa, sb);
        PushB.run(sa, sb);
        assert_stack!(sb, [2, 3, 1]);

        RotateB.run(sa, sb);
        assert_stack!(sb, [3, 1, 2]);

        ReverseRotateB.run(sa, sb);
        assert_stack!(sb, [2, 3, 1]);

        PushA.run(sa, sb);
        assert_stack!(sa, [2, 4]);
        assert_stack!(sb, [3, 1]);

        RotateBoth.run(sa, sb);
        assert_stack!(sa, [4, 2]);
        assert_stack!(sb, [1, 3]);

        ReverseRotateBoth.run(sa, sb);
        assert_stack!(sa, [2, 4]);
        assert_stack!(sb, [3, 1]);
    }

    #[test]
    fn undo_instructions() {
        let mut stack_a = VecDeque::from([1, 2, 3, 4]);
        let mut stack_b = VecDeque::new();

        let sa = &mut stack_a;
        let sb = &mut stack_b;

        use PushSwapInstruction::*;

        PushB.run(sa, sb);
        PushB.run(sa, sb);
        SwapA.run(sa, sb);
        PushB.run(sa, sb);
        ReverseRotateB.run(sa, sb);

        assert_stack!(sa, [3]);
        assert_stack!(sb, [1, 4, 2]);

        (!ReverseRotateB).run(sa, sb);
        (!PushB).run(sa, sb);
        (!SwapA).run(sa, sb);
        (!PushB).run(sa, sb);
        (!PushB).run(sa, sb);

        assert_stack!(sa, [1, 2, 3, 4]);
        assert_stack!(sb, []);
    }
}
