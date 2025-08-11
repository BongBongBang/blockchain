use bincode::{Decode, Encode};
use serde_derive::{Deserialize, Serialize};

use crate::proof_of_word::{ProofOfWork};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Block {
    pub prev_hash: Vec<u8>,
    pub data: Vec<u8>,
    pub hash: Vec<u8>,
    pub nonce: u32
}

impl Block {

    pub fn genesis() -> Self {
        Block::create_block(Vec::default(), String::from("Genesis Block")).unwrap()
    }

    pub fn new(prev_hash: Vec<u8>, data: String) -> Self {
        Block::create_block(prev_hash, data).unwrap()
    }

    fn create_block(prev_hash: Vec<u8>, data: String) -> Result<Self, String> {
        // todo: return the solid block, refresh nonce if has traversed it all.
        let mut new_block = Block {
            prev_hash,
            data: data.into_bytes(),
            hash: Vec::default(),
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
                return Ok(new_block);
            },
            None => {
                let mut err_msg = String::with_capacity(128);
                err_msg.push_str("Failed to calcute proof of work for block, prev_hash: ");
                err_msg.push_str(&String::from_utf8(new_block.prev_hash).unwrap());
                return Err(err_msg);
            }
        }
    }
}