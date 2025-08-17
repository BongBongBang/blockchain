use core::panic;

use base64::{engine::general_purpose, Engine};
use bincode::{Decode, Encode};
use serde_derive::{Deserialize, Serialize};

use crate::proof_of_word::{ProofOfWork};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Block {
    pub prev_hash: String,
    pub data: Vec<u8>,
    pub hash: String,
    pub nonce: u32
}

impl Block {

    pub fn genesis() -> Self {
        Block::create_block(String::default(), String::from("Genesis Block"))
    }

    pub fn new(prev_hash: String, data: String) -> Self {
        Block::create_block(prev_hash, data)
    }

    pub fn create_block(prev_hash: String, data: String) -> Self {
        // todo: return the solid block, refresh nonce if has traversed it all.
        let mut new_block = Block {
            prev_hash,
            data: data.into_bytes(),
            hash: String::default(),
            nonce: u32::default()
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
            },
            None => {
                let mut err_msg = String::with_capacity(128);
                err_msg.push_str("Failed to calcute proof of work for block, prev_hash: ");
                let prev_hash_str = general_purpose::STANDARD.encode(new_block.prev_hash);
                err_msg.push_str(&prev_hash_str);
                panic!("{}", err_msg);
            }
        }
    }
}