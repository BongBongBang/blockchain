#![allow(dead_code)]
mod proof_of_word;
mod block;
mod blockchain;
mod cli;


use clap::Parser;

use crate::cli::CommandLine;

    
fn main() {
    let mut blockchain = blockchain::Blockchain::init();
    let command_line = CommandLine { blockchain : &mut blockchain};

    let cli = cli::Cli::parse();
    match cli.operation.as_str() {
        "add" => {println!("{:?}", cli)}
        _ => command_line.print_usage(),
    }
    println!("{:?}", cli);
}
