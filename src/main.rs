#![allow(dead_code)]
mod proof_of_word;
mod block;
mod blockchain;
mod cli;

use std::env;

use proof_of_word::ProofOfWork;

fn main() {

    let args: Vec<String> = env::args().collect();
    let cmd = &args[1];

}
