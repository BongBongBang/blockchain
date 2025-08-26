use k256::{ecdsa::{SigningKey, VerifyingKey}, elliptic_curve::rand_core::OsRng};

const VERSION: u8 = 0;
const CHECK_SUM_LENGTH: usize = 4;

pub struct Wallet {
    pub_key: Vec<u8>,
    priv_key: SigningKey
}

impl Wallet {

    pub fn new() -> Self {

    }

    fn new_key_pair() -> (SigningKey, VerifyingKey) {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key().
        (signing_key, verifying_key)
    }

    pub fn address(&self) -> String {
        ""
    }

    pub fn checksum() -> Vec<u8> {
        vec![]
    }

    pub fn hash_pub_key() -> Vec<u8> {
        vec![]
    }
}
