use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Sub;

use base58::FromBase58;
use bincode::config::standard;
use bincode::{Decode, Encode};
use k256::EncodedPoint;
use k256::ecdsa;
use k256::ecdsa::Signature;
use k256::ecdsa::VerifyingKey;
use k256::ecdsa::signature::SignerMut;
use k256::ecdsa::signature::Verifier;
use sha2::Digest;
use sha2::Sha256;

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::wallets::Wallets;

#[derive(Debug, Encode, Decode, Clone)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct TxInput {
    pub tx_id: Vec<u8>,
    pub out_idx: usize,
    pub sig: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct TxOutput {
    pub amount: u128,
    pub pub_key_hash: Vec<u8>,
}

impl TxOutput {
    pub fn new(amount: u128, address: String) -> Self {
        let bytes = address
            .from_base58()
            .expect(&format!("Failed to decode address[{}] base58", address));

        let pub_key_hash = &bytes[1..bytes.len().sub(4)];
        TxOutput {
            amount,
            pub_key_hash: pub_key_hash.to_vec(),
        }
    }

    // 判断当前output是否归属pub_key
    pub fn belongs_to(&self, pub_key_hash: &[u8]) -> bool {
        &self.pub_key_hash == pub_key_hash
    }
}

impl TxInput {
    pub fn new(tx_id: Vec<u8>, out_idx: usize, pub_key: Vec<u8>, sig: Vec<u8>) -> Self {
        TxInput {
            tx_id,
            out_idx,
            sig,
            pub_key,
        }
    }

    // 判断当前input是否是pub_key的来源
    #[inline]
    pub fn spent_by(&self, pub_key_hash: &Vec<u8>) -> bool {
        &Wallet::hash_pub_key(&self.pub_key) == pub_key_hash
    }
}

impl AsRef<Transaction> for &mut Transaction {
    fn as_ref(&self) -> &Transaction {
        self
    }
}

impl Transaction {
    pub fn set_id(&mut self) {
        let id_bytes = bincode::encode_to_vec(&*self, standard())
            .expect("Failed to encode Transaction instance.");

        let id = sha2::Sha256::digest(&id_bytes);
        self.id = id.to_vec();
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].tx_id.len() == 0
    }

    /// 生成coinbase transaction
    ///
    /// # Arguments
    ///
    /// - `to` (`String`) - coin receiver
    ///
    /// # Returns
    ///
    /// - `Self` - Transaction
    /// ```
    pub fn coinbase_tx(to: String) -> Self {
        let tx_input = TxInput::new(
            Vec::default(),
            0,
            String::from("Coinbase").as_bytes().to_vec(),
            Vec::new(),
        );

        let tx_output = TxOutput::new(100, to);
        let tx = Transaction {
            id: Vec::default(),
            inputs: vec![tx_input],
            outputs: vec![tx_output],
        };

        tx
    }

    pub fn new(from: &str, to: &str, amount: u128, blockchain: &mut Blockchain) -> Self {
        let mut wallets = Wallets::new();

        {
            let to_wallet_op = wallets.get_wallet(to);
            if to_wallet_op.is_none() {
                panic!("不存在接收钱包: {}", to);
            }
        }

        let from_wallet_op = wallets.get_wallet_mut(from);
        if from_wallet_op.is_none() {
            panic!("不存在来源钱包: {}", from);
        }
        let from_wallet = from_wallet_op.unwrap();

        let (accumulated, valid_outputs) = blockchain
            .find_spendable_outputs(amount, &Wallet::hash_pub_key(&from_wallet.pub_key))
            .expect(&format!("Address [{}] does'nt have enough money!", from));

        let mut inputs = vec![];

        for (tx_id, out_idxes) in valid_outputs {
            for out_idx in out_idxes {
                let tx_id_bytes = hex::decode(tx_id.clone()).unwrap();
                let input = TxInput::new(
                    tx_id_bytes,
                    out_idx,
                    from_wallet.pub_key.clone(),
                    Vec::default(),
                );
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

        let mut tx = Transaction {
            id: vec![],
            inputs: inputs,
            outputs: outputs,
        };

        blockchain.sign_transaction(&mut tx, &mut from_wallet.priv_key);

        tx
    }

    /// 生成干净的只用于生成签名的Transaction
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`) -
    ///
    /// # Returns
    ///
    /// - `Self` - Sign ready Transaction
    pub fn trimmed_copy(&self) -> Self {
        let inputs = self
            .inputs
            .iter()
            .map(|input| {
                return TxInput {
                    tx_id: input.tx_id.clone(),
                    out_idx: input.out_idx,
                    sig: Vec::default(),
                    pub_key: Vec::default(),
                };
            })
            .collect();

        let outputs = self
            .outputs
            .iter()
            .map(|output| {
                return TxOutput {
                    amount: output.amount,
                    pub_key_hash: output.pub_key_hash.clone(),
                };
            })
            .collect();

        Transaction {
            id: self.id.clone(),
            inputs,
            outputs,
        }
    }

    /// 给当前Tranasction签名
    ///
    /// # Arguments
    ///
    /// - `signingKey` (`ecdsa`) - 签名Key
    /// - `prevTxs` (`HashMap<String, Transaction>`) - 签名用到的当前Tx的inputs关联的Tx.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = sign();
    /// ```
    pub fn sign(
        &mut self,
        signing_key: &mut ecdsa::SigningKey,
        mut prev_txs: HashMap<String, Transaction>,
    ) {
        let mut tx_copy = self.trimmed_copy();

        // 检验所有的input 关联Tx
        for input in &tx_copy.inputs {
            let tx_id = hex::encode(&input.tx_id);
            if !prev_txs.contains_key(&tx_id) {
                panic!("未找到Input关联的目标Tx: {}", tx_id);
            }
        }

        for idx in 0..tx_copy.inputs.len() {
            // 缩短borrow of mut tx_copy的生命周期。因为input是从mut的tx_copy出来的mut input
            // 寻找到input引用的Tx#Output并赋值pub_key_hash
            {
                let input = tx_copy.inputs.get_mut(idx).unwrap();
                let tx_id = hex::encode(&input.tx_id);
                let prev_tx = prev_txs.get_mut(&tx_id).unwrap();
                input.pub_key = prev_tx.outputs.remove(input.out_idx).pub_key_hash;
            }

            let hash = Sha256::digest(bincode::encode_to_vec(&tx_copy, standard()).unwrap());

            let signature: ecdsa::Signature = signing_key.sign(&hash);
            let input = tx_copy.inputs.get_mut(idx).unwrap();
            input.sig = signature.to_vec();
        }
    }

    /// 校验Transaction是否
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`)
    ///
    /// # Returns
    ///
    /// - `bool` - 结果
    pub fn verity(&self, mut prev_txs: HashMap<String, Transaction>) -> bool {
        if self.is_coinbase() {
            return true;
        }

        for input in &self.inputs {
            let tx_id = hex::encode(&input.tx_id);
            if !prev_txs.contains_key(&tx_id) {
                panic!("未找到Input关联的Tx: {}", tx_id);
            }
        }

        let mut tx_copy = self.trimmed_copy();

        for (idx, input) in self.inputs.iter().enumerate() {
            let tx_id = hex::encode(&input.tx_id);
            let prev_tx = prev_txs.get_mut(&tx_id).unwrap();
            tx_copy.inputs[idx].pub_key = prev_tx.outputs.remove(input.out_idx).pub_key_hash;

            let hash = Sha256::digest(bincode::encode_to_vec(&tx_copy, standard()).unwrap());

            let encoded_point = EncodedPoint::from_bytes(&input.pub_key).unwrap();
            let verifying_key = VerifyingKey::from_encoded_point(&encoded_point).unwrap();
            let raw_bytes: [u8; 64] = input.sig.clone().try_into().unwrap();
            let signature = Signature::from_bytes(&raw_bytes.into()).unwrap();
            if let Err(_) = verifying_key.verify(&hash, &signature) {
                return false;
            }
        }

        true
    }
}

#[rustfmt::skip]
impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        f.write_str(&format!("--- Transaction {}:", hex::encode(&self.id)));
        
        for (idx, input) in self.inputs.iter().enumerate() {
            f.write_str(&format!("    Input {:?}:", idx));
            f.write_str(&format!("        Tx ID:      {}", hex::encode(&input.tx_id)));
            f.write_str(&format!("        Out:        {:?}", input.out_idx));
            f.write_str(&format!("        Signature:  {}", hex::encode(&input.sig)));
            f.write_str(&format!("        Scipt:      {}", hex::encode(&input.pub_key)));
        }

        for (idx, output) in self.outputs.iter().enumerate() {
            f.write_str(&format!("    Output  {}:", idx));
            f.write_str(&format!("        Value: {}", output.amount));
            f.write_str(&format!("        Script: {}", hex::encode(&output.pub_key_hash)));
        }

        Ok(())
    }
}
