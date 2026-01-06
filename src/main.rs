use std::io::Read;

use clap::Parser;

use object::write::Object;
use object::{*};

mod aarch64;
mod compiler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Путь к свитку с песней (.слово)")]
    path: String,

    #[arg(short, long, help = "Имя творения")]
    output: Option<String>
}

fn main() {
    let args = Args::parse();
    let mut file = std::fs::File::open(args.path).expect("Не найдено священное писание!");
    let mut source = String::new();
    file.read_to_string(&mut source).expect("Не удалось прочесть священное писание!");

    let mut asm = aarch64::slasm::SlovoAsm::new();
    let mut parser = compiler::parser::SlParser::new();
    println!("Начинаю сборку..");
    
    parser.compile(&source, &mut asm);
    println!("Песень готова! Связываем...");


    let mut obj = Object::new(
        BinaryFormat::Elf,
        Architecture::Aarch64,
        Endianness::Little
    );

    let section_id = obj.add_section(vec![], b".text".to_vec(), SectionKind::Text); // vec![] пушто в elf пусто
    
    obj.append_section_data(section_id, &asm.code, 4); // выравнивание 4
    obj.add_symbol(object::write::Symbol {
        name: b"_start".to_vec(),
        value: 0, // смещение 0 от начала
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage, // глобальный символ
        weak: false,
        section: object::write::SymbolSection::Section(section_id),
        flags: SymbolFlags::None
    });

    let file_data = obj.write().expect("Не удалось записать песень в буффер!");
    let output_file = args.output.unwrap_or(String::from("песень.o"));
    std::fs::write(&output_file, file_data).expect("Не удалось записать песень на диск!");

    println!("Сотворен файл {output_file}!");

}
