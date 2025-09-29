pub mod command;

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use bincode::config;
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio::{io, net::{TcpListener, TcpStream}};
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::{
    blockchain::Blockchain,
    network::command::{
        Cmd, Command, GetDataCmd, GetblocksCmd, HeightCmd, InvType, SendBlockCmd, SendInvCmd,
    },
};

struct Server {
    pub node_id: u32,
    pub node_address: Arc<String>,
    pub miner_address: String,
    pub known_hosts: Vec<String>,
    // todo!Mutex
    pub blocks_in_transimission: HashMap<String, VecDeque<String>>,
}

impl Server {
    pub fn new(node_id: u32, miner_address: String) -> Self {
        Self {
            node_id,
            node_address: Arc::new(format!("localhost:{}", node_id)),
            miner_address,
            known_hosts: vec![String::from("localhost:3000")],
            blocks_in_transimission: HashMap::default(),
        }
    }

    /// start a blockchain node
    ///
    /// # Arguments
    ///
    /// - `node_id` (`u32`) - node_id
    /// - `miner_address` (`String`) - miner address
    pub async fn start_node(&mut self) {
        // continue local blockchain
        let blockchain = Blockchain::continue_chain();
        // start node server
        let addr = format!("localhost:{}", self.node_id);
        let listener = TcpListener::bind(&addr).await.unwrap();

        // sync version to center node
        if &addr != self.known_hosts.get(0).unwrap() {
            // send_version();
        }

        // process income
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let mut framed = Framed::new(socket, LengthHeaderDelimiter {});
            while let Some(Ok(package)) = framed.next().await {
                self.process(package, &blockchain).await;
            }
        }
    }

    async fn process(&mut self, package: Vec<u8>, blockchain: &Blockchain) {
        // let ver = package[0];
        let cmd = Cmd::decode(package[5..7].try_into().unwrap());

        match cmd {
            Cmd::Height => self.handle_height(package, blockchain).await,
            Cmd::Getblocks => self.handle_getblocks(package, blockchain).await,
            Cmd::SendInv => self.handle_sendinvcmd(package).await,
            Cmd::GetData => self.handle_getdatacmd(package, blockchain).await,
            Cmd::SendBlock => self.handle_sendblockcmd(package, blockchain).await,
            Cmd::SendTx => {}
            Cmd::Unknown => {
                println!("Receive unknown cmd!!");
            }
        }
    }

    async fn handle_getblocks(&self, package: Vec<u8>, blockchain: &Blockchain) {
        let (payload, _): (GetblocksCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();
        let addr_from = payload.node_addr;
        let block_hashes = blockchain.get_block_hashes();

        let node_addr = Arc::clone(&self.node_address);
        let send_inv_cmd = SendInvCmd::new(node_addr, InvType::Block, block_hashes);

        self.transmit(&addr_from, send_inv_cmd);
    }

    async fn handle_sendinvcmd(&mut self, package: Vec<u8>) {
        let (payload, _): (SendInvCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();

        // 取出当前请求的block id
        let mut items_deque: VecDeque<String> = payload.items.into_iter().collect();
        let id = items_deque.pop_front().unwrap_or_default();

        // 剩下的直接存到server.blocks_in_transimission
        // todo!: host_key下原有的带请求blocks
        self.blocks_in_transimission
            .insert(payload.node_addr.to_string(), items_deque);

        // 发出请求getdata
        let inv_type = payload.inv_type;
        let mut block_ids: VecDeque<String> = payload.items.into_iter().collect();
        let block_id = block_ids.pop_front().unwrap();

        // 剩下的传输blocs存到变量里
        self.blocks_in_transimission
            .insert(payload.node_addr.to_string(), block_ids);

        match inv_type {
            InvType::Block => {
                self.send_getdata(&payload.node_addr, InvType::Block, &block_id)
                    .await;
            }
            InvType::Tx => {
                self.send_getdata(&payload.node_addr, InvType::Tx, &block_id).await;
            }
        }
    }

    async fn handle_sendblockcmd(&mut self, package: Vec<u8>, blockchain: &Blockchain) {
        let (payload, _): (SendBlockCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();

        // 添加block入库
        let block = payload.block;
        blockchain.add_block(block);

        // 获取下一个待获取的block
        if !self.blocks_in_transimission.is_empty() {
            let host_key = self.blocks_in_transimission.keys().next().unwrap().clone();
            let blocks = self.blocks_in_transimission.get_mut(&host_key).unwrap();
            let block_to_get = blocks.pop_front().unwrap();

            self.send_getdata(&host_key, InvType::Block, &block_to_get);
        }
    }

    async fn handle_getdatacmd(&self, package: Vec<u8>, blockchain: &Blockchain) {
        let (payload, _): (GetDataCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();

        let id = payload.id;
        let addr_from = payload.node_addr;
        match payload.inv_type {
            InvType::Block => {
                let id_bytes = hex::decode(&id).unwrap();
                if let Some(block) = blockchain.get_block(&id_bytes) {
                    let send_block_cmd = SendBlockCmd::new(Arc::clone(&self.node_address), block);
                    self.transmit(&addr_from, send_block_cmd).await;
                } else {
                    println!("Cannot find target block, id: {}", &id);
                }
            }
            InvType::Tx => {
                todo!();
            }
        }
    }

    async fn send_getdata(&self, addr: &str, inv_type: InvType, id: &str) -> Result<(), io::Error> {
        let get_data_cmd = GetDataCmd {
            node_addr: Arc::clone(&self.node_address),
            inv_type,
            id: id.to_string(),
        };

        self.transmit(&addr, get_data_cmd).await?;

        Ok(())
    }

    async fn handle_height(&self, package: Vec<u8>, blockchain: &Blockchain) {
        let (payload, _): (HeightCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();
        let local_height = blockchain.get_height();
        if local_height > payload.height as u128 {
            // send version
            self.send_height(payload.node_addr, blockchain).await;
        } else {
            // send get blocks
            self.send_getblocks(payload.node_addr).await;
        }
    }

    async fn send_height(
        &self,
        addr: Arc<String>,
        blockchain: &Blockchain,
    ) -> Result<(), io::Error> {
        let height = blockchain.get_height();
        let height_cmd = HeightCmd::new(self.node_address.clone(), height as u32);
        self.transmit(&addr, height_cmd).await?;

        Ok(())
    }

    async fn send_getblocks(&self, addr: Arc<String>) -> Result<(), io::Error> {
        let cmd = GetblocksCmd::new(self.node_address.clone());
        self.transmit(&addr, cmd).await?;

        Ok(())
    }
}

pub struct LengthHeaderDelimiter;

impl Decoder for LengthHeaderDelimiter {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let _ver = u8::from_be_bytes(src[..1].try_into().unwrap());
        let len = u32::from_be_bytes(src[1..4].try_into().unwrap());

        if src.len() < len as usize {
            return Ok(None);
        }

        let payload: Vec<u8> = src.split_to(5 + len as usize).into();

        Ok(Some(payload))
    }
}

impl Encoder<BytesMut> for LengthHeaderDelimiter {
    type Error = io::Error;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = item.len();

        dst.reserve(len);
        dst.put(item);

        Ok(())
    }
}

trait Transmitter {
    async fn transmit<T: Command>(&self, addr: &str, cmd: T) -> Result<(), io::Error>;
}

impl Transmitter for Server {
    async fn transmit<T: Command>(&self, addr: &str, cmd: T) -> Result<(), io::Error> {
        // dial to target addr
        let client_stream = TcpStream::connect(addr).await.unwrap();
        let mut framed = Framed::new(client_stream, LengthHeaderDelimiter {});

        let mut payload = BytesMut::new();
        payload.extend_from_slice(&cmd.serialize());
        // framed.send.await返回的是Result<(), framed的编解码器的错误类型>
        framed.send(payload).await?;

        Ok(())
    }
}
