use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::merkle::MerkleTree;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        index: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        let merkle_root = MerkleTree::root(&transactions);
        let header = BlockHeader {
            index,
            timestamp: Utc::now(),
            previous_hash,
            merkle_root,
            nonce: 0,
            difficulty,
        };
        let hash = Self::calculate_hash(&header);
        Self {
            header,
            hash,
            transactions,
        }
    }

    pub fn genesis() -> Self {
        let header = BlockHeader {
            index: 0,
            timestamp: Utc::now(),
            previous_hash: "0".repeat(64),
            merkle_root: MerkleTree::root(&[]),
            nonce: 0,
            difficulty: 1,
        };
        let hash = Self::calculate_hash(&header);
        Self {
            header,
            hash,
            transactions: vec![],
        }
    }

    pub fn mine(&mut self) {
        let target = "0".repeat(self.header.difficulty as usize);
        loop {
            self.hash = Self::calculate_hash(&self.header);
            if self.hash.starts_with(&target) {
                tracing::info!(
                    "Block {} mined: {} (nonce: {})",
                    self.header.index,
                    &self.hash[..16],
                    self.header.nonce
                );
                break;
            }
            self.header.nonce += 1;
        }
    }

    pub fn calculate_hash(header: &BlockHeader) -> String {
        let data = format!(
            "{}{}{}{}{}{}",
            header.index,
            header.timestamp,
            header.previous_hash,
            header.merkle_root,
            header.nonce,
            header.difficulty
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn is_valid(&self) -> bool {
        let target = "0".repeat(self.header.difficulty as usize);
        let calculated = Self::calculate_hash(&self.header);
        calculated == self.hash && self.hash.starts_with(&target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.index, 0);
        assert!(genesis.transactions.is_empty());
        assert_eq!(genesis.header.previous_hash, "0".repeat(64));
    }

    #[test]
    fn test_mine_block() {
        let txs = vec![Transaction::new_transfer("a".into(), "b".into(), 10)];
        let mut block = Block::new(1, "0".repeat(64), txs, 1);
        block.mine();
        assert!(block.hash.starts_with("0"));
        assert!(block.is_valid());
    }

    #[test]
    fn test_block_validity() {
        let mut block = Block::new(1, "0".repeat(64), vec![], 1);
        block.mine();
        assert!(block.is_valid());

        block.header.nonce += 1;
        assert!(!block.is_valid());
    }
}
