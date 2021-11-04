// SPDX-License-Identifier: MIT

use std::{env, fs, process};

use romulan::amd::{
    Rom,
    directory::Directory,
};

fn print_directory(data: &[u8], address: u64, indent: usize) {
    //TODO: optimize
    let mut padding = String::with_capacity(indent);
    for i in 0..indent {
        padding.push(' ');
    }
    let offset = (address & 0xFFFFFF) as usize;
    match Directory::new(&data[offset..]) {
        Ok(Directory::Bios(directory)) => {
            println!("{}* {:#X}: {:X?}", padding, address, directory.header());
            for entry in directory.entries() {
                println!("{}  * {:X?}", padding, entry);
                if entry.kind == 0x70 {
                    print_directory(data, entry.source, indent + 4);
                }
            }
        },
        Ok(Directory::BiosCombo(combo)) => {
            println!("{}* {:#X}: {:X?}", padding, address, combo.header());
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4);
            }
        },
        Ok(Directory::BiosLevel2(directory)) => {
            println!("{}* {:#X}: {:X?}", padding, address, directory.header());
            for entry in directory.entries() {
                println!("{}  * {:X?}", padding, entry);
            }
        },
        Ok(Directory::Psp(directory)) => {
            println!("{}* {:#X}: {:X?}", padding, address, directory.header());
            for entry in directory.entries() {
                println!("{}  * {:X?}", padding, entry);
                if entry.kind == 0x40 {
                    print_directory(data, entry.value, indent + 4);
                }
            }
        },
        Ok(Directory::PspCombo(combo)) => {
            println!("{}* {:#X}: {:X?}", padding, address, combo.header());
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4);
            }
        },
        Ok(Directory::PspLevel2(directory)) => {
            println!("{}* {:#X}: {:X?}", padding, address, directory.header());
            for entry in directory.entries() {
                println!("{}  * {:X?}", padding, entry);
            }
        },
        Err(err) => {
            println!("{}* {:#X}: failed to load directory: {}", padding, address, err);
        }
    }
}

fn main() {
    let file = if let Some(file) = env::args().nth(1) {
        file
    } else {
        eprintln!("used_regions <file>");
        process::exit(1);
    };

    let data = fs::read(file).unwrap();

    let rom = Rom::new(&data).unwrap();

    let signature = rom.signature();
    println!("{:#X?}", signature);

    print_directory(&data, signature.psp as u64, 0);
    print_directory(&data, signature.bios as u64, 0);
}
