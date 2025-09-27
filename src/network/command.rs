use std::sync::Arc;

use bincode::{Decode, Encode, config};

use crate::{block::Block, transaction::Transaction};

pub enum Cmd {
    Height,
    Getblocks,
    SendInv,
    GetData,
    SendBlock,
    SendTx,
    Unknown,
}

impl Cmd {
    pub fn encode(&self) -> [u8; 2] {
        match self {
            Cmd::Height => [0u8, 1u8],
            Cmd::Getblocks => [0u8, 2u8],
            Cmd::SendInv => [0u8, 3u8],
            Cmd::GetData => [0u8, 4u8],
            Cmd::SendBlock => [0u8, 5u8],
            Cmd::SendTx => [0u8, 6u8],
            Cmd::Unknown => [255u8, 255u8],
        }
    }

    pub fn decode(bytes: [u8; 2]) -> Cmd {
        let seri: u16 = u16::from_be_bytes(bytes);
        match seri {
            1u16 => Cmd::Height,
            _ => Cmd::Unknown,
        }
    }
}

/// Cmd header
///
/// # Fields
///
/// - `ver` (`u8`) - ver, 1 byte
/// - `len` (`u32`) - length of body, 4 bytes.
/// - `cmd` (`Cmd`) - cmd type, 2 bytes.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let s = CmdHeader {
///     ver: value,
///     len: value,
///     cmd: value,
/// };
/// ```
pub trait Command {
    fn serialize(&self) -> Vec<u8>;
    fn version(&self) -> u8;
}

#[derive(Encode, Decode)]
pub struct HeightCmd {
    pub node_addr: Arc<String>,
    pub height: u32,
}

impl Command for HeightCmd {
    fn serialize(&self) -> Vec<u8> {
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];
        let ver = self.version();
        // ver, 1 byte
        result.push(ver);
        // todo!
        let length = payload.len() as u32;
        // len, 4 bytes, cause HeightCmd only has a 'height' field, u32
        result.extend_from_slice(&length.to_be_bytes());
        // cmd, 2 bytes
        result.extend_from_slice(&Cmd::Height.encode());
        // `height` field
        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

impl HeightCmd {
    pub fn new(node_addr: Arc<String>, height: u32) -> Self {
        Self { node_addr, height }
    }
}

#[derive(Decode, Encode)]
pub struct GetblocksCmd {
    pub node_addr: Arc<String>,
}

impl Command for GetblocksCmd {
    fn serialize(&self) -> Vec<u8> {
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];
        let ver = self.version();
        // ver, 1 byte
        result.push(ver);
        let length = payload.len() as u32;
        // len, 4 bytes, cause HeightCmd only has a 'height' field, u32
        result.extend_from_slice(&length.to_be_bytes());
        // cmd, 2 bytes
        result.extend_from_slice(&Cmd::Getblocks.encode());
        // payload field
        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

impl GetblocksCmd {
    pub fn new(node_addr: Arc<String>) -> Self {
        Self { node_addr }
    }
}

#[derive(Encode, Decode)]
pub struct SendInvCmd {
    pub node_addr: Arc<String>,
    pub inv_type: InvType,
    pub items: Vec<String>,
}

impl Command for SendInvCmd {
    fn serialize(&self) -> Vec<u8> {
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];

        let ver = self.version();
        // 1 byte for ver
        result.push(ver);
        // 4 bytes for length
        let length = payload.len() as u32;
        result.extend_from_slice(&length.to_be_bytes());

        // 2 bytes for cmd
        result.extend_from_slice(&Cmd::SendInv.encode());
        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

#[derive(Decode, Encode)]
pub enum InvType {
    Block,
    Tx,
}

impl SendInvCmd {
    pub fn new(node_addr: Arc<String>, inv_type: InvType, items: Vec<String>) -> Self {
        Self {
            node_addr,
            inv_type,
            items,
        }
    }
}

#[derive(Encode, Decode)]
pub struct GetDataCmd {
    pub node_addr: Arc<String>,
    pub inv_type: InvType,
    pub id: String,
}

impl Command for GetDataCmd {
    fn serialize(&self) -> Vec<u8> {
        let mut result = vec![];
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let ver = self.version();
        result.push(ver);

        let length = payload.len() as u32;
        result.extend_from_slice(&length.to_be_bytes());

        result.extend_from_slice(&Cmd::GetData.encode());

        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

#[derive(Encode, Decode)]
pub struct SendBlockCmd {
    pub node_addr: Arc<String>,
    pub block: Block,
}

impl Command for SendBlockCmd {
    fn serialize(&self) -> Vec<u8> {
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];

        let ver = self.version();
        result.push(ver);

        let length = payload.len() as u32;
        result.extend_from_slice(&length.to_be_bytes());
        result.extend_from_slice(&Cmd::SendBlock.encode());

        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

impl SendBlockCmd {
    pub fn new(node_addr: Arc<String>, block: Block) -> Self {
        Self { node_addr, block }
    }
}

#[derive(Encode, Decode)]
pub struct SendTxCmd {
    pub node_addr: Arc<String>,
    pub tx: Transaction,
}

impl Command for SendTxCmd {
    fn serialize(&self) -> Vec<u8> {
        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];

        let ver = self.version();
        result.push(ver);

        let length = payload.len() as u32;
        result.extend_from_slice(&length.to_be_bytes());
        result.extend_from_slice(&Cmd::SendTx.encode());

        result.extend_from_slice(&payload);

        result
    }

    fn version(&self) -> u8 {
        1u8
    }
}

impl SendTxCmd {
    pub fn new(node_addr: Arc<String>, tx: Transaction) -> Self {
        Self { node_addr, tx }
    }
}
