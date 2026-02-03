use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    Push = 0x01,
    Pop = 0x02,
    Dup = 0x03,
    Swap = 0x04,

    // Arithmetic
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Mod = 0x14,

    // Comparison
    Eq = 0x20,
    Lt = 0x21,
    Gt = 0x22,
    Not = 0x23,

    // Control flow
    Jump = 0x30,
    JumpIf = 0x31,
    Halt = 0x3F,

    // Storage
    Store = 0x40,
    Load = 0x41,

    // Logging
    Log = 0x50,
}

impl OpCode {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Push),
            0x02 => Some(Self::Pop),
            0x03 => Some(Self::Dup),
            0x04 => Some(Self::Swap),
            0x10 => Some(Self::Add),
            0x11 => Some(Self::Sub),
            0x12 => Some(Self::Mul),
            0x13 => Some(Self::Div),
            0x14 => Some(Self::Mod),
            0x20 => Some(Self::Eq),
            0x21 => Some(Self::Lt),
            0x22 => Some(Self::Gt),
            0x23 => Some(Self::Not),
            0x30 => Some(Self::Jump),
            0x31 => Some(Self::JumpIf),
            0x3F => Some(Self::Halt),
            0x40 => Some(Self::Store),
            0x41 => Some(Self::Load),
            0x50 => Some(Self::Log),
            _ => None,
        }
    }
}
