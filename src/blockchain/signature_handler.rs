#![allow(warnings)]

use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};
use sha2::{Digest, Sha256};
use rand::rngs::OsRng;
use secp256k1::ecdsa::Signature;

pub struct SignatureHandler;

impl SignatureHandler {
    pub fn generate_keys() -> (SecretKey, PublicKey) {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        (secret_key, public_key)
    }

    pub fn sign_message(secret_key: &SecretKey, message: &str) -> Signature {
        let secp = Secp256k1::new();
        let message_bytes = Sha256::digest(message.as_bytes());
        let message_hash = Message::from_slice(&message_bytes).expect("Message hash must be 32 bytes");
        secp.sign_ecdsa(&message_hash, secret_key)
    }

    pub fn verify_signature(public_key: &PublicKey, message: &str, signature: &str) -> bool {
        let secp = Secp256k1::new();
        let message_bytes = Sha256::digest(message.as_bytes());
        let message_hash = Message::from_slice(&message_bytes).expect("Message hash must be 32 bytes");
        let parsed_signature = signature.parse::<Signature>().expect("Invalid signature format");

        secp.verify_ecdsa(&message_hash, &parsed_signature, public_key).is_ok()
    }

}
