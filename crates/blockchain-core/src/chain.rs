use crate::block::Block;
use crate::errors::{CoreError, CoreResult};
use crate::state::WorldState;
use crate::transaction::{Transaction, TransactionType};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: u32,
    mining_reward: u64,
    state: WorldState,
}

impl Blockchain {
    pub fn new(difficulty: u32, mining_reward: u64) -> Self {
        let genesis = Block::genesis();
        Self {
            chain: vec![genesis],
            pending_transactions: Vec::new(),
            difficulty,
            mining_reward,
            state: WorldState::new(),
        }
    }

    pub fn chain(&self) -> &[Block] {
        &self.chain
    }

    pub fn pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut WorldState {
        &mut self.state
    }

    pub fn difficulty(&self) -> u32 {
        self.difficulty
    }

    pub fn mining_reward(&self) -> u64 {
        self.mining_reward
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().expect("Chain must have at least genesis block")
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn height(&self) -> u64 {
        self.chain.len() as u64
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> CoreResult<()> {
        if tx.sender != "system" {
            tx.verify()?;

            if tx.tx_type == TransactionType::Transfer {
                let balance = self.state.get_balance(&tx.sender);
                if balance < tx.amount {
                    return Err(CoreError::InsufficientBalance {
                        account: tx.sender.clone(),
                        balance,
                        required: tx.amount,
                    });
                }
            }
        }

        self.pending_transactions.push(tx);
        Ok(())
    }

    pub fn mine_pending(&mut self, miner_address: &str) -> CoreResult<Block> {
        let reward_tx = Transaction::new_transfer(
            "system".into(),
            miner_address.into(),
            self.mining_reward,
        );

        let mut transactions = self.pending_transactions.drain(..).collect::<Vec<_>>();
        transactions.push(reward_tx);

        // Apply state transitions
        for tx in &transactions {
            match tx.tx_type {
                TransactionType::Transfer => {
                    if tx.sender == "system" {
                        self.state.credit(&tx.recipient, tx.amount);
                    } else {
                        if !self.state.transfer(&tx.sender, &tx.recipient, tx.amount) {
                            tracing::warn!(
                                "Skipping tx {}: insufficient balance",
                                tx.id
                            );
                        }
                    }
                }
                TransactionType::ContractDeploy | TransactionType::ContractCall => {
                    // Handled by VM integration layer
                }
            }
        }

        let previous_hash = self.latest_block().hash.clone();
        let index = self.height();
        let mut block = Block::new(index, previous_hash, transactions, self.difficulty);
        block.mine();

        self.chain.push(block.clone());
        tracing::info!("Block {} added to chain", index);

        Ok(block)
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if !current.is_valid() {
                tracing::error!("Block {} has invalid hash", current.header.index);
                return false;
            }

            if current.header.previous_hash != previous.hash {
                tracing::error!(
                    "Block {} has mismatched previous_hash",
                    current.header.index
                );
                return false;
            }
        }
        true
    }

    pub fn replace_chain(&mut self, new_chain: Vec<Block>) -> CoreResult<()> {
        if new_chain.len() <= self.chain.len() {
            return Err(CoreError::InvalidChain(
                "Incoming chain is not longer than current chain".into(),
            ));
        }

        let temp = Blockchain {
            chain: new_chain.clone(),
            pending_transactions: vec![],
            difficulty: self.difficulty,
            mining_reward: self.mining_reward,
            state: WorldState::new(),
        };

        if !temp.is_chain_valid() {
            return Err(CoreError::InvalidChain(
                "Incoming chain is not valid".into(),
            ));
        }

        tracing::info!(
            "Replacing chain: {} blocks -> {} blocks",
            self.chain.len(),
            new_chain.len()
        );

        // Rebuild state from new chain
        let mut state = WorldState::new();
        for block in &new_chain {
            for tx in &block.transactions {
                if tx.tx_type == TransactionType::Transfer {
                    if tx.sender == "system" {
                        state.credit(&tx.recipient, tx.amount);
                    } else {
                        state.transfer(&tx.sender, &tx.recipient, tx.amount);
                    }
                }
            }
        }

        self.chain = new_chain;
        self.state = state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    #[test]
    fn test_new_blockchain() {
        let bc = Blockchain::new(1, 50);
        assert_eq!(bc.chain().len(), 1);
        assert_eq!(bc.height(), 1);
    }

    #[test]
    fn test_mine_block() {
        let mut bc = Blockchain::new(1, 50);
        let block = bc.mine_pending("miner").unwrap();
        assert_eq!(block.header.index, 1);
        assert_eq!(bc.chain().len(), 2);
        assert!(bc.is_chain_valid());
    }

    #[test]
    fn test_mine_with_transactions() {
        let mut bc = Blockchain::new(1, 50);

        // First mine to give miner some coins
        bc.mine_pending("miner").unwrap();
        assert_eq!(bc.state().get_balance("miner"), 50);

        // Create and sign transfer
        let wallet = Wallet::new();
        bc.state_mut().credit(&wallet.address, 1000);

        let mut tx = Transaction::new_transfer(
            wallet.address.clone(),
            "bob".into(),
            100,
        );
        tx.sign(wallet.signing_key());
        bc.add_transaction(tx).unwrap();

        bc.mine_pending("miner").unwrap();
        assert_eq!(bc.state().get_balance("bob"), 100);
        assert_eq!(bc.state().get_balance(&wallet.address), 900);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut bc = Blockchain::new(1, 50);
        let wallet = Wallet::new();

        let mut tx = Transaction::new_transfer(
            wallet.address.clone(),
            "bob".into(),
            100,
        );
        tx.sign(wallet.signing_key());
        let result = bc.add_transaction(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_validation() {
        let mut bc = Blockchain::new(1, 50);
        bc.mine_pending("miner1").unwrap();
        bc.mine_pending("miner2").unwrap();
        assert!(bc.is_chain_valid());
    }
}
