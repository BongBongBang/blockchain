use crate::blockchain;

pub struct CommandLine<'a> {
    blockchain: &'a mut blockchain::Blockchain
}

impl<'a> CommandLine<'a> {
    pub fn add_block(&mut self, data : String) {
        self.blockchain.add_block(data); 
    }
}
