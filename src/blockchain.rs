use bincode::config;
use readb::{Database, DatabaseSettings};
use serde_json::map::Iter;
use std::path::PathBuf;

use crate::block::Block;

const DB_PATH: &str = "./tmp/blocks";
const LATEST_HASH_KEY: &str = "lsh";

pub struct Blockchain {
    pub latest_hash: Vec<u8>,
    pub database: readb::DefaultDatabase,
}

pub struct Iterator<'a> {
    pub database: &'a mut readb::DefaultDatabase,
    pub current_hash: String,
}

impl Blockchain {
    pub fn init() -> Self {
        let db_path = PathBuf::from(DB_PATH);
        let database_config = DatabaseSettings {
            path: Some(db_path),
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: true,
        };

        let mut db_client = readb::DefaultDatabase::new(database_config);
        let lsh_value = db_client.get(LATEST_HASH_KEY);

        // init blockchain with latest_hash value in DB
        if let Some(lsh) = lsh_value.ok().flatten() {
            Blockchain {
                latest_hash: lsh,
                database: db_client,
            }
        } else {
            // init without block-data in DB
            let genesis_block = Block::genesis();
            let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard())
                .ok()
                .expect("Failed to init blockchain cause encoding genesis block error");

            let block_key = String::from_utf8(genesis_block.hash.clone())
                .expect("Failed to create genesis block db-key");
            db_client
                .put(&block_key, &encoded_block)
                .expect("Failed to save genesis block");

            db_client
                .put(LATEST_HASH_KEY, &encoded_block)
                .expect("Failed to put genesis block into DB");
            return Blockchain {
                latest_hash: genesis_block.hash,
                database: db_client,
            };
        }
    }

    pub fn add_block(&mut self, data: String) {
        let database = &mut self.database;
        let lsh = database
            .get(LATEST_HASH_KEY)
            .ok()
            .flatten()
            .expect("Failed to add_block cause there isn't  latest hash in DB");

        let block = Block::create_block(lsh, data);
        database
            .put(LATEST_HASH_KEY, &block.hash)
            .expect("Failed to save added block");
        let encoded_block = bincode::encode_to_vec(&block, config::standard())
            .expect("Failed to encode new added block");
        database
            .put(&String::from_utf8(block.hash).unwrap(), &encoded_block)
            .expect("Failed to save new added block");
    }

    pub fn iterator<'a>(&'a mut self) -> Iterator<'a> {
        if let Some(lsh) = self.database.get(LATEST_HASH_KEY).ok().flatten() {
            return Iterator {
                database: &mut self.database,
                current_hash: String::from_utf8(lsh)
                    .expect("Failed to create string from latest hash"),
            };
        }
        Iterator {
            database: &mut self.database,
            current_hash: String::default(),
        }
    }
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
        self.current_hash = String::from_utf8(prev_hash).unwrap();
        Some(block)
    }
}
