use std::collections::HashMap;

use crate::errors::{VmError, VmResult};
use crate::opcodes::OpCode;

const MAX_STACK_SIZE: usize = 1024;
const MAX_STEPS: u64 = 100_000;

#[derive(Debug)]
pub struct ExecutionResult {
    pub stack: Vec<i64>,
    pub storage: HashMap<u64, i64>,
    pub logs: Vec<i64>,
    pub steps_used: u64,
}

pub struct VM {
    stack: Vec<i64>,
    pc: usize,
    storage: HashMap<u64, i64>,
    logs: Vec<i64>,
    steps: u64,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(MAX_STACK_SIZE),
            pc: 0,
            storage: HashMap::new(),
            logs: Vec::new(),
            steps: 0,
        }
    }

    pub fn with_storage(mut self, storage: HashMap<u64, i64>) -> Self {
        self.storage = storage;
        self
    }

    pub fn execute(&mut self, bytecode: &[u8]) -> VmResult<ExecutionResult> {
        self.pc = 0;
        self.steps = 0;

        while self.pc < bytecode.len() {
            if self.steps >= MAX_STEPS {
                return Err(VmError::GasLimitExceeded(MAX_STEPS));
            }
            self.steps += 1;

            let opcode_byte = bytecode[self.pc];
            let opcode = OpCode::from_byte(opcode_byte)
                .ok_or(VmError::InvalidOpcode(opcode_byte))?;

            match opcode {
                OpCode::Push => {
                    self.pc += 1;
                    let value = self.read_i64(bytecode)?;
                    self.push(value)?;
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Dup => {
                    let val = *self.peek()?;
                    self.push(val)?;
                }
                OpCode::Swap => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a)?;
                    self.push(b)?;
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.wrapping_add(b))?;
                }
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.wrapping_sub(b))?;
                }
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.wrapping_mul(b))?;
                }
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    self.push(a / b)?;
                }
                OpCode::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    self.push(a % b)?;
                }
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if a == b { 1 } else { 0 })?;
                }
                OpCode::Lt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if a < b { 1 } else { 0 })?;
                }
                OpCode::Gt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if a > b { 1 } else { 0 })?;
                }
                OpCode::Not => {
                    let a = self.pop()?;
                    self.push(if a == 0 { 1 } else { 0 })?;
                }
                OpCode::Jump => {
                    let target = self.pop()? as usize;
                    if target >= bytecode.len() {
                        return Err(VmError::InvalidJump(target));
                    }
                    self.pc = target;
                    continue;
                }
                OpCode::JumpIf => {
                    let target = self.pop()? as usize;
                    let condition = self.pop()?;
                    if condition != 0 {
                        if target >= bytecode.len() {
                            return Err(VmError::InvalidJump(target));
                        }
                        self.pc = target;
                        continue;
                    }
                }
                OpCode::Halt => break,
                OpCode::Store => {
                    let value = self.pop()?;
                    let key = self.pop()? as u64;
                    self.storage.insert(key, value);
                }
                OpCode::Load => {
                    let key = self.pop()? as u64;
                    let value = self.storage.get(&key).copied().unwrap_or(0);
                    self.push(value)?;
                }
                OpCode::Log => {
                    let value = self.pop()?;
                    self.logs.push(value);
                    tracing::debug!("VM LOG: {}", value);
                }
            }

            self.pc += 1;
        }

        Ok(ExecutionResult {
            stack: self.stack.clone(),
            storage: self.storage.clone(),
            logs: self.logs.clone(),
            steps_used: self.steps,
        })
    }

    fn push(&mut self, value: i64) -> VmResult<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow(MAX_STACK_SIZE));
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> VmResult<i64> {
        self.stack
            .pop()
            .ok_or(VmError::StackUnderflow { needed: 1, got: 0 })
    }

    fn peek(&self) -> VmResult<&i64> {
        self.stack
            .last()
            .ok_or(VmError::StackUnderflow { needed: 1, got: 0 })
    }

    fn read_i64(&mut self, bytecode: &[u8]) -> VmResult<i64> {
        if self.pc + 8 > bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                len: bytecode.len(),
            });
        }
        let bytes: [u8; 8] = bytecode[self.pc..self.pc + 8]
            .try_into()
            .unwrap();
        self.pc += 7; // +1 will happen in main loop
        Ok(i64::from_le_bytes(bytes))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_val(bytecode: &mut Vec<u8>, val: i64) {
        bytecode.push(OpCode::Push as u8);
        bytecode.extend_from_slice(&val.to_le_bytes());
    }

    #[test]
    fn test_push_and_add() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 10);
        push_val(&mut bytecode, 20);
        bytecode.push(OpCode::Add as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![30]);
    }

    #[test]
    fn test_arithmetic() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 100);
        push_val(&mut bytecode, 30);
        bytecode.push(OpCode::Sub as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![70]);
    }

    #[test]
    fn test_store_and_load() {
        let mut bytecode = Vec::new();
        // Store 42 at key 0
        push_val(&mut bytecode, 0);  // key
        push_val(&mut bytecode, 42); // value
        bytecode.push(OpCode::Store as u8);
        // Load key 0
        push_val(&mut bytecode, 0); // key
        bytecode.push(OpCode::Load as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![42]);
        assert_eq!(result.storage.get(&0), Some(&42));
    }

    #[test]
    fn test_comparison() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 10);
        push_val(&mut bytecode, 20);
        bytecode.push(OpCode::Lt as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![1]);
    }

    #[test]
    fn test_log() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 42);
        bytecode.push(OpCode::Log as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.logs, vec![42]);
    }

    #[test]
    fn test_division_by_zero() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 10);
        push_val(&mut bytecode, 0);
        bytecode.push(OpCode::Div as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_stack_underflow() {
        let mut bytecode = Vec::new();
        bytecode.push(OpCode::Pop as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);
        assert!(matches!(result, Err(VmError::StackUnderflow { .. })));
    }

    #[test]
    fn test_dup_and_swap() {
        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 5);
        bytecode.push(OpCode::Dup as u8);
        bytecode.push(OpCode::Add as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![10]);
    }

    #[test]
    fn test_with_existing_storage() {
        let mut storage = HashMap::new();
        storage.insert(0, 100);

        let mut bytecode = Vec::new();
        push_val(&mut bytecode, 0);
        bytecode.push(OpCode::Load as u8);
        bytecode.push(OpCode::Halt as u8);

        let mut vm = VM::new().with_storage(storage);
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![100]);
    }
}
