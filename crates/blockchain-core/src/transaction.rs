use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    ContractDeploy,
    ContractCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub data: Vec<u8>,
    pub tx_type: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(
        sender: String,
        recipient: String,
        amount: u64,
        data: Vec<u8>,
        tx_type: TransactionType,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            recipient,
            amount,
            data,
            tx_type,
            timestamp: Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    pub fn new_transfer(sender: String, recipient: String, amount: u64) -> Self {
        Self::new(sender, recipient, amount, vec![], TransactionType::Transfer)
    }

    pub fn new_contract_deploy(sender: String, bytecode: Vec<u8>) -> Self {
        Self::new(
            sender,
            String::new(),
            0,
            bytecode,
            TransactionType::ContractDeploy,
        )
    }

    pub fn new_contract_call(sender: String, contract_address: String, call_data: Vec<u8>) -> Self {
        Self::new(
            sender,
            contract_address,
            0,
            call_data,
            TransactionType::ContractCall,
        )
    }

    pub fn hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}{:?}{:?}",
            self.id, self.sender, self.recipient, self.amount, self.timestamp, self.data, self.tx_type
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn signable_bytes(&self) -> Vec<u8> {
        let data = format!(
            "{}{}{}{}{}",
            self.id, self.sender, self.recipient, self.amount, self.timestamp
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.finalize().to_vec()
    }

    pub fn sign(&mut self, signing_key: &SigningKey) {
        let message = self.signable_bytes();
        let signature = signing_key.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        self.public_key = Some(signing_key.verifying_key().to_bytes().to_vec());
    }

    pub fn verify(&self) -> CoreResult<bool> {
        if self.sender == "system" {
            return Ok(true);
        }

        let signature_bytes = self
            .signature
            .as_ref()
            .ok_or_else(|| CoreError::InvalidSignature("Missing signature".into()))?;

        let public_key_bytes = self
            .public_key
            .as_ref()
            .ok_or_else(|| CoreError::InvalidSignature("Missing public key".into()))?;

        let sig_array: [u8; 64] = signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| CoreError::InvalidSignature("Invalid signature length".into()))?;
        let signature = Signature::from_bytes(&sig_array);

        let pk_array: [u8; 32] = public_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| CoreError::InvalidSignature("Invalid public key length".into()))?;
        let verifying_key = VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| CoreError::InvalidSignature(e.to_string()))?;

        let message = self.signable_bytes();
        verifying_key
            .verify(&message, &signature)
            .map_err(|e| CoreError::InvalidSignature(e.to_string()))?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transfer() {
        let tx = Transaction::new_transfer("alice".into(), "bob".into(), 100);
        assert_eq!(tx.sender, "alice");
        assert_eq!(tx.recipient, "bob");
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.tx_type, TransactionType::Transfer);
    }

    #[test]
    fn test_transaction_hash_deterministic() {
        let tx = Transaction {
            id: "test-id".into(),
            sender: "alice".into(),
            recipient: "bob".into(),
            amount: 100,
            data: vec![],
            tx_type: TransactionType::Transfer,
            timestamp: Utc::now(),
            signature: None,
            public_key: None,
        };
        let h1 = tx.hash();
        let h2 = tx.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let mut tx = Transaction::new_transfer("alice".into(), "bob".into(), 50);
        tx.sign(&signing_key);

        assert!(tx.signature.is_some());
        assert!(tx.verify().unwrap());
    }

    #[test]
    fn test_system_transaction_no_signature() {
        let tx = Transaction::new_transfer("system".into(), "miner".into(), 50);
        assert!(tx.verify().unwrap());
    }
}
