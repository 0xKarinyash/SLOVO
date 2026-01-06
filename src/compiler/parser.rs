use std::collections::HashMap;

use crate::aarch64::slasm::SlovoAsm;
use crate::aarch64::commands::Command;
use crate::compiler::utils::{*};

pub struct SlParser {
    labels: HashMap<String, usize>,
    curr_offset: usize,
}

impl SlParser {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            curr_offset: 0,
        }
    }

    fn first_pass_parse_line(&mut self, line: &str) {
        let line = clean_line(line);
        if line.is_empty() { return; }
        
        if line.ends_with(':') {
            let label_name = line.trim_end_matches(':').to_string();
            self.labels.insert(label_name, self.curr_offset);
        } else if line.starts_with("ОТМЕРЬ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let size: usize = parse_cyrillic_nums(parts[1]) as usize;

            self.curr_offset += (size + 3) & !3;
        } else if line.starts_with("СЛОВО") {
            let content = extract_string(&line);
            self.curr_offset += (content.len() + 3) & !3;  
        } else if line.starts_with("ВЛОЖИ") {
            let path = extract_string(&line);
            let data = std::fs::read(&path).expect(&format!("Свиток не обретен: {}", path));
            self.curr_offset += (data.len() + 3) & !3; 
        } else {
            self.curr_offset += 4;
        }
    }

    fn second_pass_parse_line(&mut self, line: &str, asm: &mut SlovoAsm) {
        let line = clean_line(line);
        if line.is_empty() || line.ends_with(':') { return; }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let mnemonic = parts[0];

        match mnemonic {
            // база
            "ПОЛОЖИ" => {
                let reg = parse_regs(parts[1].trim_matches(','));
                let val = parse_cyrillic_nums(parts[2]);
                asm.write(Command::Mov { reg, val });
                self.curr_offset += 4;
            },
            "УКАЖИ" => {
                let reg = parse_regs(parts[1].trim_matches(','));
                let label_name = parts[2];

                let target_addr = *self.labels.get(label_name)
                    .expect(&format!("Метка не найдена: {label_name}"));
                
                let relative = target_addr as i32 - self.curr_offset as i32;

                asm.write(Command::Adr { reg, offset: relative });
                self.curr_offset += 4;
            },
            "ДОЛОЖИ" => {
                asm.write(Command::Svc);
                self.curr_offset += 4;
            },
            "ВЕРНИСЬ" => { 
                asm.write(Command::Ret);
                self.curr_offset += 4;
            },

            // === Арифметика ===
            "ПРИБАВЬ" => {
                let rd = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                let third = parts[3];

                if third.starts_with('П') {
                    let rm = parse_regs(third);
                    asm.write(Command::Add { rd, rn, rm });
                } else {
                    let num = parse_cyrillic_nums(third) as u32;
                    asm.write(Command::Addi { rd, rn, num });
                }
                self.curr_offset += 4;
            },
            "ВЫЧТИ" => {
                let rd = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                let third = parts[3];

                if third.starts_with('П') {
                    let rm = parse_regs(third);
                    asm.write(Command::Sub { rd, rn, rm });
                } else {
                    let num = parse_cyrillic_nums(third) as u32; 
                    asm.write(Command::Subi { rd, rn, num });
                }
                self.curr_offset += 4;
            },
            "УМНОЖЬ" => {
                let rd = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                let rm = parse_regs(parts[3].trim_matches(','));

                asm.write(Command::Mul { rd, rn, rm });
                self.curr_offset += 4;
            },
            "РАЗДЕЛИ" => {
                let rd = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                let rm = parse_regs(parts[3].trim_matches(','));

                asm.write(Command::SDiv { rd, rn, rm });
                self.curr_offset += 4;
            },
            "РАЗНОСТЬ" => { // (E/X)OR
                let rd = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                let rm = parse_regs(parts[3].trim_matches(','));

                asm.write(Command::Eor { rd, rn, rm });
                self.curr_offset += 4;
            },

            // сравнение/ветвление
            "СРАВНИ" => {
                let rn = parse_regs(parts[1].trim_matches(','));
                let third = parts[2].trim_matches(',');

                if third.starts_with('П') {
                    let rm = parse_regs(third);
                    asm.write(Command::Cmp { rn, rm });
                } else {
                    let num = parse_cyrillic_nums(third) as u32;
                    asm.write(Command::Cmpi { rn, num });
                }
                self.curr_offset += 4;
            },
            "СТУПАЙ" => { // Безусловный прыжок (B)
                let label_name = parts[1];
                let target_addr = *self.labels.get(label_name)
                    .expect(&format!("Метка не найдена: {label_name}"));
                let offset = target_addr as i32 - self.curr_offset as i32;

                asm.write(Command::B { offset });
                self.curr_offset += 4;
            },
            
            // Условные прыжки (КОЛИ_...)
            // "КОЛИ_... СТУПАЙ МЕТКА"
            _ if mnemonic.starts_with("КОЛИ_") => {
                let cond = match mnemonic {
                    "КОЛИ_РАВНО" => 0x0,    // EQ (Equal)
                    "КОЛИ_НЕРАВНО" => 0x1,  // NE (Not Equal)
                    "КОЛИ_БОЛЬШЕ" => 0xC,   // GT (Greater Than)
                    "КОЛИ_МЕНЬШЕ" => 0xB,   // LT (Less Than)
                    "КОЛИ_ВЫШЕ" => 0xA,     // GE (Greater or Equal)
                    "КОЛИ_НИЖЕ" => 0xD,     // LE (Less or Equal)
                    _ => panic!("Неизвестное условие: {mnemonic}")
                };
                
                // parts[0] = КОЛИ_..., parts[1] = СТУПАЙ (пропускаем), parts[2] = МЕТКА
                let label_name = parts[2]; 
                let target_addr = *self.labels.get(label_name)
                    .expect(&format!("Метка не найдена: {label_name}"));
                let offset = target_addr as i32 - self.curr_offset as i32;

                asm.write(Command::Bcc { cond, offset });
                self.curr_offset += 4;
            },

            "ИЗЫМИ" => { // LDR (8 байт)
                let rt = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                
                asm.write(Command::Ldr { rt, rn });
                self.curr_offset += 4;
            },
            "ВВЕРГНИ" => { // STR (8 байт)
                let rt = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));
                
                asm.write(Command::Str { rt, rn });
                self.curr_offset += 4;
            },
            "ИЗЫМИ_БАЙТ" => { // LDRB
                let rt = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));

                asm.write(Command::Ldrb { rt, rn });
                self.curr_offset += 4;
            },
            "ВВЕРГНИ_БАЙТ" => { // STRB
                let rt = parse_regs(parts[1].trim_matches(','));
                let rn = parse_regs(parts[2].trim_matches(','));

                asm.write(Command::Strb { rt, rn });
                self.curr_offset += 4;
            }

            // остальное
            "ОТМЕРЬ" => {
                let size: usize = parse_cyrillic_nums(parts[1]) as usize;
                for _ in 0..size {
                    asm.code.push(0);
                }
               let padding = (4 - (size % 4)) % 4;
                for _ in 0..padding {
                    asm.code.push(0);
                }

                self.curr_offset += (size + 3) & !3;
            }

            "СЛОВО" => {
                let content = extract_string(&line);
                asm.ascii(&content);
                
                let padding = (4 - (content.len() % 4)) % 4;
                for _ in 0..padding {
                    asm.code.push(0);
                }

                self.curr_offset += (content.len() + 3) & !3;
            },

            "ВЛОЖИ" => {
                let path = extract_string(&line);
                let data = std::fs::read(&path).expect(&format!("Свиток не обретен: {}", path));
                
                asm.code.extend(&data);

                let padding = (4 - (data.len() % 4)) % 4;
                for _ in 0..padding {
                    asm.code.push(0);
                }

               self.curr_offset += (data.len() + 3) & !3;
            }
            _ => panic!("Неизвестный указ: {mnemonic}")
        }
    }
    
    pub fn compile(&mut self, source: &str, asm: &mut SlovoAsm) {
        self.curr_offset = 0;
        for line in source.lines() {
            self.first_pass_parse_line(line);
        }

        self.curr_offset = 0;
        for line in source.lines() {
            self.second_pass_parse_line(line, asm);
        }
    }
}