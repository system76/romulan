// SPDX-License-Identifier: MIT

use std::{env, fs, process};

use romulan::amd::Rom;

fn main() {
    let file = if let Some(file) = env::args().nth(1) {
        file
    } else {
        eprintln!("used_regions <file>");
        process::exit(1);
    };

    let data = fs::read(file).unwrap();

    let rom = Rom::new(&data).unwrap();

    println!("{:#X?}", rom.signature());
}
