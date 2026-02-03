use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

pub struct MerkleTree;

impl MerkleTree {
    pub fn root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return Self::hash_pair("", "");
        }

        let mut hashes: Vec<String> = transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                let last = hashes.last().unwrap().clone();
                hashes.push(last);
            }

            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
            }
            hashes = next_level;
        }

        hashes.into_iter().next().unwrap()
    }

    fn hash_pair(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", left, right).as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    #[test]
    fn test_empty_merkle_root() {
        let root = MerkleTree::root(&[]);
        assert!(!root.is_empty());
    }

    #[test]
    fn test_single_transaction_merkle() {
        let tx = Transaction::new_transfer("a".into(), "b".into(), 10);
        let root = MerkleTree::root(&[tx]);
        assert!(!root.is_empty());
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_merkle_deterministic() {
        let txs = vec![
            Transaction::new_transfer("a".into(), "b".into(), 10),
            Transaction::new_transfer("c".into(), "d".into(), 20),
        ];
        let r1 = MerkleTree::root(&txs);
        let r2 = MerkleTree::root(&txs);
        assert_eq!(r1, r2);
    }
}
