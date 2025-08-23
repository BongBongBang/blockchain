use bincode::{Encode, config};
use readb::{Database, DatabaseSettings};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
};

use crate::{
    block::{self, Block},
    transaction::{self, Transaction, TxOutput},
};

const DB_PATH: &str = "./blocks";
const LATEST_HASH_KEY: &str = "lsh";

pub struct Blockchain {
    pub latest_hash: String,
    pub database: readb::DefaultDatabase,
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
        let database_config = DatabaseSettings {
            path: Some(db_path),
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: true,
        };

        let mut db_client = readb::DefaultDatabase::new(database_config);
        let lsh_value = db_client.get(LATEST_HASH_KEY);

        if let Some(lsh) = lsh_value.ok().flatten() {
            Blockchain {
                latest_hash: hex::encode(lsh),
                database: db_client,
            }
        } else {
            panic!("Blockchain latest hash doesn't exist, init blockchain first!");
        }
    }
    pub fn init(to: String) -> Self {
        // 判断本地数据库是否存在
        let db_path = PathBuf::from(DB_PATH);
        if db_path.exists() {
            panic!("Blockchain has already existed, just continue it!");
        }

        // 初始化数据库
        let database_config = DatabaseSettings {
            path: Some(db_path),
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: false,
        };

        let mut db_client = readb::DefaultDatabase::new(database_config);

        // init coinbase & genesis block
        let coinbase_tx = Transaction::coinbase_tx(to);
        let genesis_block = Block::genesis(coinbase_tx);
        let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard())
            .ok()
            .expect("Failed to init blockchain cause encoding genesis block error");

        let block_key = genesis_block.hash.clone();
        db_client
            .put(&block_key, &encoded_block)
            .expect("Failed to save genesis block");

        db_client.put(
            LATEST_HASH_KEY,
            &hex::decode(block_key.clone()).expect(&format!(
                "Failed to hex::decode block_key {} to bytes",
                block_key
            )),
        );
        // db_client
        //     .persist()
        //     .expect("Failed to store genesis block data !!!");
        return Blockchain {
            latest_hash: genesis_block.hash,
            database: db_client,
        };
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let database = &mut self.database;
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

        database
            .persist()
            .expect("Failed to store added block data !!!");
    }

    pub fn iterator<'a>(&'a mut self) -> Iterator<'a> {
        if let Some(lsh) = self.database.get(LATEST_HASH_KEY).ok().flatten() {
            return Iterator {
                database: &mut self.database,
                current_hash: hex::encode(lsh),
            };
        }
        Iterator {
            database: &mut self.database,
            current_hash: String::default(),
        }
    }

    // 寻找所有未花费的transaction
    pub fn find_unspent_tx(&mut self, address: &'static str) -> Vec<Transaction> {
        let mut iter = self.iterator();

        let mut unspent_tx = Vec::default();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        // block layer
        while let Some(block) = iter.next() {
            // transaction layer
            'transaction: for tx in block.transactions {
                let tx_id = hex::encode(&tx.id);

                // output layer
                for (out_idx, output) in tx.outputs.iter().enumerate() {
                    if spent_txos.contains_key(&tx_id) {
                        let out_idxes = spent_txos.get(&tx_id).unwrap();
                        // 如果这个out_idx的output已经被花费掉了
                        if out_idxes.contains(&out_idx) {
                            continue;
                        }
                    } else {
                        // 当前tx没有被input引用过
                        // 判断当前Output是否属于目标用户
                        if output.belongs(address) {
                            // 如果不是coinbase，记录所有的input引用
                            if !tx.is_coinbase() {
                                for input in &tx.inputs {
                                    if let Some(out_idxes) = spent_txos.get_mut(&tx_id) {
                                        out_idxes.push(input.out_idx);
                                    } else {
                                        let out_idxes = vec![input.out_idx];
                                        spent_txos.insert(tx_id.clone(), out_idxes);
                                    }
                                }
                            }
                            
                            unspent_tx.push(tx);
                            continue 'transaction;
                        }
                    }
                }
            }
        }

        unspent_tx
    }

    pub fn find_utxo() -> Vec<TxOutput> {
        vec![]
    }

    pub fn find_spendable_outputs() {}
}

pub struct Iterator<'a> {
    pub database: &'a mut readb::DefaultDatabase,
    pub current_hash: String,
}

impl<'a> Iterator<'a> {
    pub fn next(&mut self) -> Option<Block> {
        if &self.current_hash == "" {
            return None;
        }
        let database = &mut self.database;

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
