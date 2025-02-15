use secp256k1::PublicKey;
use crate::blockchain::ledger::Ledger;
use crate::blockchain::transaction::Transaction;

#[derive(Debug)]
pub struct Blockchain {
    pub ledger: Ledger
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            ledger: Ledger::new()
        }
    }

    pub fn add_transaction(&mut self, sender: String, recipient: String, amount: f64, signature: String, public_key: PublicKey) {
        let transaction = Transaction::new(sender, recipient, amount, signature);
        self.ledger.add_transaction(transaction, &public_key);
    }

    pub fn mine_block(&mut self) {
        self.ledger.mine_block();
    }

    pub fn is_valid(&self) -> bool {
        self.ledger.is_valid_chain()
    }
}