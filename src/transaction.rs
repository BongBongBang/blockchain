use bincode::config::standard;
use bincode::{Decode, Encode};
use sha2::Digest;

use crate::blockchain::Blockchain;

#[derive(Debug, Encode, Decode)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Debug, Encode, Decode)]
pub struct TxInput {
    pub tx_id: Vec<u8>,
    pub out_idx: usize,
    pub sig: String,
}

#[derive(Debug, Encode, Decode)]
pub struct TxOutput {
    pub amount: u128,
    pub pub_key: String,
}

impl TxOutput {
    pub fn new(amount: u128, pub_key: String) -> Self {
        TxOutput { amount, pub_key }
    }
    // 判断当前output是否归属pub_key
    pub fn belongs(&self, pub_key: &str) -> bool {
        self.pub_key == pub_key
    }
}

impl TxInput {
    pub fn new(tx_id: Vec<u8>, out_idx: usize, sig: String) -> Self {
        TxInput {
            tx_id,
            out_idx,
            sig,
        }
    }

    // 判断当前input是否是pub_key的来源
    pub fn tranferred_to(&self, pub_key: &'static str) -> bool {
        self.sig == pub_key
    }
}

impl Transaction {
    pub fn set_id(&mut self) {
        let id_bytes = bincode::encode_to_vec(&*self, standard())
            .expect("Failed to encode Transaction instance.");

        let mut hasher = sha2::Sha256::new();
        hasher.update(&id_bytes);
        self.id = hasher.finalize().to_vec();
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].tx_id.len() == 0 && self.inputs[0].out_idx == 0
    }

    pub fn coinbase_tx(to: String) -> Self {
        let tx_input = TxInput::new(Vec::default(), 0, String::from("Coinbase Transaction"));
        let tx_output = TxOutput::new(100, to.clone());
        let tx = Transaction {
            id: Vec::default(),
            inputs: vec![tx_input],
            outputs: vec![tx_output],
        };

        tx
    }

    pub fn new(
        from: &str,
        to: &str,
        amount: u128,
        blockchain: &mut Blockchain,
    ) -> Self {
        let (accumulated, valid_outputs) =
            blockchain
                .find_spendable_outputs(amount, from)
                .expect(&format!(
                    "Address {} does'nt have enough money!",
                    from
                ));

        let mut inputs = vec![];

        for (tx_id, out_idxes) in valid_outputs {
            for out_idx in out_idxes {
                let tx_id_bytes = hex::decode(tx_id.clone()).unwrap();
                let input = TxInput::new(tx_id_bytes, out_idx, to.to_string());
                inputs.push(input);
            }
        }

        let mut outputs = vec![];

        let to_output = TxOutput::new(amount, to.to_string());
        outputs.push(to_output);

        if accumulated > amount {
            let remain_output = TxOutput::new(accumulated - amount, from.to_string());
            outputs.push(remain_output);
        }

        let tx = Transaction {
            id: vec![],
            inputs: inputs,
            outputs: outputs,
        };

        tx
    }
}
