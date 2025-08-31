use base58::ToBase58;
use bincode::{
    BorrowDecode, Decode, Encode,
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use k256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng, ecdsa::VerifyingKey};
use k256::ecdsa::signature::Verifier;
use sha2::{Digest, Sha256};

const VERSION: u8 = 0;
const CHECK_SUM_LENGTH: usize = 4;

#[derive(Debug)]
pub struct Wallet {
    pub_key: Vec<u8>,
    priv_key: SigningKey,
}

impl Wallet {
    pub fn new() -> Self {
        let (signing_key, pub_key) = Wallet::new_key_pair();

        Wallet {
            pub_key,
            priv_key: signing_key,
        }
    }

    /*
     * 生成签名密钥对
     */
    fn new_key_pair() -> (SigningKey, Vec<u8>) {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        // SEC1/EncodedPoint compressed
        let pub_key: Vec<u8> = verifying_key.to_encoded_point(true).to_bytes().to_vec();

        (signing_key, pub_key)
    }

    /*
     * 获取Wallet的Addres
     */
    pub fn address(&self) -> String {
        let pub_key_hashed = Wallet::hash_pub_key(&self.pub_key);
        let mut ver_pubkey = vec![VERSION];
        // 追加pub_key哈希
        ver_pubkey.extend(pub_key_hashed);

        let checksum = Wallet::checksum(&ver_pubkey);

        // 追加checksum
        ver_pubkey.extend(checksum);

        ver_pubkey.to_base58()
    }

    /*
     * 对{version}{pub_key_hashed}进行两次Sha256哈希处理
     */
    fn checksum(ver_pubkey: &[u8]) -> Vec<u8> {
        Sha256::digest(Sha256::digest(ver_pubkey)).to_vec()[..CHECK_SUM_LENGTH].to_vec()
    }

    /*
     * 对pub_key进行Sha256 \ Ripemd160
     */
    pub fn hash_pub_key(pub_key: &Vec<u8>) -> Vec<u8> {
        let sha256_hashed = Sha256::digest(pub_key);

        let ripemd_hashed = ripemd::Ripemd160::digest(sha256_hashed);

        ripemd_hashed.to_vec()
    }
}

impl Encode for Wallet {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let bytes = self.priv_key.to_bytes();
        bytes.encode(encoder)?;

        self.pub_key.encode(encoder)
    }
}

impl<Context> Decode<Context> for Wallet {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let priv_key_bytes = Vec::<u8>::decode(decoder)?;
        let pub_key = Vec::<u8>::decode(decoder)?;

        if let Ok(priv_key) = SigningKey::from_slice(&priv_key_bytes) {
            return Ok(Wallet { pub_key, priv_key });
        }
        Err(DecodeError::OtherString(String::from(
            "根据字节反序列化SigningKey失败!",
        )))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Wallet {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let priv_key_bytes = Vec::<u8>::decode(decoder)?;
        let pub_key = Vec::<u8>::decode(decoder)?;
        if let Ok(priv_key) = SigningKey::from_slice(&priv_key_bytes) {
            return Ok(Wallet { pub_key, priv_key });
        }
        Err(DecodeError::OtherString(String::from(
            "根据字节反序列化SigningKey失败!",
        )))
    }
}
