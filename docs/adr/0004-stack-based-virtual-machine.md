# ADR-0004: Stack-Based Virtual Machine

## Status
Accepted

## Context
Smart contracts need a runtime environment. Options considered: register-based VM, stack-based VM, WASM runtime.

## Decision
Implement a custom stack-based VM with:
- 18 opcodes (arithmetic, comparison, control flow, storage, logging)
- Max stack depth: 1024
- Gas limit: 100,000 steps per execution
- Persistent storage per contract (HashMap<u64, i64>)
- Text assembly compiler for human-readable contract source

Opcodes: PUSH, POP, DUP, SWAP, ADD, SUB, MUL, DIV, MOD, EQ, LT, GT, NOT, JUMP, JUMPIF, STORE, LOAD, LOG, HALT.

## Consequences
- Simple to implement and debug
- Gas limit prevents infinite loops
- Storage persists between contract calls
- Limited instruction set; can be extended with new opcodes
- No floating point support (integer-only arithmetic)
- Text assembly is readable but not a high-level language
