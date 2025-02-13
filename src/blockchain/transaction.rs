use serde::Deserialize;
const BASE_FEE: f64 = 0.01;

#[derive(Debug, Deserialize, Clone)]
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
}
