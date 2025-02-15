use super::transaction::Transaction;
use super::wallet::Wallet;
use crate::blockchain::signature_handler::SignatureHandler;
use crate::blockchain::Block;
use rocksdb::{DBWithThreadMode, Options, SingleThreaded, DB};
use secp256k1::PublicKey;
use std::collections::HashMap;

const BLOCK_TARGET_TIME: u64 = 10;
const ADJUSTMENT_BLOCK_COUNT: usize = 5;
const MIN_TRANSACTIONS_FOR_BLOCK: usize = 2;
const BLOCK_REWARD: f64 = 1.0;

#[derive(Debug)]
pub struct Ledger {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub balances: HashMap<String, f64>,
    pub difficulty: u32,
    pub db: DB,
    pub total_fees_collected: f64,
    pub miner_wallet: Wallet,
}

impl Ledger {
    pub fn new() -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, "data/blockchain_db").expect("Failed to open RocksDB");
        let mut chain: Vec<Block> = Self::load_chain_from_db(&db);
        let mempool: Vec<Transaction> = Self::load_transactions_from_db(&db);
        let balances = Self::load_balances_from_db(&db);

        if chain.is_empty() {
            let genesis_block = Block::new(0, String::from("0"), String::from("Genesis block"), 2);
            chain.push(genesis_block);
        }

        let miner_wallet = Wallet::new();
        println!("Miner Wallet Address: {}", miner_wallet.address);

        Self {
            chain,
            mempool,
            balances,
            difficulty: 2,
            db,
            total_fees_collected: 0.0,
            miner_wallet,
        }
    }

    fn load_transactions_from_db(db: &DBWithThreadMode<SingleThreaded>) -> Vec<Transaction> {
        if let Ok(mempool_data) = db.get("mempool") {
            if let Some(data) = mempool_data {
                return serde_json::from_slice(&data).unwrap_or_else(|_| vec![]);
            }
        }
        vec![]
    }

    fn load_chain_from_db(db: &DBWithThreadMode<SingleThreaded>) -> Vec<Block> {
        if let Ok(blockchain_data) = db.get("blockchain") {
            if let Some(data) = blockchain_data {
                return serde_json::from_slice(&data).unwrap_or_else(|_| vec![]);
            }
        }
        vec![]
    }

    fn load_balances_from_db(db: &DBWithThreadMode<SingleThreaded>) -> HashMap<String, f64> {
        if let Ok(balance_data) = db.get("balances") {
            if let Some(data) = balance_data {
                return serde_json::from_slice(&data).unwrap_or_else(|_| HashMap::new());
            }
        }
        HashMap::new()
    }

    pub fn add_transaction(&mut self, transaction: Transaction, sender_public_key: &PublicKey) {
        let sender_balance = self.balances.get(&transaction.sender).unwrap_or(&0.0);
        let total_cost = transaction.amount + transaction.fee;

        if *sender_balance < total_cost {
            println!("Sender {} has insufficient funds!", transaction.sender);
            return;
        }

        if !SignatureHandler::verify_signature(sender_public_key, &transaction.hash(), &transaction.signature) {
            println!("Invalid transaction signature! Rejecting transaction.");
            return;
        }

        self.deduct_total_amount_from_sender(&transaction, total_cost);
        self.add_received_amount_to_recipient(&transaction);

        self.mempool.push(transaction);
        self.save_balances();
        self.save_mempool();
        println!("✅ Transaction added to mempool.");
    }


    pub fn mine_block(&mut self) {
        if self.mempool.len() < MIN_TRANSACTIONS_FOR_BLOCK {
            println!("⏳ Not enough transactions in the mempool to mine a block. Waiting...");
            return;
        }

        let selected_transactions = self.select_transactions_for_block();
        let total_fees: f64 = selected_transactions.iter().map(|tx| tx.fee).sum();
        self.total_fees_collected += total_fees;

        let data: String = format!("{:?}", selected_transactions);
        let target_prefix = "0".repeat(self.difficulty as usize);

        let miner_address = self.miner_wallet.address.clone();
        self.start_mining(target_prefix, &data, &miner_address, total_fees);
    }

    fn select_transactions_for_block(&mut self) -> Vec<Transaction> {
        self.mempool.sort_by(|a, b| b.fee.partial_cmp(&a.fee).unwrap());
        self.mempool.clone()
    }

    fn start_mining(&mut self, target_prefix: String, data: &str, miner_address: &str, total_fees: f64) {
        let last_block = self.chain.last().unwrap();
        let mut new_block = Block::new(last_block.index + 1, last_block.hash.clone(), data.to_string(), self.difficulty);
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
        println!(
            "Block Mined! Nonce: {}, Hash: {}, Difficulty: {}, Time: {} sec",
            new_block.nonce, new_block.hash, self.difficulty, mining_time
        );

        self.reward_the_miner(miner_address, total_fees);

        self.chain.push(new_block);
        self.save_chain();
        self.save_mempool();
        self.save_total_fees();
        self.mempool.clear();

        self.adjust_difficulty();
    }

    fn reward_the_miner(&mut self, miner_address: &str, total_fees: f64) {
        self.balances
            .entry(miner_address.to_string())
            .and_modify(|bal| *bal += BLOCK_REWARD + total_fees)
            .or_insert(BLOCK_REWARD + total_fees);
    }

    fn add_received_amount_to_recipient(&mut self, transaction: &Transaction) {
        self.balances
            .entry(transaction.recipient.clone()) // Fixed incorrect field name (was `recipient`)
            .and_modify(|bal| *bal += transaction.amount)
            .or_insert(transaction.amount);
    }

    fn deduct_total_amount_from_sender(&mut self, transaction: &Transaction, total_cost: f64) {
        self.balances
            .entry(transaction.sender.clone())
            .and_modify(|bal| *bal -= total_cost);
    }

    fn save_chain(&self) {
        let serialized_chain = serde_json::to_string(&self.chain).unwrap();
        self.db.put("blockchain", serialized_chain).expect("Failed to save blockchain");
    }

    fn save_mempool(&self) {
        let serialized_mempool = serde_json::to_string(&self.mempool).unwrap(); // Fixed incorrect field reference
        self.db.put("mempool", serialized_mempool).expect("Failed to save mempool");
    }

    fn save_balances(&self) {
        let serialized_balances = serde_json::to_string(&self.balances).unwrap();
        self.db.put("balances", serialized_balances).expect("Failed to save balances");
    }

    fn save_total_fees(&self) {
        let serialized_fees = serde_json::to_string(&self.total_fees_collected).unwrap();
        self.db.put("total_fees", serialized_fees).expect("Failed to save total fees");
    }

    fn adjust_difficulty(&mut self) {
        if self.chain.len() < ADJUSTMENT_BLOCK_COUNT {
            return;
        }

        let last_n_block: &[Block] = &self.chain[self.chain.len() - ADJUSTMENT_BLOCK_COUNT..];
        let first_block: &Block = &last_n_block[0];
        let last_block: &Block = last_n_block.last().unwrap();

        let actual_time = last_block.timestamp.parse::<u64>().unwrap_or(0)
            - first_block.timestamp.parse::<u64>().unwrap_or(0); // Fixed possible parse error
        let expected_time = BLOCK_TARGET_TIME * ADJUSTMENT_BLOCK_COUNT as u64;

        if actual_time < expected_time / 2 {
            self.difficulty += 1;
            println!("⚡ Mining too fast. Increasing difficulty to {}", self.difficulty);
        } else if actual_time > expected_time * 2 {
            self.difficulty = self.difficulty.saturating_sub(1);
            println!("🐢 Mining too slow. Decreasing difficulty to {}", self.difficulty);
        }
    }

    pub fn is_valid_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let calculated_hash = Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.previous_hash,
                &current.data,
                current.nonce,
            );

            if current.hash != calculated_hash {
                return false;
            }
        }
        true
    }
}
