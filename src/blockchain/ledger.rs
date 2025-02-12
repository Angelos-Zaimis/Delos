use super::transaction::Transaction;
use crate::blockchain::Block;

const BLOCK_TARGET_TIME: u64 = 10;
const ADJUSTMENT_BLOCK_COUNT: usize = 5;

#[derive(Debug)]
pub struct Ledger {
    pub chain: Vec<Block>,
    pub transactions: Vec<Transaction>,
    pub difficulty: u32,
}

impl Ledger {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), String::from("Genesis block"), 2);
        Self {
            chain: vec![genesis_block],
            transactions: Vec::new(),
            difficulty: 2
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
        let mut new_block = Block::new(last_block.index + 1, last_block.hash.clone(), data, self.difficulty);

        let target_prefix = "0".repeat(self.difficulty as usize);
        let start_time = std::time::Instant::now();

        while !new_block.hash.starts_with(&target_prefix) {
            new_block.nonce += 1;
            new_block.hash = Block::calculate_hash(
                new_block.index,
                &new_block.timestamp,
                &new_block.previous_hash,
                &new_block.data,
                new_block.nonce,
            );
        }

        let mining_time = start_time.elapsed().as_secs();

        println!("Block Mined! Nonce: {}, Hash: {}, Difficulty: {}, Time: {} sec", new_block.nonce, new_block.hash, self.difficulty, mining_time);

        self.chain.push(new_block);
        self.transactions.clear();

        self.adjust_difficulty();
    }

    fn adjust_difficulty(&mut self) {
        if self.chain.len() < ADJUSTMENT_BLOCK_COUNT {
            return;
        }

        let last_n_block: &[Block] = &self.chain[self.chain.len() - ADJUSTMENT_BLOCK_COUNT..];
        let first_block: &Block = &last_n_block[0];
        let last_block: &Block = last_n_block.last().unwrap();

        let actual_time: u64 =  &last_block.timestamp.parse::<u64>().unwrap() - &first_block.timestamp.parse::<u64>().unwrap();
        let expected_time: u64 = BLOCK_TARGET_TIME * ADJUSTMENT_BLOCK_COUNT as u64;

        if actual_time < expected_time / 2 {
            self.difficulty += 1;
            println!("Mining too fast.Increasing difficulty to {}", self.difficulty);
        } else if actual_time > expected_time * 2 {
            self.difficulty = self.difficulty.saturating_sub(1);
            print!("Mining too slow.Decreasing difficulty to {}", self.difficulty);
        }
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