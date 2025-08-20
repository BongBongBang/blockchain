use crate::blockchain::Blockchain;

pub struct Transaction<'a> {
    pub id: Vec<u8>,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub blockchain: &'a Blockchain,
}

#[derive(Debug)]
pub struct TxInput {
    pub tx_id: Vec<u8>,
    pub out_idx: u32,
    pub sig: String,
}

#[derive(Debug)]
pub struct TxOutput {
    pub amount: u128,
    pub pub_key: String,
}

impl TxOutput {
    pub fn new(amount: u128, pub_key: String) -> Self {
        TxOutput { amount, pub_key }
    }
    // 判断当前output是否归属pub_key
    pub fn belongs(&self, pub_key: &'static str) -> bool {
        self.pub_key == pub_key
    }
}

impl TxInput {
    pub fn new(tx_id: Vec<u8>, out_idx: u32, sig: String) -> Self {
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

impl<'a> Transaction<'a> {
    // todo:
    pub fn set_id(&self) {}

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].tx_id.len() == 0 && self.inputs[0].out_idx == 0
    }

    pub fn coinbase_tx(to: String, chain: &'a Blockchain) -> Self {
        let tx_input = TxInput::new(Vec::default(), 0, String::from("Coinbase Transaction"));
        let tx_output = TxOutput::new(100, to.clone());
        let tx = Transaction {
            id: Vec::default(),
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            blockchain: chain,
        };
        tx
    }

    pub fn new(from: String, to: String, amount: u128, chain: &'a Blockchain) -> Self {
        let tx = Transaction {
            id: vec![],
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            blockchain: chain,
        };

        tx
    }
}
