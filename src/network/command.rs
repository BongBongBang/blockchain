pub enum Cmd {
    Height,
}

impl Cmd {
    pub fn encode(&self) -> [u8; 2] {
        match self {
            Cmd::Height => [0u8, 1u8]
        }
    }
}


struct CmdHeader {
    pub ver: u8,
    pub len: u32,
    pub cmd: Cmd,
}

pub trait Command {
    fn serialize(&self) -> Vec<u8>;
}

pub struct HeightCmd {
    pub header: CmdHeader,
    pub height: u32,
}

impl Command for HeightCmd {
    fn serialize(&self) -> Vec<u8> {

        let mut result = vec![];

        let header = &self.header;

        // ver, 1 byte
        result.push(header.ver);
        // len, 4 bytes, cause HeightCmd only has a 'height' field, u32
        result.extend_from_slice(&header.len.to_be_bytes());
        // cmd, 2 bytes
        result.extend_from_slice(&header.cmd.encode());
        // `height` field
        result.extend_from_slice(&self.height.to_be_bytes());

        result 
    }
}
