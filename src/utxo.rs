use readb::Database;

use crate::{block::Block, blockchain::Blockchain, tx::TxOutput};

const UTXO_PREFIX : &str = "utxo-";

pub struct UTXO<'a> {
    blockchain: &'a Blockchain
}

impl<'a> UTXO<'a> {

    pub fn find_spendable_outputs(&self, pub_key_hash: &[u8], amount: u128) -> Vec<TxOutput> {
        vec![]
    }

    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Vec<TxOutput> {
        self.blockchain.database.lock().unwrap().get(key)
        vec![]
    }

    pub fn count_tx(&self) -> u128 {
        1
    }

    pub fn update(&self, block: &Block) {

    }

    /// 删除blockchain存储的所有utxo-记录
    pub fn clear_utxo(&self) {

    }
}