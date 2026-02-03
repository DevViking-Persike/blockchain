use sha2::{Digest, Sha256};

use blockchain_core::state::WorldState;

use crate::errors::{VmError, VmResult};
use crate::vm::VM;

#[derive(Debug)]
pub struct ContractResult {
    pub logs: Vec<i64>,
    pub stack_top: Option<i64>,
    pub steps_used: u64,
}

pub struct ContractExecutor;

impl ContractExecutor {
    pub fn deploy(
        state: &mut WorldState,
        sender: &str,
        bytecode: Vec<u8>,
    ) -> VmResult<String> {
        // Generate contract address from sender + bytecode hash
        let mut hasher = Sha256::new();
        hasher.update(sender.as_bytes());
        hasher.update(&bytecode);
        hasher.update(
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        let hash = hex::encode(hasher.finalize());
        let address = format!("0xc{}", &hash[..39]);

        state.deploy_contract(address.clone(), bytecode, sender.to_string());

        tracing::info!("Contract deployed at {} by {}", address, sender);
        Ok(address)
    }

    pub fn call(
        state: &mut WorldState,
        contract_address: &str,
        _call_data: &[u8],
    ) -> VmResult<ContractResult> {
        let contract = state
            .get_contract(contract_address)
            .ok_or_else(|| {
                VmError::ContractError(format!(
                    "Contract not found: {}",
                    contract_address
                ))
            })?;

        let bytecode = contract.bytecode.clone();
        let storage = contract.storage.clone();

        let mut vm = VM::new().with_storage(storage);
        let result = vm.execute(&bytecode)?;

        // Update contract storage
        if let Some(contract) = state.get_contract_mut(contract_address) {
            contract.storage = result.storage;
        }

        Ok(ContractResult {
            stack_top: result.stack.last().copied(),
            logs: result.logs,
            steps_used: result.steps_used,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile;

    #[test]
    fn test_deploy_and_call() {
        let mut state = WorldState::new();

        let source = r#"
            PUSH 0
            PUSH 42
            STORE
            PUSH 0
            LOAD
            DUP
            LOG
            HALT
        "#;
        let bytecode = compile(source).unwrap();

        let address =
            ContractExecutor::deploy(&mut state, "alice", bytecode).unwrap();
        assert!(address.starts_with("0xc"));

        let result =
            ContractExecutor::call(&mut state, &address, &[]).unwrap();
        assert_eq!(result.logs, vec![42]);
        assert_eq!(result.stack_top, Some(42));
    }

    #[test]
    fn test_call_nonexistent_contract() {
        let mut state = WorldState::new();
        let result = ContractExecutor::call(&mut state, "0xnotfound", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_contract_storage_persists() {
        let mut state = WorldState::new();

        let source = r#"
            PUSH 0
            PUSH 100
            STORE
            HALT
        "#;
        let bytecode = compile(source).unwrap();
        let address =
            ContractExecutor::deploy(&mut state, "alice", bytecode).unwrap();

        ContractExecutor::call(&mut state, &address, &[]).unwrap();

        let contract = state.get_contract(&address).unwrap();
        assert_eq!(contract.storage.get(&0), Some(&100));
    }
}
