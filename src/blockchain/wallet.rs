use secp256k1::{SecretKey, PublicKey};
use super::transaction::Transaction;
use sha2::{Sha256, Digest};
use crate::blockchain::signature_handler::SignatureHandler;

#[derive(Debug)]
pub struct Wallet {
    pub private_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let (private_key, public_key) = SignatureHandler::generate_keys();

        let mut hasher = Sha256::new();
        hasher.update(public_key.to_string().as_bytes());
        let hash_result = hasher.finalize();
        let address = format!("{:x}", hash_result);

        Self {
            private_key,
            public_key,
            address
        }
    }

    pub fn sign_transaction(&self,  transaction: &mut Transaction) {
        transaction.signature = SignatureHandler::sign_message(&self.private_key, &transaction.hash()).to_string();     }
}