#[derive(Debug)]
pub struct Transaction {
    sender: String,
    recipient: String,
    amount: f64,
    signature: String,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64, signature: String) -> Self {
        Self {
            sender,
            recipient,
            amount,
            signature
        }
    }
    pub fn is_valid(&self) -> bool {
        self.amount > 0.0 && !self.sender.is_empty() && !self.recipient.is_empty()
    }
}
