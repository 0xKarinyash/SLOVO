use crate::aarch64::regs::Register;

pub fn parse_cyrillic_nums(s: &str) -> u16 {
    let s = s.trim();
    let digit_parts = if let Some(rest) = s.strip_prefix("0х") { 
        rest
    } else {
        return s.parse().expect("Лукавое число");
    };

    let mut res: u16 = 0;
    for c in digit_parts.chars() {
        let digit = match c {
            '0'..='9' => c as u16 - '0' as u16,
            'А' => 10,
            'Б' => 11,
            'В' => 12, 
            'Г' => 13,
            'Д' => 14,
            'Е' => 15,
            _ => panic!("Бред в числе: {c}"),
        };
        res = (res << 4) | digit;
    }
    res
}

pub fn parse_regs(s: &str) -> Register {
    let s = s.trim();
    let num_str: String = s.chars().skip(1).collect();
    let num = num_str.parse::<u8>().expect("Неизвестный помысел");
    Register::P(num)
}

pub fn clean_line(line: &str) -> String {
    line.split(';').next().unwrap_or("").trim().to_string()
}

pub fn extract_string(line: &str) -> String {
    let start = line.find('"').unwrap() + 1;
    let end = line.rfind('"').unwrap();
    let line = line.replace("\\n", "\n");
    line[start..end].to_string()
}