use crate::aarch64::regs::Register;

pub enum Command {
    Mov  { reg: Register, val: u16 },
    Adr  { reg: Register, offset: i32 },
    Svc,
    Ret,
    Add  { rd: Register, rn: Register, rm: Register },
    Addi { rd: Register, rn: Register, num: u32 },
    Sub  { rd: Register, rn: Register, rm: Register },
    Subi { rd: Register, rn: Register, num: u32 },
    Mul  { rd: Register, rn: Register, rm: Register },
    SDiv { rd: Register, rn: Register, rm: Register },
    Cmp  { rn: Register, rm: Register },
    Cmpi { rn: Register, num: u32 },
    B    { offset: i32 },
    Bcc  { cond: u8, offset: i32 }, // Bcond
    Ldr  { rt: Register, rn: Register },
    Str  { rt: Register, rn: Register },
    Eor  { rd: Register, rn: Register, rm: Register },
    Ldrb { rt: Register, rn: Register },
    Strb { rt: Register, rn: Register }
}

impl Command {
    pub fn encode(&self) -> u32 {
        match self {
            Command::Mov { reg, val } => {
                let imm = (*val as u32) << 5;
                0xD2800000 | imm | reg.to_u32()
            }
            Command::Adr { reg, offset } => {
                let off = *offset as u32;
                let immlo = (off & 0b11) << 29;
                let immhi = ((off >> 2) & 0x7FFFF) << 5; // когда я это писала 3 часа назад я думала "нифига я умная" а щас я не могу это прочитать T_T
                0x10000000 | immlo | immhi | reg.to_u32()
            }
            Command::Svc => {
                0xD4000001
            }
            Command::Ret => {
                0xD65F03C0
            }
            Command::Add { rd, rn, rm } => {
                0x8B000000 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Addi { rd, rn, num } => {
                let imm12 = (num & 0xFFF) << 10; 
                0x91000000 | imm12 | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Sub { rd, rn, rm } => {
                0xCB000000 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Subi { rd, rn, num } => {
                let imm12 = (num & 0xFFF) << 10; 
                0xD1000000 | imm12 | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Mul { rd, rn, rm } => {
                0x9B007C00 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::SDiv { rd, rn, rm } => {
                0x9AC00C00 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Cmp { rn, rm } => {
                0xEB000000 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | 0x1F // SUBS но пишем в помысел П31 (чтобы выкинуть)
            }
            Command::Cmpi { rn, num } => {
                let imm12 = (num & 0xFFF) << 10;
                0xF1000000 | imm12 | (rn.to_u32() << 5) | 0x1F
            }
            Command::B { offset } => {
                let imm26 = ((offset / 4) & 0x03FFFFFF) as u32;
                0x14000000 | imm26
            }
            Command::Bcc { cond, offset } => {
                let imm19 = (((offset / 4) & 0x7FFFF) as u32) << 5;
                let c = (cond & 0xF) as u32;
                0x54000000 | imm19 | c
            }
            Command::Ldr { rt, rn } => {
                0xF9400000 | (rn.to_u32() << 5) | rt.to_u32()
            }
            Command::Str { rt, rn } => {
                0xF9000000 | (rn.to_u32() << 5) | rt.to_u32()
            }
            Command::Eor { rd, rn, rm } => {
                0xCA000000 | (rm.to_u32() << 16) | (rn.to_u32() << 5) | rd.to_u32()
            }
            Command::Ldrb { rt, rn } => {
                0x39400000 | (rn.to_u32() << 5) | rt.to_u32()
            }
            Command::Strb { rt, rn } => {
                0x39000000 | (rn.to_u32() << 5) | rt.to_u32()
            }
        }
    }
}

