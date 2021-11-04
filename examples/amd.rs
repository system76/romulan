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
            println!("{}* {:#X}: BIOS Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} Region {:02X} Flags {:02X} SubProg {:02X} Size {:08X} Source {:08X} Dest {:016X}: {}", padding, entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.size, entry.source, entry.destination, entry.description());
                if entry.kind == 0x70 {
                    print_directory(data, entry.source, indent + 4);
                }
            }
        },
        Ok(Directory::BiosCombo(combo)) => {
            println!("{}* {:#X}: BIOS Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4);
            }
        },
        Ok(Directory::BiosLevel2(directory)) => {
            println!("{}* {:#X}: BIOS Level 2 Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} Region {:02X} Flags {:02X} SubProg {:02X} Size {:08X} Source {:08X} Dest {:016X}: {}", padding, entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.size, entry.source, entry.destination, entry.description());
            }
        },
        Ok(Directory::Psp(directory)) => {
            println!("{}* {:#X}: PSP Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} SubProg {:02X} Size {:08X} Value {:08X}: {}", padding, entry.kind, entry.sub_program, entry.size, entry.value, entry.description());
                if entry.kind == 0x40 {
                    print_directory(data, entry.value, indent + 4);
                }
            }
        },
        Ok(Directory::PspCombo(combo)) => {
            println!("{}* {:#X}: PSP Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4);
            }
        },
        Ok(Directory::PspLevel2(directory)) => {
            println!("{}* {:#X}: PSP Level 2 Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} SubProg {:02X} Size {:08X} Value {:08X}: {}", padding, entry.kind, entry.sub_program, entry.size, entry.value, entry.description());
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
