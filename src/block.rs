#[derive(Debug)]
pub struct Block {


    pub prev_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub data: Vec<u8>,
    pub nonce: u128
}