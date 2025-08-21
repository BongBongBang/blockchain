use core::panic;
use bincode::{Decode, Encode};

use crate::{proof_of_work::ProofOfWork, transaction::Transaction};

#[derive(Debug, Encode, Decode)]
pub struct Block {
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub hash: String,
    pub nonce: u32,
}

impl Block {
    pub fn genesis(coinbase: Transaction) -> Self {
        Block::create_block(String::default(), vec![coinbase])
    }

    pub fn create_block(prev_hash: String, transactions: Vec<Transaction>) -> Self {
        // todo: return the solid block, refresh nonce if has traversed it all.
        let mut new_block = Block {
            prev_hash,
            transactions,
            hash: String::default(),
            nonce: u32::default(),
        };

        // do the proof of work
        let proof_of_work = ProofOfWork::new(&new_block);
        let cal_result = proof_of_work.run();

        // match fail or success
        match cal_result {
            Some((nonce, hash)) => {
                new_block.nonce = nonce;
                new_block.hash = hash;
                return new_block;
            }
            None => {
                let mut err_msg = String::with_capacity(128);
                err_msg.push_str("Failed to calcute proof of work for block, prev_hash: ");
                let prev_hash = hex::encode(new_block.prev_hash);
                err_msg.push_str(&prev_hash);
                panic!("{}", err_msg);
            }
        }
    }

    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut result = Vec::default();
        for tx in &self.transactions {
            result.extend(&tx.id);
        }

        result
    }
}
