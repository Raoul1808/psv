use std::collections::{HashSet, VecDeque};

use crate::sim::{error::PushSwapError, instructions::PushSwapInstruction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumPair {
    orig: i32,
    normalized: i32,
}

fn normalize(nums: &[i32]) -> Vec<NumPair> {
    let mut vec: Vec<_> = nums.iter().enumerate().collect();
    vec.sort_by_key(|(_, n)| *n);
    let mut vec: Vec<_> = vec.into_iter().enumerate().collect();
    vec.sort_by_key(|(_, (i, _))| *i);
    vec.into_iter()
        .map(|(n, (_, o))| NumPair {
            orig: *o,
            normalized: n as i32,
        })
        .collect()
}

fn check_duplicates(nums: &[i32]) -> Result<(), PushSwapError> {
    let mut uniq = HashSet::new();
    nums.iter().try_for_each(move |x| {
        uniq.insert(x)
            .then_some(())
            .ok_or(PushSwapError::DuplicateNumber(*x))
    })
}

pub struct Sim {
    instructions: Vec<PushSwapInstruction>,
    program_counter: usize,
    stack_a: VecDeque<NumPair>,
    stack_b: VecDeque<NumPair>,
}

impl Sim {
    pub fn new(
        nums: &[i32],
        instructions: Vec<PushSwapInstruction>,
    ) -> Result<Self, PushSwapError> {
        check_duplicates(nums)?;
        let numbers = normalize(nums);
        let stack_a = VecDeque::from(numbers);
        let mut stack_b = VecDeque::new();
        stack_b.reserve_exact(stack_a.len());
        Ok(Self {
            instructions,
            program_counter: 0,
            stack_a,
            stack_b,
        })
    }

    pub fn step(&mut self) -> bool {
        if self.program_counter >= self.instructions.len() {
            return false;
        }

        self.instructions[self.program_counter].run(&mut self.stack_a, &mut self.stack_b);
        self.program_counter += 1;
        true
    }

    pub fn undo(&mut self) -> bool {
        if self.program_counter == 0 {
            return false;
        }

        self.program_counter -= 1;
        (!self.instructions[self.program_counter]).run(&mut self.stack_a, &mut self.stack_b);
        true
    }

    pub fn make_contiguous(&mut self) {
        self.stack_a.make_contiguous();
        self.stack_b.make_contiguous();
    }

    pub fn as_slices(&self) -> (&[NumPair], &[NumPair]) {
        (self.stack_a.as_slices().0, self.stack_b.as_slices().0)
    }

    pub fn is_sorted(&self) -> bool {
        self.stack_b.is_empty() && self.stack_a.iter().is_sorted_by_key(|p| p.orig)
    }

    pub fn program_counter(&self) -> usize {
        self.program_counter
    }
}

#[cfg(test)]
mod test {
    use crate::sim::{
        error::PushSwapError,
        instructions::PushSwapInstruction,
        logic::{NumPair, Sim, check_duplicates, normalize},
    };

    #[inline]
    fn np(o: i32, n: i32) -> NumPair {
        NumPair {
            orig: o,
            normalized: n,
        }
    }

    #[test]
    fn normalize_nums() {
        let nums = &[-42, 21, 1116, -727, 9, -81];
        let expected = vec![
            np(-42, 2),
            np(21, 4),
            np(1116, 5),
            np(-727, 0),
            np(9, 3),
            np(-81, 1),
        ];

        let nums = normalize(nums);
        assert_eq!(nums, expected);
    }

    #[test]
    fn duplicate_check() {
        let unique_nums = &[1, 2, 3, 4, 5, 6, 7];

        let res = check_duplicates(unique_nums);
        assert!(res.is_ok());

        let duplicate_num = &[1, 2, 3, 4, 5, 6, 7, 2, 8];
        let res = check_duplicates(duplicate_num);
        assert!(res.is_err());
        let err = res.unwrap_err();
        let expected = PushSwapError::DuplicateNumber(2);
        assert_eq!(err, expected);
    }

    macro_rules! assert_sim {
        ($sim:expr, $sa:expr, $sb:expr) => {
            let sa = $sa;
            let sb = $sb;
            let expected = (sa.as_slice(), sb.as_slice());
            $sim.make_contiguous();
            let slices = $sim.as_slices();
            assert_eq!(slices, expected);
        };
    }

    #[test]
    fn step_sim() {
        use PushSwapInstruction::*;
        let mut sim = Sim::new(
            &[4, 7, 2, 8, 3],
            vec![RotateA, RotateA, PushB, RotateA, PushB, PushA, PushA],
        )
        .unwrap();

        let n4 = np(4, 2);
        let n7 = np(7, 3);
        let n2 = np(2, 0);
        let n8 = np(8, 4);
        let n3 = np(3, 1);

        assert!(!sim.is_sorted());
        assert_eq!(sim.program_counter(), 0);
        assert_sim!(sim, [n4, n7, n2, n8, n3], []);
        sim.step();
        assert_sim!(sim, [n7, n2, n8, n3, n4], []);
        sim.step();
        assert_sim!(sim, [n2, n8, n3, n4, n7], []);
        sim.step();
        assert_sim!(sim, [n8, n3, n4, n7], [n2]);
        sim.step();
        assert_sim!(sim, [n3, n4, n7, n8], [n2]);
        sim.step();
        assert_sim!(sim, [n4, n7, n8], [n3, n2]);
        sim.step();
        assert_sim!(sim, [n3, n4, n7, n8], [n2]);
        sim.step();
        assert_sim!(sim, [n2, n3, n4, n7, n8], []);
        assert!(sim.is_sorted());
        assert_eq!(sim.program_counter(), 7);
    }

    #[test]
    fn undo_sim() {
        use PushSwapInstruction::*;
        let mut sim = Sim::new(
            &[4, 7, 2, 8, 3],
            vec![RotateA, RotateA, PushB, RotateA, PushB, PushA, PushA],
        )
        .unwrap();

        let n4 = np(4, 2);
        let n7 = np(7, 3);
        let n2 = np(2, 0);
        let n8 = np(8, 4);
        let n3 = np(3, 1);

        while sim.step() {}
        assert_eq!(sim.program_counter(), 7);
        assert_sim!(sim, [n2, n3, n4, n7, n8], []);
        while sim.undo() {}
        assert_eq!(sim.program_counter(), 0);
        assert_sim!(sim, [n4, n7, n2, n8, n3], []);
    }
}
