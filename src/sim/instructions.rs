use std::str::FromStr;

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

#[cfg(test)]
mod test {
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
}
