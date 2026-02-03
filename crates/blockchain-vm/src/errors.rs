use thiserror::Error;

#[derive(Debug, Error)]
pub enum VmError {
    #[error("Stack overflow: max depth {0}")]
    StackOverflow(usize),

    #[error("Stack underflow: needed {needed}, got {got}")]
    StackUnderflow { needed: usize, got: usize },

    #[error("Gas limit exceeded: max {0} steps")]
    GasLimitExceeded(u64),

    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid jump target: {0}")]
    InvalidJump(usize),

    #[error("Program counter out of bounds: {pc} >= {len}")]
    PcOutOfBounds { pc: usize, len: usize },

    #[error("Compilation error: {0}")]
    CompileError(String),

    #[error("Contract error: {0}")]
    ContractError(String),
}

pub type VmResult<T> = Result<T, VmError>;
