use std::{rc::Rc, sync::Arc};

use base58::FromBase58;
use bytes::BytesMut;
use clap::{Parser, ValueEnum};
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    blockchain::Blockchain, cli, network::{command::{Command, SendTxCmd}, LengthHeaderDelimiter, Server}, proof_of_work::ProofOfWork, transaction::Transaction, utxo::UTXOSet, wallet::{self, Wallet}, wallets::Wallets
};

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum CliOperation {
    #[clap(rename_all = "kebab-case")]
    GetBalance,
    #[clap(rename_all = "kebab-case")]
    PrintChain,
    #[clap(rename_all = "kebab-case")]
    Send,
    #[clap(rename_all = "kebab-case")]
    CreateChain,
    #[clap(rename_all = "kebab-case")]
    PrintUsage,
    #[clap(rename_all = "kebab-case")]
    CreateWallet,
    #[clap(rename_all = "kebab-case")]
    ListAddress,
    #[clap(rename_all = "kebab-case")]
    Rebuild,
    #[clap(rename_all = "kebab-case")]
    StartNode,
}

#[derive(Parser, Debug)]
pub struct CliParam {
    #[arg(index = 1)]
    pub operation: CliOperation,

    #[arg(long = "address")]
    pub address: Option<String>,

    #[arg(long = "from")]
    pub from: Option<String>,

    #[arg(long = "to")]
    pub to: Option<String>,

    #[arg(long = "amount")]
    pub amount: Option<u128>,

    #[arg(long = "miner-address")]
    pub miner_address: Option<String>,

    #[arg(long = "node-id")]
    pub node_id: u32,

    #[arg(long = "mine")]
    pub mine: Option<bool>,
}

impl CliParam {
    // validate args input by user
    pub fn validate_args(&self) {
        if self.operation == CliOperation::Send && (self.from.is_none() || self.to.is_none()) {
            panic!("[send] operation requires -from -to argument");
        }
    }
}

pub struct CommandLine {
    // pub blockchain: &'a mut blockchain::Blockchain,
    pub cli_param: CliParam,
}

impl CommandLine {
    pub fn new() -> Self {
        let cli = cli::CliParam::parse();
        cli.validate_args();

        CommandLine { cli_param: cli }
    }

    pub async fn run(&mut self) {
        match self.cli_param.operation {
            CliOperation::CreateChain => self.create_chain().await,
            CliOperation::GetBalance => self.get_balance().await,
            CliOperation::PrintChain => self.print_chain().await,
            CliOperation::CreateWallet => self.create_wallet(),
            CliOperation::ListAddress => self.get_all_address(),
            CliOperation::Send => self.send().await,
            CliOperation::PrintUsage => self.print_usage(),
            CliOperation::Rebuild => self.rebuild().await,
            CliOperation::StartNode => self.start_node().await,
        }
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("get-balance -address ADDRESS - Get the balance for an address");
        println!(
            "create-chain --node-id NODE_ID --address ADDRESS - Create a blockchain and send genesis reward to address."
        );
        println!("print-chain --node-id NODE_ID - Prints the blocks in the chain");
        println!(
            "send --node-id NODE_ID --from FROM --to TO --amount AMOUNT --mine - Send amount of coins. Then -mine flag is set, mine off of this node."
        );
        println!("create-wallet --node-id NODE_ID - Creates a new Wallet");
        println!("list-address --node-id NODE_ID - Lists the addresses in out wallet file");
        println!("rebuild --node-id NODE_ID - Rebuilds the UTXO set.");
        println!(
            "startnode --node-id NODE_ID --miner-address ADDRESS - Start a node with ID specified in NODE_ID"
        );
    }

    async fn start_node(&mut self) {
        let node_id = self.cli_param.node_id;
        if let Some(miner_address) = &self.cli_param.miner_address {
            if !Wallet::validate_address(miner_address) {
                panic!("Invalid miner-address: {}", miner_address); 
            }
        }
        // start server
        let miner_address: String = self.cli_param.miner_address.take().unwrap();
        let mut server = Server::new(node_id, miner_address); 
        server.start_node().await;
    }

    fn create_wallet(&mut self) {
        let node_id = self.cli_param.node_id;
        let mut wallets = Wallets::new(node_id);
        let address = wallets.add_wallet();
        wallets.save_file(node_id);
        println!("Succeed creating wallet: {}\n", address);
    }

    fn get_all_address(&mut self) {
        let node_id = self.cli_param.node_id;
        let wallets = Wallets::new(node_id);
        for address in wallets.get_all_addresses() {
            println!("Address: {}", address);
        }
    }

    async fn create_chain(&mut self) {
        let address = self.cli_param.address.take().unwrap();
        if !Wallet::validate_address(&address) {
            panic!("Address: {} is not a valid address", address);
        }

        let node_id = self.cli_param.node_id;

        let blockchain = Rc::new(Blockchain::init(node_id, address).await);
        let utxo_set = UTXOSet::new(blockchain);
        utxo_set.rebuild().await;
        println!("Created blockchain!");
    }

    async fn rebuild(&self) {
        let node_id = self.cli_param.node_id;

        let blockchain = Rc::new(Blockchain::continue_chain(node_id).await);
        let utxo_set = UTXOSet::new(blockchain);
        utxo_set.rebuild().await;
        println!("UTXO set rebuild!");
    }

    async fn print_chain(&self) {

        let node_id = self.cli_param.node_id;
        let blockchain = Blockchain::continue_chain(node_id).await;
        let mut iter = blockchain.iterator().await;
        loop {
            if let Some(block) = iter.next().await {
                println!("Height: {}", block.height);
                println!("Prev hash: {:?}", &block.prev_hash);
                println!("Hash: {:?}", &block.hash);
                let pow = ProofOfWork::new(&block);
                println!("Pow: {:?}\n\n", pow.validate());
            } else {
                println!("---------------------------------------\n");
                println!("Iterate all block!");
                break;
            }
        }
    }

    async fn get_balance(&mut self) {
        let node_id = self.cli_param.node_id;
        let blockchain = Rc::new(Blockchain::continue_chain(node_id).await);

        let address = self.cli_param.address.take().unwrap();
        if !Wallet::validate_address(&address) {
            panic!("地址: {} 不是一个合法的地址", address);
        }

        let addr_base58 = address.from_base58().unwrap();
        let pubkey_hash = &addr_base58[1..addr_base58.len() - wallet::CHECK_SUM_LENGTH];

        let utxo_set = UTXOSet::new(blockchain);

        let utxos = utxo_set.find_utxo(pubkey_hash).await;
        if utxos.len() == 0 {
            println!("Address {} doesn't own any coin!", &address);
        } else {
            let mut accumulated = 0u128;
            for utxo in utxos {
                accumulated += utxo.amount;
            }
            println!("Address {} has {} coins!", address, accumulated);
        }
    }

    async fn send(&mut self) {
        let cli_param = &mut self.cli_param;
        // 校验发送、接收钱包地址
        if !Wallet::validate_address(&cli_param.from.as_deref().unwrap()) {
            panic!(
                "From address: {} is not a valid address",
                &cli_param.from.take().unwrap()
            );
        }
        if !Wallet::validate_address(&cli_param.to.as_deref().unwrap()) {
            panic!(
                "To address: {} is not a valid address",
                &cli_param.to.take().unwrap()
            );
        }

        let node_id = cli_param.node_id;

        let blockchain = Rc::new(Blockchain::continue_chain(node_id).await);
        let mut utxo_set = UTXOSet::new(Rc::clone(&blockchain));

        let node_id = cli_param.node_id;
        let addr_from = cli_param.from.take().unwrap();

        let mut wallets = Wallets::new(node_id);

        // 获取转账钱包记录
        if let Some(wallet_from) = wallets.get_wallet_mut(&addr_from) {

            let tx = Transaction::new(
                wallet_from,
                &cli_param.to.take().unwrap(),
                cli_param.amount.take().unwrap(),
                &mut utxo_set,
            ).await;

            if self.cli_param.mine.unwrap() {
                let new_block = blockchain.mine_block(vec![tx]).await;
                // 更新UTXO set
                utxo_set.update(&new_block).await;
                println!("Succeed sending coin!");
            } else {
                // todo: send tcp tx to center node
                let tcp_stream = TcpStream::connect("localhost:3000").await.unwrap();
                let mut framed = Framed::new(tcp_stream, LengthHeaderDelimiter {});

                let cmd = SendTxCmd::new(Arc::new(String::from("localhost:3000")), tx);
                let mut payload = BytesMut::new();
                payload.extend_from_slice(&cmd.serialize());
                framed.send(payload).await.unwrap();
                println!("Sent Tx to center node!");
            }
        } else {
            panic!("不存在from钱包: {}", addr_from)
        }
    }
}
