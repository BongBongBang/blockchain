use bincode::{config, Decode, Encode};

pub enum Cmd {
    Height,
    Unknown,
}

impl Cmd {
    pub fn encode(&self) -> [u8; 2] {
        match self {
            Cmd::Height => [0u8, 1u8],
            Cmd::Unknown => [255u8, 255u8],
        }
    }

    pub fn decode(bytes: [u8; 2]) -> Cmd {
        let seri: u16 = u16::from_be_bytes(bytes);
        match seri {
            1u16 => Cmd::Height,
            _ => Cmd::Unknown
        }
    }
}

/// Cmd header
///
/// # Fields
///
/// - `ver` (`u8`) - ver, 1 byte
/// - `len` (`u32`) - length of body, 4 bytes.
/// - `cmd` (`Cmd`) - cmd type, 2 bytes.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let s = CmdHeader {
///     ver: value,
///     len: value,
///     cmd: value,
/// };
/// ```
pub trait Command {
    fn serialize(&self) -> Vec<u8>;

    fn version(&self) -> u8;
}

#[derive(Encode, Decode)]
pub struct HeightCmd {
    pub height: u32,
}

impl Command for HeightCmd {

    fn serialize(&self) -> Vec<u8> {

        let payload = bincode::encode_to_vec(self, config::standard()).unwrap();

        let mut result = vec![];
        let ver = self.version();
        // ver, 1 byte
        result.push(ver);
        // todo!
        let length = payload.len() as u32;
        // len, 4 bytes, cause HeightCmd only has a 'height' field, u32
        result.extend_from_slice(&length.to_be_bytes());
        // cmd, 2 bytes
        result.extend_from_slice(&Cmd::Height.encode());
        // `height` field
        result.extend_from_slice(&payload);

        result
    }
    
    fn version(&self) -> u8 {
        1u8
    }
}
