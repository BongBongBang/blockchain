use crate::blockchain::Blockchain;

const UTXO_PREFIX : &str = "utxo-";

pub struct UTXO<'a> {
    blockchain: &'a Blockchain
}