use bincode::{Encode, config};
use k256::ecdsa::SigningKey;
use readb::{Database, DatabaseSettings, DefaultDatabase};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    block::Block,
    register_exit_callback,
    transaction::{Transaction, TxOutput},
};

const DB_PATH: &str = "./blocks";
const LATEST_HASH_KEY: &str = "lsh";

pub struct Blockchain {
    pub latest_hash: String,
    pub database: Arc<Mutex<readb::DefaultDatabase>>,
}

impl Encode for Blockchain {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.latest_hash.encode(encoder)?;
        Ok(())
    }
}

impl std::fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blockchain")
            .field("latest_hash", &self.latest_hash)
            .field("database", &"non-debuggable")
            .finish()
    }
}

impl Blockchain {
    // 本地数据库是否存在
    fn exists_db() -> bool {
        let db_path = PathBuf::from(DB_PATH);
        fs::exists(db_path).unwrap()
    }

    /// 从本地数据库文件初始化区块链
    pub fn continue_chain() -> Self {
        if !Blockchain::exists_db() {
            panic!("Blockchain DB doesn't exist, init one first!");
        }

        let db_path = PathBuf::from(DB_PATH);

        let db_client_mutex = Blockchain::init_db_client(db_path);
        let mut db_client = db_client_mutex.lock().unwrap();
        let lsh_value = db_client.get(LATEST_HASH_KEY);

        if let Some(lsh) = lsh_value.ok().flatten() {
            Blockchain {
                latest_hash: hex::encode(lsh),
                database: Arc::clone(&db_client_mutex),
            }
        } else {
            panic!("Blockchain latest hash doesn't exist, init blockchain first!");
        }
    }

    /*
    初始化数据库链接实例
     */
    fn init_db_client(db_path: PathBuf) -> Arc<Mutex<DefaultDatabase>> {
        // 初始化数据库
        let database_config = DatabaseSettings {
            path: Some(db_path),
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: true,
        };

        let db_client_mutex = Arc::new(Mutex::new(readb::DefaultDatabase::new(database_config)));
        let db_client_callback = Arc::clone(&db_client_mutex);

        register_exit_callback(move || {
            db_client_callback
                .lock()
                .unwrap()
                .persist()
                .expect("Failed to persist readb");
        });

        db_client_mutex
    }

    pub fn init(to: String) -> Self {
        // 判断本地数据库是否存在
        let db_path = PathBuf::from(DB_PATH);
        if db_path.exists() {
            panic!("Blockchain has already existed, just continue it!");
        }

        let db_client_mutex = Blockchain::init_db_client(db_path);
        let mut db_client = db_client_mutex.lock().unwrap();

        // init coinbase & genesis block
        let mut coinbase_tx = Transaction::coinbase_tx(to);
        coinbase_tx.set_id();
        let genesis_block = Block::genesis(coinbase_tx);
        let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard())
            .ok()
            .expect("Failed to init blockchain cause encoding genesis block error");

        let block_key = genesis_block.hash.clone();
        db_client
            .put(&block_key, &encoded_block)
            .expect("Failed to save genesis block");

        let hash_bytes = hex::decode(&block_key).expect(&format!(
            "Failed to hex::decode block_key {} to bytes",
            block_key
        ));
        db_client
            .put(LATEST_HASH_KEY, &hash_bytes)
            .expect("Failed to store latest hash");
        // db_client
        //     .persist()
        // .expect("Failed to store genesis block data !!!");
        return Blockchain {
            latest_hash: genesis_block.hash,
            database: Arc::clone(&db_client_mutex),
        };
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let mut database = self.database.lock().unwrap();
        let lsh_bytes = database
            .get(LATEST_HASH_KEY)
            .ok()
            .flatten()
            .expect("Failed to add_block cause there isn't  latest hash in DB");

        // create block
        let block = Block::create_block(hex::encode(lsh_bytes), transactions);

        // save lastest hash
        database
            .put(
                LATEST_HASH_KEY,
                &hex::decode(&block.hash).expect(&format!(
                    "Failed to decode hex hash {} to bytes",
                    &block.hash
                )),
            )
            .expect("Failed to save added block");

        // save new block to DB
        let encoded_block = bincode::encode_to_vec(&block, config::standard())
            .expect("Failed to encode new added block");
        database
            .put(&block.hash, &encoded_block)
            .expect("Failed to save new added block");

        // database
        //     .persist()
        //     .expect("Failed to store added block data !!!");
    }

    pub fn iterator(&mut self) -> Iterator {
        let mut database = self.database.lock().unwrap();
        if let Some(lsh) = database.get(LATEST_HASH_KEY).ok().flatten() {
            return Iterator {
                database: Arc::clone(&self.database),
                current_hash: hex::encode(lsh),
            };
        }
        Iterator {
            database: Arc::clone(&self.database),
            current_hash: String::default(),
        }
    }

    // 寻找所有未花费的transaction
    pub fn find_unspent_tx(&mut self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        let mut iter = self.iterator();

        let mut unspent_tx = Vec::default();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        // block layer
        while let Some(block) = iter.next() {
            // transaction layer
            'transaction: for tx in block.transactions {
                let tx_id = hex::encode(&tx.id);

                // 如果不是coinbase，记录所有的input引用
                if !tx.is_coinbase() {
                    for input in &tx.inputs {
                        if input.spent_by(pub_key_hash) {
                            let ref_tx_id = hex::encode(&input.tx_id);
                            if let Some(out_idxes) = spent_txos.get_mut(&ref_tx_id) {
                                out_idxes.push(input.out_idx);
                            } else {
                                let out_idxes = vec![input.out_idx];
                                spent_txos.insert(ref_tx_id, out_idxes);
                            }
                        }
                    }
                }

                // output layer
                for (out_idx, output) in tx.outputs.iter().enumerate() {
                    if spent_txos.contains_key(&tx_id) {
                        let out_idxes = spent_txos.get(&tx_id).unwrap();
                        // 如果这个out_idx的output已经被花费掉了
                        if out_idxes.contains(&out_idx) {
                            continue;
                        }
                    }

                    if output.belongs_to(pub_key_hash) {
                        unspent_tx.push(tx);
                        continue 'transaction;
                    }
                }
            }
        }

        unspent_tx
    }

    /*
     * 寻找某地址所有未支付Output
     */
    pub fn find_utxo(&mut self, pub_key_hash: &[u8]) -> Vec<TxOutput> {
        let unspent_txes = self.find_unspent_tx(pub_key_hash);

        if unspent_txes.len() == 0 {
            return vec![];
        }

        let mut utxos = Vec::default();
        for tx in unspent_txes {
            for output in tx.outputs {
                // todo: fix multiple outputs belongs to same address in a Transaction
                if output.belongs_to(pub_key_hash) {
                    utxos.push(output);
                }
            }
        }

        utxos
    }

    /*
     * 寻找某地址可以用来支付某金额的Outputs
     */
    pub fn find_spendable_outputs(
        &mut self,
        amount: u128,
        pub_key_hash: &Vec<u8>,
    ) -> Option<(u128, HashMap<String, Vec<usize>>)> {
        let unspent_txes = self.find_unspent_tx(pub_key_hash);

        let mut accumulated = 0u128;
        let mut spendable_outputs: HashMap<String, Vec<usize>> = HashMap::new();

        'tx: for tx in &unspent_txes {
            for (out_idx, output) in tx.outputs.iter().enumerate() {
                if output.belongs_to(pub_key_hash) && accumulated < amount {
                    let tx_id = hex::encode(&tx.id);

                    accumulated += output.amount;

                    if let Some(out_idxes) = spendable_outputs.get_mut(&tx_id) {
                        out_idxes.push(out_idx);
                    } else {
                        let out_idxes = vec![out_idx];
                        spendable_outputs.insert(tx_id, out_idxes);
                    }

                    if accumulated > amount {
                        break 'tx;
                    }
                }
            }
        }

        if accumulated > amount {
            Some((accumulated, spendable_outputs))
        } else {
            None
        }
    }

    /// 寻找目标Transaction
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`undefined`)
    /// - `tx_id` (`&Vec<u8>`) - tx_id
    ///
    /// # Returns
    ///
    /// - `Transaction` - 事务.
    pub fn find_transaction(&mut self, tx_id: &Vec<u8>) -> Transaction {
        let mut iter = self.iterator();
        while let Some(block) = iter.next() {
            for tx in block.transactions {
                if &tx.id == tx_id {
                    return tx;
                }
            }
        }

        panic!("Transaction: {} doesn't exist", hex::encode(tx_id));
    }

    /// 给Tx签名
    ///
    /// # Arguments
    ///
    /// - `tx` (`&Transaction`) - 待签名的Tx
    /// - `priv_key` (`&SigningKey`) - 签名的私钥
    /// # Returns
    ///
    pub fn sign_transaction(&mut self, tx_to_sign: &mut Transaction, priv_key: &mut SigningKey) {
        let mut prev_txs: HashMap<String, Transaction> = HashMap::new();
        for input in &tx_to_sign.inputs {
            let tx = self.find_transaction(&input.tx_id);
            prev_txs.insert(hex::encode(&input.tx_id), tx);
        }

        tx_to_sign.sign(priv_key, prev_txs);
    }

    /// 校验某个Tx
    /// 
    /// # Returns
    /// 
    /// - `bool` - 是否通过校验
    /// 
    pub fn verify_transaction(&mut self, tx_to_verify: &Transaction) -> bool {

        let mut prev_txs : HashMap<String, Transaction> = HashMap::default();

        for input in &tx_to_verify.inputs {
            let tx = self.find_transaction(&input.tx_id);
            prev_txs.insert(hex::encode(&input.tx_id), tx);
        }

        tx_to_verify.verity(prev_txs)
    }

}

pub struct Iterator {
    pub database: Arc<Mutex<readb::DefaultDatabase>>,
    pub current_hash: String,
}

impl Iterator {
    pub fn next(&mut self) -> Option<Block> {
        if &self.current_hash == "" {
            return None;
        }
        let mut database = self.database.lock().unwrap();

        let encoded_data = database
            .get(&self.current_hash)
            .ok()
            .flatten()
            .expect(&format!("Hash {} has no data in DB!", &self.current_hash));

        let (block, _len): (Block, usize) =
            bincode::decode_from_slice(&encoded_data, config::standard()).expect(&format!(
                "Fail to decode data from DB, hash {}",
                &self.current_hash
            ));

        let prev_hash = block.prev_hash.clone();
        self.current_hash = prev_hash;
        Some(block)
    }
}
