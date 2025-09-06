use std::ops::Sub;

use base58::FromBase58;
use bincode::{Decode, Encode};

use crate::wallet::Wallet;


#[derive(Debug, Encode, Decode, Clone)]
pub struct TxInput {
    pub tx_id: Vec<u8>,
    pub out_idx: usize,
    pub sig: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct TxOutput {
    pub amount: u128,
    pub pub_key_hash: Vec<u8>,
}

#[derive(Debug, Decode, Encode)]
pub struct TxOutputs {
    pub outputs: Vec<TxOutput>
}

impl TxOutputs {
    pub fn new(outputs: Vec<TxOutput>) -> Self {
        Self { outputs }
    }

    pub fn push(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }
}

impl Default for TxOutputs {
    fn default() -> Self {
        Self { outputs: Default::default() }
    }
}


impl TxOutput {
    pub fn new(amount: u128, address: String) -> Self {
        let bytes = address
            .from_base58()
            .expect(&format!("Failed to decode address[{}] base58", address));

        let pub_key_hash = &bytes[1..bytes.len().sub(4)];
        TxOutput {
            amount,
            pub_key_hash: pub_key_hash.to_vec(),
        }
    }

    // 判断当前output是否归属pub_key
    pub fn belongs_to(&self, pub_key_hash: &[u8]) -> bool {
        &self.pub_key_hash == pub_key_hash
    }
}

impl TxInput {
    pub fn new(tx_id: Vec<u8>, out_idx: usize, pub_key: Vec<u8>, sig: Vec<u8>) -> Self {
        TxInput {
            tx_id,
            out_idx,
            sig,
            pub_key,
        }
    }

    // 判断当前input是否是pub_key的来源
    #[inline]
    pub fn spent_by(&self, pub_key_hash: &[u8]) -> bool {
        &Wallet::hash_pub_key(&self.pub_key) == pub_key_hash
    }
}

