// SPDX-License-Identifier: MIT

use std::{env, fs, process};

use romulan::amd::{
    Rom,
    directory::Directory,
};

fn directory(data: &[u8], address: u32) {
    let offset = (address & 0xFFFFFF) as usize;
    match Directory::new(&data[offset..]) {
        Ok(Directory::Bios(directory)) => {
            println!("{:X?}", directory.header());
            for entry in directory.entries() {
                println!("{:X?}", entry);
            }
        },
        Ok(Directory::BiosCombo(combo)) => {
            println!("{:X?}", combo.header());
            for entry in combo.entries() {
                println!("{:X?}", entry);
                directory(data, entry.directory as u32);
            }
        },
        Ok(Directory::Psp(directory)) => {
            println!("{:X?}", directory.header());
            for entry in directory.entries() {
                println!("{:X?}", entry);
            }
        },
        Ok(Directory::PspCombo(combo)) => {
            println!("{:X?}", combo.header());
            for entry in combo.entries() {
                println!("{:X?}", entry);
                directory(data, entry.directory as u32);
            }
        },
        Err(err) => {
            println!("Failed to load directory: {}", err);
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

    directory(&data, signature.psp);
    directory(&data, signature.bios);
}
