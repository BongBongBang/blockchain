#![allow(dead_code)]
mod block;
mod blockchain;
mod cli;
mod proof_of_word;

use clap::Parser;

use crate::cli::{CommandLine, CliOperation};

fn main() {
    let mut blockchain = blockchain::Blockchain::init();
    let mut command_line = CommandLine {
        blockchain: &mut blockchain,
    };

    let cli = cli::Cli::parse();
    cli.validate_args();
    match cli.operation {
        CliOperation::Add => command_line.add_block(cli.block_data.unwrap()),
        CliOperation::PrintChain => command_line.print_chain(),
        _ => command_line.print_usage(),
    }
}
