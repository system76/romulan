// SPDX-License-Identifier: MIT

use std::{env, fmt::Write, fs, path::PathBuf, process};

use romulan::amd::{
    directory::{BiosDirectoryEntry, Directory, PspDirectoryEntry},
    Rom,
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

fn print_bios_dir_entry(entry: &BiosDirectoryEntry, padding: &str) {
    let BiosDirectoryEntry {
        kind,
        region_kind,
        flags,
        sub_program,
        ..
    } = entry;
    let size = entry.size;
    let source = entry.source;
    let destination = entry.destination;
    let desc = entry.description();
    println!("{padding}  * Type {kind:02X} Region {region_kind:02X} Flags {flags:02X} SubProg {sub_program:02X} Size {size:08X} Source {source:016X} Dest {destination:016X}: {desc}");
}

fn print_psp_dir_entry(entry: &PspDirectoryEntry, padding: &str) {
    let PspDirectoryEntry {
        kind,
        sub_program,
        rom_id,
        ..
    } = entry;
    let size = entry.size;
    let value = entry.value;
    let desc = entry.description();
    println!("{padding}  * Type {kind:02X} SubProg {sub_program:02X} Rom {rom_id:02X} Size {size:08X} Value {value:016X}: {desc}");
}

// FIXME: DO NOT HARDCODE THIS!!!
// this needs to be per flash part size; define enum etc
const ADDR_MASK: u64 = 0x00FF_FFFF;

fn print_directory(data: &[u8], address: u64, indent: usize, export_opt: Option<&PathBuf>) {
    let mut padding = String::with_capacity(indent);
    for i in 0..indent {
        padding.push(' ');
    }
    let offset = (address & ADDR_MASK) as usize;
    match Directory::new(&data[offset..]) {
        Ok(Directory::Bios(directory)) => {
            println!("{padding}* {address:#X}: BIOS Directory");
            for entry in directory.entries() {
                print_bios_dir_entry(entry, &padding);
                if let Some(export) = export_opt {
                    let name = format!(
                        "BIOS/Level1/Type{:02X}_Region{:02X}_Flags{:02X}_SubProg{:02X}_{}",
                        entry.kind,
                        entry.region_kind,
                        entry.flags,
                        entry.sub_program,
                        entry.description().replace(" ", "_")
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
                        }
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
        }
        Ok(Directory::BiosCombo(combo)) => {
            println!("{}* {:#X}: BIOS Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4, export_opt);
            }
        }
        Ok(Directory::BiosLevel2(directory)) => {
            println!("{}* {:#X}: BIOS Level 2 Directory", padding, address);
            for entry in directory.entries() {
                print_bios_dir_entry(entry, &padding);
                if let Some(export) = export_opt {
                    let name = format!(
                        "BIOS/Level2/Type{:02X}_Region{:02X}_Flags{:02X}_SubProg{:02X}_{}",
                        entry.kind,
                        entry.region_kind,
                        entry.flags,
                        entry.sub_program,
                        entry.description().replace(" ", "_")
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
                        }
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
            }
        }
        Ok(Directory::Psp(directory)) => {
            println!("{}* {:#X}: PSP Directory", padding, address);
            for entry in directory.entries() {
                print_psp_dir_entry(entry, &padding);
                if let Some(export) = export_opt {
                    let name = format!(
                        "PSP/Level1/Type{:02X}_SubProg{:02X}_Rom{:02X}_{}",
                        entry.kind,
                        entry.sub_program,
                        entry.rom_id,
                        entry.description().replace(" ", "_")
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
                        }
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
        }
        Ok(Directory::PspCombo(combo)) => {
            println!("{}* {:#X}: PSP Combo Directory", padding, address);
            for entry in combo.entries() {
                println!("{}  * {:X?}", padding, entry);
                print_directory(data, entry.directory, indent + 4, export_opt);
            }
        }
        Ok(Directory::PspLevel2(directory)) => {
            println!("{}* {:#X}: PSP Level 2 Directory", padding, address);
            for entry in directory.entries() {
                print_psp_dir_entry(entry, &padding);
                if let Some(export) = export_opt {
                    let name = format!(
                        "PSP/Level2/Type{:02X}_SubProg{:02X}_Rom{:02X}_{}",
                        entry.kind,
                        entry.sub_program,
                        entry.rom_id,
                        entry.description().replace(" ", "_")
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
                        }
                        Err(err) => {
                            fs::write(dir.join("error"), err)
                                .expect(&format!("failed to write '{}/error'", name));
                        }
                    };
                }
            }
        }
        Err(err) => {
            println!(
                "{}* {:#X}: failed to load directory: {}",
                padding, address, err
            );
        }
    }
}

const DIR_UNSET: u32 = 0xffff_ffff;

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
    let efs = rom.efs();
    println!("{efs:#X?}");

    let dirs = [
        efs.psp_legacy,
        efs.psp,
        efs.bios,
        efs.bios_17_00_0f,
        efs.bios_17_10_1f,
        efs.bios_17_30_3f_19_00_0f,
    ];
    dirs.iter().for_each(|d| {
        if *d != DIR_UNSET {
            print_directory(&data, *d as u64, 0, export_opt.as_ref())
        }
    });
}
