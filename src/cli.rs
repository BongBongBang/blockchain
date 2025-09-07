use std::rc::Rc;

use base58::FromBase58;
use clap::{Parser, ValueEnum};

use crate::{
    blockchain::Blockchain,
    cli,
    proof_of_work::ProofOfWork,
    transaction::Transaction,
    utxo::UTXOSet,
    wallet::{self, Wallet},
    wallets::Wallets,
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

    pub fn run(&mut self) {
        match self.cli_param.operation {
            CliOperation::CreateChain => self.create_chain(),
            CliOperation::GetBalance => self.get_balance(),
            CliOperation::PrintChain => self.print_chain(),
            CliOperation::CreateWallet => self.create_wallet(),
            CliOperation::ListAddress => self.get_all_address(),
            CliOperation::Send => self.send(),
            CliOperation::PrintUsage => self.print_chain(),
            CliOperation::Rebuild => self.rebuild(),
        }
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("get-balance -address ADDRESS - Get the balance for an address");
        println!("print-chain - Prints the blocks in the chain");
        println!(
            "create-chain -address ADDRESS - Create a blockchain and send genesis reward to address."
        );
        println!("send -from FROM -to TO -amount AMOUNT - Send amount of coins");
        println!("create-wallet - Creates a new Wallet");
        println!("list-address - Lists the addresses in out wallet file");
    }

    fn create_wallet(&self) {
        let mut wallets = Wallets::new();
        let address = wallets.add_wallet();
        wallets.save_file();
        println!("Succeed creating wallet: {}\n", address);
    }

    fn get_all_address(&self) {
        let wallets = Wallets::new();
        for address in wallets.get_all_addresses() {
            println!("Address: {}", address);
        }
    }

    fn create_chain(&mut self) {
        let address = self.cli_param.address.take().unwrap();
        if !Wallet::validate_address(&address) {
            panic!("Address: {} is not a valid address", address);
        }

        let blockchain = Rc::new(Blockchain::init(address));
        let utxo_set = UTXOSet::new(blockchain);
        utxo_set.rebuild();
        println!("Created blockchain!");
    }

    fn rebuild(&self) {
        let blockchain = Rc::new(Blockchain::continue_chain());
        let utxo_set = UTXOSet::new(blockchain);
        utxo_set.rebuild();
        println!("UTXO set rebuild!");
    }

    fn print_chain(&self) {
        let blockchain = Blockchain::continue_chain();
        let mut iter = blockchain.iterator();
        loop {
            if let Some(block) = iter.next() {
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

    fn get_balance(&mut self) {
        let blockchain = Rc::new(Blockchain::continue_chain());

        let address = self.cli_param.address.take().unwrap();
        if !Wallet::validate_address(&address) {
            panic!("地址: {} 不是一个合法的地址", address);
        }

        let addr_base58 = address.from_base58().unwrap();
        let pubkey_hash = &addr_base58[1..addr_base58.len() - wallet::CHECK_SUM_LENGTH];

        let utxo_set = UTXOSet::new(blockchain);

        let utxos = utxo_set.find_utxo(pubkey_hash);
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

    fn send(&mut self) {
        let cli_param = &mut self.cli_param;

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
        let blockchain = Rc::new(Blockchain::continue_chain());
        let mut utxo_set = UTXOSet::new(Rc::clone(&blockchain));

        let mut tx = Transaction::new(
            &cli_param.from.take().unwrap(),
            &cli_param.to.take().unwrap(),
            cli_param.amount.take().unwrap(),
            &mut utxo_set,
        );
         
        let tx_id = tx.hash();
        tx.id = tx_id;

        let block = blockchain.add_block(vec![tx]);
        utxo_set.update(&block);

        println!("Succeed sending coin!");
    }
}
