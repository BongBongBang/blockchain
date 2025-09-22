pub mod command;

use std::sync::{LazyLock, OnceLock};

use bytes::BytesMut;
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
    sync::RwLock,
};
use tokio_util::codec::{Decoder, Encoder};

use crate::network::command::Command;

// static NODE_ID: OnceLock<u32> = OnceLock::new();
// static MINER_ADDRESS: OnceLock<String> = OnceLock::new();
// static KNOWN_HOSTS: LazyLock<RwLock<Vec<String>>> =
//     LazyLock::new(|| RwLock::new(vec![String::from("localhost:3000")]));

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
        }
    }
}


pub struct LengthHeaderDelimiter;

impl Decoder for LengthHeaderDelimiter {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl Encoder<BytesMut> for LengthHeaderDelimiter {
    type Error = io::Error;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(additional);
        todo!()
    }
}


trait Transmitter {
    async fn transmit<T: Command>(&self, addr: &str, cmd: T);
}

impl Transmitter for Server {
    async fn transmit<T: Command>(&self, addr: &str, cmd: T) {
        // serialize cmd data
        let payload = cmd.serialize();
        // dial to target addr
        let mut client = TcpStream::connect(addr).await.unwrap();
        client.write_all(&payload);
    }
}
