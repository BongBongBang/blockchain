pub mod command;

use std::sync::{LazyLock, OnceLock};

use tokio::{
    io::AsyncWriteExt, net::{TcpListener, TcpSocket, TcpStream}, sync::RwLock
};

use crate::network::command::Command;

// static NODE_ID: OnceLock<u32> = OnceLock::new();
// static MINER_ADDRESS: OnceLock<String> = OnceLock::new();
// static KNOWN_HOSTS: LazyLock<RwLock<Vec<String>>> =
//     LazyLock::new(|| RwLock::new(vec![String::from("localhost:3000")]));

struct Server {
    pub node_id: u32,
    pub miner_address: String,
    pub konwn_hosts: Vec<String>,
}

impl Server {
    /// start a blockchain node
    ///
    /// # Arguments
    ///
    /// - `node_id` (`u32`) - node_id
    /// - `miner_address` (`String`) - miner address
    pub async fn start_node(&self, node_id: u32, miner_address: String) {
        // init local node info
        // NODE_ID.set(node_id);
        // MINER_ADDRESS.set(miner_address);

        // // start node server
        // let addr = format!("localhost:{}", node_id);
        // let listener = TcpListener::bind(&addr).await.unwrap();

        // // sync version to center node
        // if &addr != KNOWN_HOSTS.read().await.get(0).unwrap() {
        //     // send_version();
        // }

        // // process income
        // loop {
        //     let (socket, _) = listener.accept().await.unwrap();
        // }
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
