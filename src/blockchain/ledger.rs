use rocksdb::{Options, DB};
use super::transaction::Transaction;
use crate::blockchain::Block;

const BLOCK_TARGET_TIME: u64 = 10;
const ADJUSTMENT_BLOCK_COUNT: usize = 5;
const MIN_TRANSACTIONS_FOR_BLOCK: usize = 2;

#[derive(Debug)]
pub struct Ledger {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub difficulty: u32,
    pub db: DB,
    pub total_fees_collected: f64,
}

impl Ledger {
    pub fn new() -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, "data/blockchain_db").expect("Failed to open RocksDB");

        let mut chain = Vec::new();
        let mut mempool = Vec::new();

        // Load existing blockchain from RocksDB
        if let Ok(blockchain_data) = db.get("blockchain") {
            if let Some(data) = blockchain_data {
                chain = serde_json::from_slice(&data).unwrap_or_else(|_| vec![]);
            }
        }

        // Load existing mempool from RocksDB
        if let Ok(mempool_data) = db.get("mempool") {
            if let Some(data) = mempool_data {
                mempool = serde_json::from_slice(&data).unwrap_or_else(|_| vec![]);
            }
        }

        if chain.is_empty() {
            // If the database is empty, create a genesis block
            let genesis_block = Block::new(0, String::from("0"), String::from("Genesis block"), 2);
            chain.push(genesis_block);
        }

        Self {
            chain,
            mempool,
            difficulty: 2,
            db,
            total_fees_collected: 0.0,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        if transaction.is_valid() {
            self.mempool.push(transaction);
            self.save_mempool();
        } else {
            println!("Invalid transaction: {:?}", transaction);
        }
    }

    pub fn mine_block(&mut self) {
        if self.mempool.len() < MIN_TRANSACTIONS_FOR_BLOCK {
            println!("Not enough transactions in the mempool to mine a block. Waiting...");
            return;
        }

        let selected_transactions = self.select_transactions_for_block();
        let last_block = self.chain.last().unwrap();
        let total_fees: f64 = selected_transactions.iter().map(|tx| tx.fee).sum();
        self.total_fees_collected += total_fees;

        let data: String = format!("{:?}", selected_transactions);
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
        self.save_chain();
        self.save_mempool();
        self.save_total_fees();

        self.mempool.clear();

        self.adjust_difficulty();

    }

    fn save_total_fees(&self) {
        let serialized_fees = serde_json::to_string(&self.total_fees_collected).unwrap();
        self.db.put("total_fees", serialized_fees).expect("Failed to save total fees");
    }

    fn save_chain(&self) {
        let serialized_chain = serde_json::to_string(&self.chain).unwrap();
        self.db.put("blockchain", serialized_chain).unwrap()
    }

    fn save_mempool(&self) {
        let serialized_mempool = serde_json::to_string(&self.chain).unwrap();
        self.db.put("mempool", serialized_mempool).expect("Failed to save mempool");
    }

    fn select_transactions_for_block(&mut self) -> Vec<Transaction> {
        self.mempool.sort_by(|a,b| b.fee.partial_cmp(&a.fee).unwrap());

        let selected = self.mempool.iter().take(MIN_TRANSACTIONS_FOR_BLOCK).cloned().collect();
        selected
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