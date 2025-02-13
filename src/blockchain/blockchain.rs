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

    pub fn add_transaction(&mut self, sender: String, recipient: String, amount: f64, signature: String) {
        let transaction = Transaction::new(sender, recipient, amount, signature);
        self.ledger.add_transaction(transaction);
    }

    pub fn mine_block(&mut self, miner_address: &str) {
        self.ledger.mine_block(miner_address);
    }

    pub fn is_valid(&self) -> bool {
        self.ledger.is_valid_chain()
    }
}