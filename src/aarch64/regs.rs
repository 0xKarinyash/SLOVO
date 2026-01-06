#[derive(Debug, Clone, Copy)]
pub enum Register {
    P(u8), // Помысел П0...П30
}

impl Register {
    pub fn to_u32(&self) -> u32 {
        match self {
            Register::P(n) => *n as u32,
        }
    }
}