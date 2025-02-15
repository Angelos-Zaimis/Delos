use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BASE_FEE: f64 = 0.01;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: String,
    pub fee: f64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64, signature: String) -> Self {
        Self {
            sender,
            recipient,
            amount,
            signature,
            fee: BASE_FEE
        }
    }
    pub fn is_valid(&self) -> bool {
        self.amount > 0.0 && self.fee >= 0.0 && !self.sender.is_empty() && !self.recipient.is_empty()
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}", self.sender, self.recipient, self.amount, self.fee));
        format!("{:x}", hasher.finalize())
    }
}
