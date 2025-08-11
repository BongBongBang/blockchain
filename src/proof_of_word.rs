#![allow(dead_code)]
use ethereum_types::U256;
use sha2::{Digest, Sha256};
use crate::block::Block;

// max difficulty is 255
const DIFFICULTY : u8 = 1;

#[derive(Debug)]
pub struct ProofOfWork<'a> {
    pub target: U256,
    pub block: &'a Block
}

impl<'a> ProofOfWork<'a> {

    pub fn new(block: &'a Block) -> Self {
        let mut target = U256::from(1);
        target = target << (256u32 - DIFFICULTY as u32);
        ProofOfWork { target, block }
    }

    pub  fn init_data(&self, nonce: &u32) -> Vec<u8> {
        // prev_hash data nonce difficulty
        let mut hash : Vec<u8> = Vec::new();
        hash.extend(&self.block.prev_hash);
        hash.extend(&self.block.data);
        hash.extend(nonce.to_be_bytes());
        hash.extend(DIFFICULTY.to_be_bytes());

        hash
    }

    pub fn run(&self) -> Option<(u32, Vec<u8>)> {

        let mut nonce = 1u32;

        let hash = loop {

            if nonce == u32::MAX {
                break None;
            }

            let data_to_hash = &self.init_data(&nonce);
            let mut hasher = Sha256::new();
            hasher.update(data_to_hash);
            let hash = hasher.finalize().to_vec();
            if U256::from_big_endian(&hash) < self.target {
                break Some(hash);
            }

            nonce += 1;
        };

        match hash {
            Some(hash) => Some((nonce, hash)),
            None => None
        }
    }

    pub fn validate(&self, block: &Block) -> bool {

        let nonce = block.nonce;
        let data_to_hash = &self.init_data(&nonce);
        let mut hasher = sha2::Sha256::new();
        hasher.update(data_to_hash);
        let hash = hasher.finalize().to_vec();

        U256::from_big_endian(&hash) < self.target
    }
}