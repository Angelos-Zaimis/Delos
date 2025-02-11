use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};

#[derive(Debug)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: f32,
    signature: String,
}

impl Transaction {
    fn new(sender: String, recipient: String, amount: f32, signature: String) -> Self {
        Self {
            sender,
            recipient,
            amount,
            signature
        }
    }

    fn sign(&mut self, secret_key: &SecretKey) {
        let context = Secp256k1::new();
        let message = self.create_message();
        let signature = context.sign(&Message::from_slice(&message).unwrap(), secret_key);
    }

    fn create_message(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.sender.as_bytes());
        bytes.extend(self.recipient.as_bytes());
        bytes.extend(self.amount.to_le_bytes());
        bytes
    }

    fn verify(&self, public_key: &PublicKey) -> bool {
        let context = Secp256k1::new();
        let message = self.create_message();
        let signature = Signature::from_str(&self.signature).unwrap();

        context.verify(&Message::from_slice(&message).unwrap(), &signature, public_key)
    }
}
