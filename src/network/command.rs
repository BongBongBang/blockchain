struct CmdHeader {
    pub ver: u8,
    pub len: u32,
    pub cmd: String,
}

pub trait Command {
    fn serialize(&self) -> Vec<u8>;
}

pub struct HeightCmd {
    pub header: CmdHeader,
    pub height: u8,
}

impl Command for HeightCmd {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}
