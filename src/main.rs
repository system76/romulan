// SPDX-License-Identifier: MIT

use romulan::intel::{Rom, BiosFile, BiosSection, BiosSections, BiosVolume, BiosVolumes};
use romulan::intel::{section, volume};
use std::{env, fs, io, mem, process, thread};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use uefi::guid::SECTION_LZMA_COMPRESS_GUID;

fn dump_lzma(compressed_data: &[u8], padding: &str) {
    // For some reason, xz2 does not work with this data
    let mut child = Command::new("xz")
        .arg("--decompress")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let data = {
        let mut stdout = child.stdout.take().unwrap();
        let read_thread = thread::spawn(move || -> io::Result<Vec<u8>> {
            let mut data = Vec::<u8>::new();
            stdout.read_to_end(&mut data)?;
            Ok(data)
        });

        {
            let mut stdin = child.stdin.take().unwrap();
            let _write_result = stdin.write_all(compressed_data);
        }

        read_thread.join().unwrap().unwrap()
    };

    let status = child.wait().unwrap();
    if status.success() {
        println!("{}Decompressed: {} K", padding, data.len() / 1024);

        for section in BiosSections::new(&data) {
            dump_section(&section, &format!("{}    ", padding));
        }
    } else {
        println!("{}Error: {}", padding, status);
    }
}

fn dump_guid_defined(section_data: &[u8], padding: &str) {
    let header = plain::from_bytes::<section::GuidDefined>(section_data).unwrap();
    let data_offset = header.data_offset;
    let data = &section_data[(data_offset as usize) ..];
    let guid = header.guid;
    let len = data.len() / 1024;
    println!("{}  {}: {} K", padding, guid, len);

    #[allow(clippy::single_match)]
    match guid {
        SECTION_LZMA_COMPRESS_GUID => {
            let compressed_data = &section_data[mem::size_of::<section::GuidDefined>() ..];
            dump_lzma(compressed_data, &format!("{}    ", padding));
        },
        _ => ()
    }
}

fn dump_section(section: &BiosSection, padding: &str) {
    let header = section.header();
    let kind = header.kind();
    let data = section.data();
    let len = data.len() / 1024;
    println!("{}{:?}:  {} K", padding, kind, len);

    match kind{
        section::HeaderKind::GuidDefined => {
            dump_guid_defined(data, &format!("{}    ", padding));
        },
        section::HeaderKind::VolumeImage => {
            for volume in BiosVolumes::new(data) {
                dump_volume(&volume, &format!("{}    ", padding));
            }
        },
        _ => ()
    }
}

fn dump_file(file: &BiosFile, polarity: bool, padding: &str) {
    let header = file.header();
    let guid = header.guid;
    let data = file.data();
    let len = data.len() / 1024;
    let kind = header.kind();
    let attributes = header.attributes();
    let alignment = header.alignment();
    let state = header.state(polarity);
    println!("{}{}: {} K", padding, guid, len);
    println!("{}  Kind: {:?}", padding, kind);
    println!("{}  Attrib: {:?}", padding, attributes);
    println!("{}  Align: {}", padding, alignment);
    println!("{}  State: {:?}", padding, state);

    if header.sectioned() {
        for section in file.sections() {
            dump_section(&section, &format!("{}    ", padding));
        }
    }
}

fn dump_volume(volume: &BiosVolume, padding: &str) {
    let header = volume.header();
    let guid = header.guid;
    let header_len = header.header_length;
    let len = volume.data().len()/1024;
    let attributes = header.attributes();
    println!("{}{}: {}, {} K", padding, guid, header_len, len);
    println!("{}  Attrib: {:?}", padding, attributes);

    let polarity = attributes.contains(volume::Attributes::ERASE_POLARITY);
    for file in volume.files() {
        dump_file(&file, polarity, &format!("{}    ", padding));
    }
}

fn romulan(path: &str) -> Result<(), String> {
    println!("{}", path);

    let mut data = Vec::new();
    fs::File::open(path).map_err(|err| {
        format!("failed to open {}: {}", path, err)
    })?.read_to_end(&mut data).map_err(|err| {
        format!("failed to read {}: {}", path, err)
    })?;

    let rom = Rom::new(&data)?;

    if rom.high_assurance_platform()? {
        println!("  HAP: set");
    } else {
        println!("  HAP: not set");
    }

    if let Some(bios) = rom.bios()? {
        println!("  BIOS: {} K", bios.data().len()/1024);
        for volume in bios.volumes() {
            dump_volume(&volume, "    ");
        }
    } else {
        println!("  BIOS: None");
    }

    if let Some(me) = rom.me()? {
        println!("  ME: {} K", me.data().len()/1024);
        if let Some(version) = me.version() {
            println!("    Version: {}", version);
        } else {
            println!("    Version: Unknown");
        }
    } else {
        println!("  ME: None");
    }

    Ok(())
}


fn main() {
    for arg in env::args().skip(1) {
        if let Err(err) = romulan(&arg) {
            eprintln!("romulan: {}: {}", arg, err);
            process::exit(1);
        }
    }
}
