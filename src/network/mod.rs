pub mod command;

use bytes::{BufMut, BytesMut};
use futures::SinkExt;
use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::network::command::Command;

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
