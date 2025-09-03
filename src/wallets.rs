use std::{collections::HashMap, fs::File, io::Write, path::Path};

use bincode::{Decode, Encode, config::standard};

use crate::wallet::Wallet;
use std::fs;
#[derive(Encode, Decode)]
pub struct Wallets {
    pub wallets: HashMap<String, Wallet>,
}

const WALLET_FILE: &str = "./tmp/wallets.data";

impl Wallets {
    pub fn new() -> Self {
        let wallets_map = HashMap::new();

        let mut wallets = Wallets {
            wallets: wallets_map,
        };
        // 加载本地的钱包文件
        wallets.load_file();

        wallets
    }

    pub fn add_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.address();
        self.wallets.insert(address.clone(), wallet);

        address
    }

    /// 获取所有的钱包地址引用
    ///
    /// # Arguments
    ///
    /// - `&'a self` (`undefined`) - Wallets
    ///
    /// # Returns
    ///
    /// - `Vec<&'a str>` - 钱包地址引用列表
    pub fn get_all_addresses<'a>(&'a self) -> Vec<&'a str> {
        let keys = self.wallets.keys().map(|item| item.as_str()).collect();
        keys
    }

    /// 获取一个钱包对象
    ///
    /// # Arguments
    ///
    /// - `&'a self` (`undefined`) - Wallet
    /// - `address` (`&str`) - Wallet's address
    ///
    /// # Returns
    ///
    /// - `Option<&'a Wallet>` - Wallet instance
    pub fn get_wallet<'a>(&'a self, address: &str) -> Option<&'a Wallet> {
        self.wallets.get(address)
    }

    pub fn get_wallet_mut<'a>(&'a mut self, address: &str) -> Option<&'a mut Wallet> {
        self.wallets.get_mut(address)
    }

    fn load_file(&mut self) {
        if let Ok(data) = fs::read(WALLET_FILE) {
            if let Ok(decoded) = bincode::decode_from_slice::<Wallets, _>(&data, standard()) {
                let wallets = decoded.0.wallets;
                self.wallets = wallets;
            }
        }
    }

    fn save_file(&self) {
        let bytes = bincode::encode_to_vec(&self, standard()).unwrap();
        let path = Path::new(WALLET_FILE);
        if let Some(parent_path) = path.parent() {
            let _ = fs::create_dir_all(parent_path).expect("创建钱包目录地址失败!");
        }
        let mut file = File::create(WALLET_FILE).unwrap();
        let _ = file.write_all(&bytes).expect("持久化钱包数据失败！");
    }
}
