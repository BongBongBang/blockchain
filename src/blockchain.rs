use std::path::PathBuf;
use bincode::config;
use readb::{Database, DatabaseSettings};

use crate::block::Block;

const DB_PATH : &str = "./tmp/blocks";
const LATEST_HASH_KEY : &str = "lsh";

pub struct Blockchain {
    pub latest_hash: Vec<u8>,
    pub readb_client: readb::DefaultDatabase
}

impl Blockchain {

    pub fn init() -> Self {
        let db_path = PathBuf::from(DB_PATH);
        let database_config = DatabaseSettings {
            path: Some(db_path),
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: true
        };

        let mut db_client = readb::DefaultDatabase::new(database_config);
        let lsh_value = db_client.get(LATEST_HASH_KEY);

        // init blockchain with latest_hash value in DB
        if let Some(lsh) = lsh_value.ok().flatten() {
            
            Blockchain {latest_hash: lsh, readb_client: db_client}

        } else {
            // init without block-data in DB
            let genesis_block = Block::genesis();
            let encoded_block = bincode::encode_to_vec(&genesis_block, config::standard()).ok();

            if let Some(value) = encoded_block {
                if db_client.put(LATEST_HASH_KEY, &value).is_ok() {
                    return Blockchain { latest_hash: genesis_block.hash, readb_client: db_client };
                } else {
                    panic!("Failed to put genesis block into DB");
                }
            } else {
                panic!("Failed to init blockchain cause encoding genesis block error");
            }
        }
    }
}