pub mod command;

use bincode::config;
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::{
    blockchain::Blockchain,
    network::command::{Cmd, Command, HeightCmd},
};

struct Server {
    pub node_id: u32,
    pub miner_address: String,
    pub known_hosts: Vec<String>,
}

impl Server {
    pub fn new(node_id: u32, miner_address: String) -> Self {
        Self {
            node_id,
            miner_address,
            known_hosts: vec![String::from("localhost:3000")],
        }
    }

    /// start a blockchain node
    ///
    /// # Arguments
    ///
    /// - `node_id` (`u32`) - node_id
    /// - `miner_address` (`String`) - miner address
    pub async fn start_node(&self) {
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

    async fn process(&self, package: Vec<u8>, blockchain: &Blockchain) {
        // let ver = package[0];
        let cmd = Cmd::decode(package[5..7].try_into().unwrap());

        match cmd {
            Cmd::Height => self.handle_height(package, blockchain).await,
            Cmd::Unknown => {
                println!("Receive unknown cmd!!");
            }
        }
    }

    async fn handle_height(&self, package: Vec<u8>, blockchain: &Blockchain) {
        let (payload, _): (HeightCmd, usize) =
            bincode::decode_from_slice(&package[7..], config::standard()).unwrap();
        let local_height = blockchain.get_height();
        if local_height > payload.height as u128 {
            // send version
        } else {
            // send get blocks
        }
    }

    async fn send_height() {

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
    async fn transmit<T: Command>(&self, addr: &str, cmd: T);
}

impl Transmitter for Server {
    async fn transmit<T: Command>(&self, addr: &str, cmd: T) {
        // dial to target addr
        let client_stream = TcpStream::connect(addr).await.unwrap();
        let mut framed = Framed::new(client_stream, LengthHeaderDelimiter {});

        let mut payload = BytesMut::new();
        payload.extend_from_slice(&cmd.serialize());
        framed.send(payload).await.unwrap();
    }
}
