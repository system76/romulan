// SPDX-License-Identifier: MIT

use std::{env, fs, path::PathBuf, process};

use romulan::amd::{
    Rom,
    directory::Directory,
};

fn print_directory(data: &[u8], address: u64, indent: usize, export_opt: Option<&PathBuf>) {
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
                if let Some(export) = export_opt {
                    let dir = export.join("bios").join("l1");
                    fs::create_dir_all(&dir).expect("failed to create bios/l1 export directory");
                    let file = dir.join(format!("Type_{:02X}_Region_{:02X}_Flags_{:02X}_SubProg_{:02X}_{}", entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.description().replace(" ", "_")));
                    let data = format!("Size {:08X}", entry.size);
                    fs::write(&file, &data).expect("failed to write bios/l1 export file");
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
                println!("{}  * Type {:02X} Region {:02X} Flags {:02X} SubProg {:02X} Size {:08X} Source {:08X} Dest {:016X}: {}", padding, entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.size, entry.source, entry.destination, entry.description());
                if let Some(export) = export_opt {
                    let dir = export.join("bios").join("l2");
                    fs::create_dir_all(&dir).expect("failed to create bios/l2 export directory");
                    let file = dir.join(format!("Type_{:02X}_Region_{:02X}_Flags_{:02X}_SubProg_{:02X}_{}", entry.kind, entry.region_kind, entry.flags, entry.sub_program, entry.description().replace(" ", "_")));
                    let data = format!("Size {:08X}", entry.size);
                    fs::write(&file, &data).expect("failed to write bios/l2 export file");
                }
            }
        },
        Ok(Directory::Psp(directory)) => {
            println!("{}* {:#X}: PSP Directory", padding, address);
            for entry in directory.entries() {
                println!("{}  * Type {:02X} SubProg {:02X} Size {:08X} Value {:08X}: {}", padding, entry.kind, entry.sub_program, entry.size, entry.value, entry.description());
                if let Some(export) = export_opt {
                    let dir = export.join("psp").join("l1");
                    fs::create_dir_all(&dir).expect("failed to create psp/l1 export directory");
                    let file = dir.join(format!("Type_{:02X}_SubProg_{:02X}_{}", entry.kind, entry.sub_program, entry.description().replace(" ", "_")));
                    let data = if entry.size == 0xFFFFFFFF {
                        format!("Value {:016X}", entry.value)
                    } else {
                        format!("Size {:08X}", entry.size)
                    };
                    fs::write(&file, &data).expect("failed to write psp/l1 export file");
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
                println!("{}  * Type {:02X} SubProg {:02X} Size {:08X} Value {:08X}: {}", padding, entry.kind, entry.sub_program, entry.size, entry.value, entry.description());
                if let Some(export) = export_opt {
                    let dir = export.join("psp").join("l2");
                    fs::create_dir_all(&dir).expect("failed to create psp/l2 export directory");
                    let file = dir.join(format!("Type_{:02X}_SubProg_{:02X}_{}", entry.kind, entry.sub_program, entry.description().replace(" ", "_")));
                    let data = if entry.size == 0xFFFFFFFF {
                        format!("Value {:016X}", entry.value)
                    } else {
                        format!("Size {:08X}", entry.size)
                    };
                    fs::write(&file, &data).expect("failed to write psp/l2 export file");
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
