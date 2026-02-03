use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Wallet {
    signing_key: SigningKey,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let address = Self::derive_address(&signing_key);
        Self {
            signing_key,
            address,
        }
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    pub fn info(&self) -> WalletInfo {
        WalletInfo {
            address: self.address.clone(),
            public_key: self.public_key_hex(),
        }
    }

    fn derive_address(signing_key: &SigningKey) -> String {
        let public_key = signing_key.verifying_key();
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("0x{}", &hash[..40])
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::Transaction;

    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42);
    }

    #[test]
    fn test_unique_addresses() {
        let w1 = Wallet::new();
        let w2 = Wallet::new();
        assert_ne!(w1.address, w2.address);
    }

    #[test]
    fn test_sign_transaction_with_wallet() {
        let wallet = Wallet::new();
        let mut tx = Transaction::new_transfer(
            wallet.address.clone(),
            "0xrecipient".into(),
            50,
        );
        tx.sign(wallet.signing_key());
        assert!(tx.verify().unwrap());
    }
}
