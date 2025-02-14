use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

#[derive(Debug)]
pub struct Wallet {
    pub private_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let private_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &private_key);

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
}