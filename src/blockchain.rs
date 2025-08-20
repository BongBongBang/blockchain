use base64::{Engine, engine::general_purpose};
use bincode::{config, Encode};
use readb::{Database, DatabaseSettings};
use std::path::PathBuf;

use crate::block::Block;

const DB_PATH: &str = "./blocks";
const LATEST_HASH_KEY: &str = "lsh";

pub struct Blockchain {
    pub latest_hash: String,
    pub database: readb::DefaultDatabase,
}

impl Encode for Blockchain {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
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
                latest_hash: general_purpose::STANDARD.encode(lsh),
                database: db_client,
            }
        } else {
            // init without block-data in DB
            let genesis_block = Block::genesis();
            let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard())
                .ok()
                .expect("Failed to init blockchain cause encoding genesis block error");

            let block_key = genesis_block.hash.clone();
            db_client
                .put(&block_key, &encoded_block)
                .expect("Failed to save genesis block");

            db_client
                .put(
                    LATEST_HASH_KEY,
                    &general_purpose::STANDARD
                        .decode(block_key)
                        .expect("Failed to decode genesis block's hash to hex str"),
                )
                .expect("Failed to put genesis block into DB");
            // db_client
            //     .persist()
            //     .expect("Failed to store genesis block data !!!");
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

        let block = Block::create_block(general_purpose::STANDARD.encode(lsh), data);
        database
            .put(
                LATEST_HASH_KEY,
                &general_purpose::STANDARD
                    .decode(&block.hash)
                    .unwrap_or_else(|_e| {
                        panic!("Fail to decode hash: {:?}", &block.hash);
                    }),
            )
            .expect("Failed to save added block");
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
                current_hash: general_purpose::STANDARD.encode(lsh),
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
        self.current_hash = prev_hash;
        Some(block)
    }
}
