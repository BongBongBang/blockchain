use bincode::{Encode, config};
use k256::ecdsa::SigningKey;
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{block::Block, register_exit_callback, transaction::Transaction, tx::TxOutputs};

const DB_PATH: &str = "./blocks";
const LATEST_HASH_KEY: &str = "lsh";

pub struct Blockchain {
    pub latest_hash: String,
    pub database: Arc<Mutex<sled::Db>>,
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
        let db_client = db_client_mutex.lock().unwrap();
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
    fn init_db_client(db_path: PathBuf) -> Arc<Mutex<sled::Db>> {
        let db = sled::open(db_path).expect("Failed to open Sled db!");

        let db_client_mutex = Arc::new(Mutex::new(db));
        let db_client_callback = Arc::clone(&db_client_mutex);

        register_exit_callback(move || {
            db_client_callback
                .lock()
                .unwrap()
                .flush()
                .expect("Failed to persist sled");
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
        let db_client = db_client_mutex.lock().unwrap();

        // init coinbase & genesis block
        let coinbase_tx = Transaction::coinbase_tx(to);
        let genesis_block = Block::genesis(coinbase_tx);
        let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard())
            .ok()
            .expect("Failed to init blockchain cause encoding genesis block error");

        let block_key = genesis_block.hash.clone();
        db_client
            .insert(&block_key, encoded_block)
            .expect("Failed to save genesis block");

        let hash_bytes = hex::decode(&block_key).expect(&format!(
            "Failed to hex::decode block_key {} to bytes",
            block_key
        ));
        db_client
            .insert(LATEST_HASH_KEY, hash_bytes)
            .expect("Failed to store latest hash");

        return Blockchain {
            latest_hash: genesis_block.hash,
            database: Arc::clone(&db_client_mutex),
        };
    }

    pub fn add_block(&self, block: Block) {
        let database = self.database.lock().unwrap();

        if database.contains_key(&block.hash).unwrap() {
            return;
        }

        // save new block to DB
        let encoded_block = bincode::encode_to_vec(&block, config::standard())
            .expect("Failed to encode new added block");
        database
            .insert(&block.hash, encoded_block)
            .expect("Failed to save new added block");

        // load the last block
        let lsh_bytes = database
            .get(LATEST_HASH_KEY)
            .ok()
            .flatten()
            .expect("Failed to add_block cause there isn't latest hash in DB");

        let lsh = hex::encode(lsh_bytes);

        let last_block_bytes = database.get(&lsh).ok().flatten().expect(&format!(
            "Cannot load the last block from blockchain: {}",
            &lsh
        ));

        let (last_block, _): (Block, usize) =
            bincode::decode_from_slice(&last_block_bytes, config::standard()).unwrap();

        // save lastest hash
        if block.height > last_block.height {
            database
                .insert(
                    LATEST_HASH_KEY,
                    hex::decode(&block.hash).expect(&format!(
                        "Failed to decode hex hash {} to bytes",
                        &block.hash
                    )),
                )
                .expect("Failed to save added block");
        }
    }

    /// 根据hash获取目标的Block
    /// 
    /// # Arguments
    /// 
    /// - `&self` (`undefined`) - Blockchain
    /// - `hash` (`&[u8]`) - hash
    /// 
    /// # Returns
    /// 
    /// - `Option<Block>` - Block
    pub fn get_block(&self, hash: &[u8]) -> Option<Block> {
        let key = hex::encode(hash);
        let database = self.database.lock().unwrap();
        let val = database.get(key).ok().flatten();

        match val {
            Some(bytes) => {
                let (block, _): (Block, usize) =
                    bincode::decode_from_slice(&bytes, config::standard()).unwrap();
                return Some(block);
            }
            None => None,
        }
    }

    pub fn get_block_hashes(&self) -> Vec<String> {
        let mut iter = self.iterator();

        let mut hashes : Vec<String> = vec![];
        while let Some(block) = iter.next() {
            hashes.push(block.hash);
        }

        hashes
    }

    pub fn mine_block(&self, transactions: Vec<Transaction>) -> Block {

        // verify all transactions
        for tx in &transactions {
            let verify = self.verify_transaction(tx);
            if !verify {
                let tx_id = hex::encode(&tx.id);
                panic!("Detected invalid Transaction, id: {}", tx_id);
            }
        }

        // get block height
        let database = self.database.lock().unwrap();
        let last_hash = database.get(LATEST_HASH_KEY).ok().flatten().unwrap();
        let last_hash_key = hex::encode(last_hash);

        let last_block_bytes = database.get(last_hash_key).ok().flatten().unwrap();

        let (last_block, _) : (Block, usize) = bincode::decode_from_slice(&last_block_bytes, config::standard()).unwrap();

        // do mine
        let new_block = Block::create_block(last_block.hash.clone(), transactions, last_block.height);

        // save new block & update lsh
        let new_block_bytes = bincode::encode_to_vec(&new_block, config::standard()).unwrap();
        database.insert(&new_block.hash, new_block_bytes).unwrap();
        let hash_bytes = hex::decode(&new_block.hash).unwrap();
        database.insert(LATEST_HASH_KEY, hash_bytes).unwrap();

        new_block
    }

    pub fn get_height(&self) -> u128 {
        let database = self.database.lock().unwrap();
        if let Some(lsh) = database.get(LATEST_HASH_KEY).ok().flatten() {
            let lastest_hash = hex::encode(lsh);
            if let Some(block_bytes) = database.get(lastest_hash).ok().flatten() {
                let (block, _): (Block, usize) =
                    bincode::decode_from_slice(&block_bytes, config::standard()).unwrap();
                return block.height;
            }
        }

        panic!("Didn't find the block height!")
    }

    pub fn iterator(&self) -> Iterator {
        let database = self.database.lock().unwrap();
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

    /*
     * 寻找某地址所有未支付Output
     */
    pub fn find_utxos(&self) -> HashMap<String, TxOutputs> {
        let mut iter = self.iterator();
        let mut utxos = HashMap::<String, TxOutputs>::default();
        let mut spent_outputs = HashMap::<String, Vec<usize>>::default();

        while let Some(block) = iter.next() {
            for tx in block.transactions {
                let tx_id = hex::encode(&tx.id);

                // process tx inputs
                if !tx.is_coinbase() {
                    for input in tx.inputs {
                        spent_outputs
                            .entry(tx_id.clone())
                            .or_insert_with(Vec::new)
                            .push(input.out_idx);
                    }
                }

                for (idx, output) in tx.outputs.into_iter().enumerate() {
                    if spent_outputs.contains_key(&tx_id) {
                        if spent_outputs.get(&tx_id).unwrap().contains(&idx) {
                            continue;
                        }
                    }

                    utxos
                        .entry(tx_id.clone())
                        .or_insert_with(TxOutputs::default)
                        .push(output);
                }
            }
        }

        utxos
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
    pub fn find_transaction(&self, tx_id: &[u8]) -> Transaction {
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
    pub fn sign_transaction(&self, tx_to_sign: &mut Transaction, priv_key: &mut SigningKey) {
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
    pub fn verify_transaction(&self, tx_to_verify: &Transaction) -> bool {
        let mut prev_txs: HashMap<String, Transaction> = HashMap::default();

        for input in &tx_to_verify.inputs {
            let tx = self.find_transaction(&input.tx_id);
            prev_txs.insert(hex::encode(&input.tx_id), tx);
        }

        tx_to_verify.verity(prev_txs)
    }
}

pub struct Iterator {
    pub database: Arc<Mutex<sled::Db>>,
    pub current_hash: String,
}

impl Iterator {
    pub fn next(&mut self) -> Option<Block> {
        if &self.current_hash == "" {
            return None;
        }
        let database = self.database.lock().unwrap();

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
