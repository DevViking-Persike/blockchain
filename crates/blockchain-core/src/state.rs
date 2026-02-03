use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub balance: u64,
    pub nonce: u64,
}

impl AccountState {
    pub fn new(balance: u64) -> Self {
        Self { balance, nonce: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractState {
    pub bytecode: Vec<u8>,
    pub storage: HashMap<u64, i64>,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    accounts: HashMap<String, AccountState>,
    contracts: HashMap<String, ContractState>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
        }
    }

    pub fn get_account(&self, address: &str) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts
            .get(address)
            .map(|a| a.balance)
            .unwrap_or(0)
    }

    pub fn get_or_create_account(&mut self, address: &str) -> &mut AccountState {
        self.accounts
            .entry(address.to_string())
            .or_insert_with(|| AccountState::new(0))
    }

    pub fn credit(&mut self, address: &str, amount: u64) {
        let account = self.get_or_create_account(address);
        account.balance += amount;
    }

    pub fn debit(&mut self, address: &str, amount: u64) -> bool {
        let account = self.get_or_create_account(address);
        if account.balance >= amount {
            account.balance -= amount;
            account.nonce += 1;
            true
        } else {
            false
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> bool {
        let from_balance = self.get_balance(from);
        if from_balance < amount {
            return false;
        }
        self.debit(from, amount);
        self.credit(to, amount);
        true
    }

    pub fn deploy_contract(
        &mut self,
        address: String,
        bytecode: Vec<u8>,
        owner: String,
    ) {
        self.contracts.insert(
            address,
            ContractState {
                bytecode,
                storage: HashMap::new(),
                owner,
            },
        );
    }

    pub fn get_contract(&self, address: &str) -> Option<&ContractState> {
        self.contracts.get(address)
    }

    pub fn get_contract_mut(&mut self, address: &str) -> Option<&mut ContractState> {
        self.contracts.get_mut(address)
    }

    pub fn accounts(&self) -> &HashMap<String, AccountState> {
        &self.accounts
    }

    pub fn contracts(&self) -> &HashMap<String, ContractState> {
        &self.contracts
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_and_balance() {
        let mut state = WorldState::new();
        state.credit("alice", 1000);
        assert_eq!(state.get_balance("alice"), 1000);
    }

    #[test]
    fn test_transfer() {
        let mut state = WorldState::new();
        state.credit("alice", 1000);
        assert!(state.transfer("alice", "bob", 300));
        assert_eq!(state.get_balance("alice"), 700);
        assert_eq!(state.get_balance("bob"), 300);
    }

    #[test]
    fn test_transfer_insufficient() {
        let mut state = WorldState::new();
        state.credit("alice", 100);
        assert!(!state.transfer("alice", "bob", 200));
        assert_eq!(state.get_balance("alice"), 100);
    }

    #[test]
    fn test_deploy_contract() {
        let mut state = WorldState::new();
        state.deploy_contract("contract1".into(), vec![1, 2, 3], "alice".into());
        let contract = state.get_contract("contract1").unwrap();
        assert_eq!(contract.bytecode, vec![1, 2, 3]);
        assert_eq!(contract.owner, "alice");
    }
}
