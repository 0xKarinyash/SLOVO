use crate::aarch64::commands::Command;

pub struct SlovoAsm {
    pub code: Vec<u8>
}

impl SlovoAsm { 
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn curr_offset(&self) -> usize { 
        self.code.len()
    }


    pub fn write(&mut self, cmd: Command) {
        let code = cmd.encode();
        let bytes = code.to_le_bytes();
        self.code.extend_from_slice(&bytes);
    }

    pub fn ascii(&mut self, text: &str) -> usize {
        let start_addr = self.curr_offset();        
        self.code.extend_from_slice(text.as_bytes());
        start_addr
    }
}