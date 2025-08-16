use clap::Parser;

use crate::{blockchain, proof_of_word::ProofOfWork};

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(index = 1)]
    pub operation: String,
    #[arg(short = 'd', long = "data")]
    pub block_data: String
}

pub struct CommandLine<'a> {
    pub blockchain: &'a mut blockchain::Blockchain
}

impl<'a> CommandLine<'a> {
    pub fn add_block(&mut self, data : String) {
        self.blockchain.add_block(data); 
        println!("Added Block!");
    }

    pub fn print_usage(&self) {
        println!("Usage:");
        println!("add -BLOCK_DATA - add a block to the chain");
        println!("print - print blocks in the chain");
    }

    // validate args input by user
    pub fn validate_args(&self, args: &Vec<String>) {
        if args.len() < 2 {
            self.print_usage();
            panic!();
        }
    }

    pub fn print_chain(&mut self) {
        let mut iter = self.blockchain.iterator();
        loop {
            if let Some(block) = &iter.next() {
                
                println!("Prev hash: {:?}", &block.prev_hash);
                println!("Data: {:?}", &block.data);
                println!("Hash: {:?}", &block.hash);
                let pow = ProofOfWork::new(&block);
                println!("Pow: {:?}", pow.validate());

            } else {
                println!("Iterate all block!");
            }
        }
    }
}
