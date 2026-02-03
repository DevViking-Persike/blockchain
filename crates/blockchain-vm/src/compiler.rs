use crate::errors::{VmError, VmResult};
use crate::opcodes::OpCode;

/// Compiles text assembly into bytecode.
///
/// Assembly format (one instruction per line):
///   PUSH <i64>
///   POP
///   DUP
///   SWAP
///   ADD / SUB / MUL / DIV / MOD
///   EQ / LT / GT / NOT
///   JUMP / JUMPIF
///   STORE / LOAD
///   LOG
///   HALT
pub fn compile(source: &str) -> VmResult<Vec<u8>> {
    let mut bytecode = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let instruction = parts[0].to_uppercase();
        match instruction.as_str() {
            "PUSH" => {
                let value: i64 = parts
                    .get(1)
                    .ok_or_else(|| {
                        VmError::CompileError(format!(
                            "Line {}: PUSH requires a value",
                            line_num + 1
                        ))
                    })?
                    .parse()
                    .map_err(|e| {
                        VmError::CompileError(format!(
                            "Line {}: invalid number: {}",
                            line_num + 1,
                            e
                        ))
                    })?;
                bytecode.push(OpCode::Push as u8);
                bytecode.extend_from_slice(&value.to_le_bytes());
            }
            "POP" => bytecode.push(OpCode::Pop as u8),
            "DUP" => bytecode.push(OpCode::Dup as u8),
            "SWAP" => bytecode.push(OpCode::Swap as u8),
            "ADD" => bytecode.push(OpCode::Add as u8),
            "SUB" => bytecode.push(OpCode::Sub as u8),
            "MUL" => bytecode.push(OpCode::Mul as u8),
            "DIV" => bytecode.push(OpCode::Div as u8),
            "MOD" => bytecode.push(OpCode::Mod as u8),
            "EQ" => bytecode.push(OpCode::Eq as u8),
            "LT" => bytecode.push(OpCode::Lt as u8),
            "GT" => bytecode.push(OpCode::Gt as u8),
            "NOT" => bytecode.push(OpCode::Not as u8),
            "JUMP" => bytecode.push(OpCode::Jump as u8),
            "JUMPIF" => bytecode.push(OpCode::JumpIf as u8),
            "HALT" => bytecode.push(OpCode::Halt as u8),
            "STORE" => bytecode.push(OpCode::Store as u8),
            "LOAD" => bytecode.push(OpCode::Load as u8),
            "LOG" => bytecode.push(OpCode::Log as u8),
            _ => {
                return Err(VmError::CompileError(format!(
                    "Line {}: unknown instruction '{}'",
                    line_num + 1,
                    instruction
                )));
            }
        }
    }

    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_compile_and_run() {
        let source = r#"
            PUSH 10
            PUSH 20
            ADD
            HALT
        "#;
        let bytecode = compile(source).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![30]);
    }

    #[test]
    fn test_compile_with_comments() {
        let source = r#"
            # This adds two numbers
            PUSH 5
            PUSH 3
            ADD  ; add them
            LOG
            HALT
        "#;
        let bytecode = compile(source).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.logs, vec![8]);
    }

    #[test]
    fn test_compile_store_load() {
        let source = r#"
            PUSH 0
            PUSH 42
            STORE
            PUSH 0
            LOAD
            HALT
        "#;
        let bytecode = compile(source).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();
        assert_eq!(result.stack, vec![42]);
    }

    #[test]
    fn test_compile_unknown_instruction() {
        let source = "UNKNOWN 42";
        let result = compile(source);
        assert!(matches!(result, Err(VmError::CompileError(_))));
    }

    #[test]
    fn test_compile_push_missing_value() {
        let source = "PUSH";
        let result = compile(source);
        assert!(matches!(result, Err(VmError::CompileError(_))));
    }
}
