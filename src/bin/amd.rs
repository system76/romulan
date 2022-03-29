// SPDX-License-Identifier: MIT

use std::{env, fmt::Write, fs, path::PathBuf, process};

use romulan::amd::{
    Rom,
    directory::Directory,
};

fn hexdump(data: &[u8]) -> String {
    let mut s = String::new();
    for chunk in data.chunks(16) {
        for (i, b) in chunk.iter().enumerate() {
            if i != 0 {
                s.push(' ');
            }
            write!(s, "{:02X}", b).unwrap();
        }
        s.push('\n');
    }
    s
}

fn print_directory(data: &[u8], address: u64, indent: usize, export_opt: Option<&PathBuf>) {
    //TODO: optimize
    let mut padding = String::with_capacity(indent);
    for i in 0..indent {
        padding.push(' ');
    }
    let offset = (address & 0x1FFFFFF) as usize;
    match Directory::new(&data[offset..]) {
        Ok(Directory::Bios(directory)) => {
            println!("{}* {:#X}: BIOS Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} Region {:02X} Flags {:02X} SubProg {:02X} Size {:08X} Source {:016X} Dest {:016X}: {}", padding, entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.size, entry.source, entry.destination, entry.description());
                if let Some(export) = export_opt {
                    let name = format!(
                        "BIOS/Level1/Type{:02X}_Region{:02X}_Flags{:02X}_SubProg{:02X}_{}",
                        entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.description().replace(" ", "_")
                    );
                    let dir = export.join(&name);
                    if dir.exists() {
                        panic!("directory already exists '{}'", name);
                    }
                    fs::create_dir_all(&dir)
                        .expect(&format!("failed to create directory '{}'", name));
                    match entry.data(data) {
                        Ok(ok) => {
                            fs::write(dir.join("raw"), &ok)
                                .expect(&format!("failed to write '{}/raw'", name));
                            fs::write(dir.join("hex"), hexdump(&ok))
                                .expect(&format!("failed to write '{}/hex'", name));
                        },
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
                if entry.kind == 0x70 {
                    print_directory(data, entry.source, indent + 4, export_opt);
                }
            }
        },
        Ok(Directory::BiosCombo(combo)) => {
            println!("{}* {:#X}: BIOS Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4, export_opt);
            }
        },
        Ok(Directory::BiosLevel2(directory)) => {
            println!("{}* {:#X}: BIOS Level 2 Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} Region {:02X} Flags {:02X} SubProg {:02X} Size {:08X} Source {:016X} Dest {:016X}: {}", padding, entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.size, entry.source, entry.destination, entry.description());
                if let Some(export) = export_opt {
                    let name = format!(
                        "BIOS/Level2/Type{:02X}_Region{:02X}_Flags{:02X}_SubProg{:02X}_{}",
                        entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.description().replace(" ", "_")
                    );
                    let dir = export.join(&name);
                    if dir.exists() {
                        panic!("directory already exists '{}'", name);
                    }
                    fs::create_dir_all(&dir)
                        .expect(&format!("failed to create directory '{}'", name));
                    match entry.data(data) {
                        Ok(ok) => {
                            fs::write(dir.join("raw"), &ok)
                                .expect(&format!("failed to write '{}/raw'", name));
                            fs::write(dir.join("hex"), hexdump(&ok))
                                .expect(&format!("failed to write '{}/hex'", name));
                        },
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
            }
        },
        Ok(Directory::Psp(directory)) => {
            println!("{}* {:#X}: PSP Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} SubProg {:02X} Rom {:02X} Size {:08X} Value {:016X}: {}", padding, entry.kind, entry.sub_program, entry.rom_id, entry.size, entry.value, entry.description());
                if let Some(export) = export_opt {
                    let name = format!(
                        "PSP/Level1/Type{:02X}_SubProg{:02X}_Rom{:02X}_{}",
                        entry.kind, entry.sub_program, entry.rom_id, entry.description().replace(" ", "_")
                    );
                    let dir = export.join(&name);
                    if dir.exists() {
                        eprintln!("directory already exists '{}'", name);
                    }
                    fs::create_dir_all(&dir)
                        .expect(&format!("failed to create directory '{}'", name));
                    match entry.data(data) {
                        Ok(ok) => {
                            fs::write(dir.join("raw"), &ok)
                                .expect(&format!("failed to write '{}/raw'", name));
                            fs::write(dir.join("hex"), hexdump(&ok))
                                .expect(&format!("failed to write '{}/hex'", name));
                        },
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
                if entry.kind == 0x40 {
                    print_directory(data, entry.value, indent + 4, export_opt);
                }
            }
        },
        Ok(Directory::PspCombo(combo)) => {
            println!("{}* {:#X}: PSP Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4, export_opt);
            }
        },
        Ok(Directory::PspLevel2(directory)) => {
            println!("{}* {:#X}: PSP Level 2 Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} SubProg {:02X} Size {:08X} Value {:016X}: {}", padding, entry.kind, entry.sub_program, entry.size, entry.value, entry.description());
                if let Some(export) = export_opt {
                    let name = format!(
                        "PSP/Level2/Type{:02X}_SubProg{:02X}_Rom{:02X}_{}",
                        entry.kind, entry.sub_program, entry.rom_id, entry.description().replace(" ", "_")
                    );
                    let dir = export.join(&name);
                    if dir.exists() {
                        panic!("directory already exists '{}'", name);
                    }
                    fs::create_dir_all(&dir)
                        .expect(&format!("failed to create directory '{}'", name));
                    match entry.data(data) {
                        Ok(ok) => {
                            fs::write(dir.join("raw"), &ok)
                                .expect(&format!("failed to write '{}/raw'", name));
                            fs::write(dir.join("hex"), hexdump(&ok))
                                .expect(&format!("failed to write '{}/hex'", name));
                        },
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
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
        eprintln!("used_regions <file> [export]");
        process::exit(1);
    };

    let export_opt = if let Some(export) = env::args().nth(2) {
        let export = PathBuf::from(export);
        if export.exists() {
            fs::remove_dir_all(&export).expect("failed to clean export directory");
        }
        fs::create_dir(&export).expect("failed to create export directory");
        Some(export)
    } else {
        None
    };

    let data = fs::read(file).unwrap();

    let rom = Rom::new(&data).unwrap();

    let signature = rom.signature();
    println!("{:#X?}", signature);

    print_directory(&data, signature.psp as u64, 0, export_opt.as_ref());
    print_directory(&data, signature.bios as u64, 0, export_opt.as_ref());
}
