#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PushSwapError {
    #[error("invalid instruction `{0}`")]
    InvalidInstruction(String),

    #[error("duplicate number `{0}`")]
    DuplicateNumber(i32),
}
