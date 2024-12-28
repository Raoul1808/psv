use super::PushSwapInstruction;

pub fn parse_push_swap(text: &str) -> Result<Vec<PushSwapInstruction>, usize> {
    let raw = text.split_whitespace().enumerate();
    let mut instructions = vec![];
    for (i, ins) in raw {
        let i = i + 1;
        let ins = {
            use PushSwapInstruction::*;
            match ins {
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
                _ => return Err(i),
            }
        };
        instructions.push(ins);
    }
    Ok(instructions)
}
