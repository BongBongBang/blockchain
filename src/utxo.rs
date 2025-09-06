use std::collections::HashMap;

use bincode::config::{self, standard};

use crate::{
    block::Block,
    blockchain::Blockchain,
    tx::{TxOutput, TxOutputs},
};

const UTXO_PREFIX: &str = "utxo-";

pub struct UTXOSet<'a> {
    pub blockchain: &'a mut Blockchain,
}

impl<'a> UTXOSet<'a> {

    pub fn new(blockchain: &'a mut Blockchain) -> Self {
        Self { blockchain }
    }
    /// 找到足够amount的可花费Txoutput
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`) - UTXO
    /// - `pub_key_hash` (`&[u8]`) - address
    /// - `amount` (`u128`) - amount
    ///
    /// # Returns
    ///
    /// - `(u128, HashMap<String, Vec<usize>>)` - (找到的总金额， <tx_id, out_idx>)
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: u128,
    ) -> Option<(u128, HashMap<String, Vec<usize>>)> {
        let mut accumulated = 0u128;
        let mut spendable_outputs = HashMap::<String, Vec<usize>>::default();

        // traverse utxo-*
        for result in self
            .blockchain
            .database
            .lock()
            .unwrap()
            .scan_prefix(UTXO_PREFIX)
        {
            let (tx_id, val) = result.unwrap();

            // decode to TxOutputs
            let (tx_outputs, _): (TxOutputs, usize) =
                bincode::decode_from_slice(&val, config::standard()).unwrap();

            // traverse TxOutput and accumulate TxOutput.amount
            for (idx, tx_output) in tx_outputs.outputs.iter().enumerate() {
                if accumulated < amount && tx_output.belongs_to(pub_key_hash) {
                    spendable_outputs
                        .entry(String::from_utf8_lossy(&tx_id).to_string())
                        .or_insert_with(Vec::new)
                        .push(idx);
                    accumulated += tx_output.amount;
                }
            }
        }

        if accumulated > amount {
            return Some((accumulated, spendable_outputs));
        }
        None
    }

    /// 找到adress的所有utxo
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`) - UTXO
    /// - `pub_key_hash` (`&[u8]`) - address
    ///
    /// # Returns
    ///
    /// - `Vec<TxOutput>` - UTXOs
    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Vec<TxOutput> {
        let mut utxos = vec![];
        for result in self
            .blockchain
            .database
            .lock()
            .unwrap()
            .scan_prefix(UTXO_PREFIX)
        {
            let (_, val) = result.unwrap();
            let (tx_outputs, _): (TxOutputs, usize) =
                bincode::decode_from_slice(&val, standard()).unwrap();
            for tx_output in tx_outputs.outputs {
                if tx_output.belongs_to(pub_key_hash) {
                    utxos.push(tx_output);
                }
            }
        }

        utxos
    }

    /// 统计含有未花费tx的总数
    /// 
    /// # Arguments
    /// 
    /// - `&self` (`undefined`) - UTXO
    /// 
    /// # Returns
    /// 
    /// - `u128` - 总数
    pub fn count_tx(&self) -> u128 {
        let mut count = 1u128;

        for _ in self
            .blockchain
            .database
            .lock()
            .unwrap()
            .scan_prefix(UTXO_PREFIX)
        {
            count += 1;
        }

        count
    }

    /// 用Block刷新现有的UTXO
    /// 
    /// # Arguments
    /// 
    /// - `&self` (`undefined`) - UTXO
    /// - `block` (`&'a Block`) - Block
    pub fn update(&self, block: &'a Block) {

        for tx in &block.transactions {

            let database = self.blockchain.database.lock().unwrap();
            
            // invalid referenced UTXO
            for input in &tx.inputs {
                let input_tx_id = hex::encode(&input.tx_id);
                let key = format!("{}{}", UTXO_PREFIX, input_tx_id);
                let val =  database.get(key.clone()).ok().flatten().unwrap();

                let (tx_outputs, _) : (TxOutputs, usize) = bincode::decode_from_slice(&val, config::standard()).unwrap();

                let mut utxos : Vec<TxOutput> = vec![];

                for (idx, tx_output) in tx_outputs.outputs.into_iter().enumerate() {
                    if idx != input.out_idx {
                        utxos.push(tx_output);
                    }
                }

                let unspent_tx_outputs = TxOutputs::new(utxos);
                let bytes = bincode::encode_to_vec(unspent_tx_outputs, config::standard()).unwrap();
                database.insert(key, bytes);
            }

            // save new TxOutput
            let tx_id = hex::encode(&tx.id);
            let tx_id_key = format!("{}{}", UTXO_PREFIX, tx_id);

            let new_tx_outputs = TxOutputs::new(tx.outputs.iter().map(|output| {
                return TxOutput {pub_key_hash: output.pub_key_hash.clone(), amount: output.amount};
            }).collect());
            let bytes = bincode::encode_to_vec(new_tx_outputs, config::standard()).unwrap();
            database.insert(tx_id_key, bytes);
        }
        todo!("这里的tx output idx 协变了");
    }

    /// 重建utxo set的数据库
    /// 
    /// # Arguments
    /// 
    /// - `&self` (`undefined`) - UTXOSet
    pub fn rebuild(&mut self) {
        self.clear_utxo();
        let utxos = self.blockchain.find_utxos();
        if utxos.is_empty() {
            return;
        }

        let database = self.blockchain.database.lock().unwrap();
        for (k, v) in utxos.into_iter() {
            let utxo_key = format!("{}{}", UTXO_PREFIX, k); 
            let utxo_bytes = bincode::encode_to_vec(v, config::standard()).unwrap();
            database.insert(utxo_key, utxo_bytes);
        }
    }

    /// 删除blockchain存储的所有utxo-记录
    fn clear_utxo(&self) {
        let database = self.blockchain.database.lock().unwrap();
        for result in database.scan_prefix(UTXO_PREFIX) {
            let (k, _) = result.unwrap();
            database.remove(k);
        }
    }
}
