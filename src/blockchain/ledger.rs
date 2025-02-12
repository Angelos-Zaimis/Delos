use super::transaction::Transaction;
use crate::blockchain::Block;
#[derive(Debug)]
pub struct Ledger {
    pub chain: Vec<Block>,
    pub transactions: Vec<Transaction>,
}

impl Ledger {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), String::from("Genesis block"));
        Self {
            chain: vec![genesis_block],
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        if transaction.is_valid() {
            self.transactions.push(transaction);
        } else {
            println!("Invalid transaction: {:?}", transaction);
        }
    }

    pub fn mine_block(&mut self) {
        let last_block: &Block = self.chain.last().unwrap();
        let data: String = format!("{:?}", self.transactions);
        let new_block = Block::new(last_block.index + 1, last_block.hash.clone(), data);

        self.chain.push(new_block);
        self.transactions.clear();
    }

    pub fn is_valid_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                false;
            }

            let calculated_hash = Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.previous_hash,
                &current.data,
                current.nonce
            );

            if current.hash != calculated_hash {
                false;
            }
        }
        true
    }
}