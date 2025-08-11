use std::path::PathBuf;
use readb::{Database, DatabaseSettings};
use serde::Serialize;

use crate::block::Block;

const DB_PATH : &str = "./tmp/blocks";
const LATEST_HASH_KEY : &str = "lsh";

#[derive(Debug)]
pub struct Blockchain {
    pub latest_hash: Vec<u8>,
    pub readb_client: readb::DefaultDatabase
}

impl Blockchain {

    pub fn init() -> Self {
        let db_path = PathBuf::from(DB_PATH);
        let database_config = DatabaseSettings {
            path: db_path,
            index_type: readb::IndexType::HashMap,
            cache_size: None,
            create_path: true
        };

        let db_client = readb::DefaultDatabase::new(database_config);
        let lsh_value = db_client.get(LATEST_HASH_KEY);

        // init blockchain with latest_hash value in DB
        if let Some(lsh) = lsh_value.ok().flatten() {
            
            Blockchain {latest_hash: lsh, readb_client: db_client}

        } else {
            // init without block-data in DB
            let genesis_block = Block::genesis();
            genesis_block.serialize(bincode)

            Blockchain { latest_hash: genesis_block.hash, readb_client: db_client }
        }
    }
}