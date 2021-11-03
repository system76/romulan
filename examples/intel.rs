// SPDX-License-Identifier: MIT

use std::{env, fs, process};

use romulan::intel::{RegionKind, Rom};

fn main() {
    let file = if let Some(file) = env::args().nth(1) {
        file
    } else {
        eprintln!("used_regions <file>");
        process::exit(1);
    };

    let data = fs::read(file).unwrap();

    // Get the Flash Descriptor Region Section
    let flash_region = Rom::new(&data).unwrap().flash_region().unwrap();

    // Determine a regions base and limit addresses from the Flash
    // Region Record that corresponds to it.
    let region_area = |flreg: u32| -> (u32, u32) {
        let base = (flreg & 0x7FFF) << 12;
        let limit = ((flreg & (0x7FFF << 16)) >> 4) | 0xFFF;
        (base, limit)
    };

    // Print a regions index and name, along with it's base and limit
    // addresses. Explicitly mark unused regions as such.
    let print_region_info = |region: RegionKind| {
        let reg = flash_region.data[region as usize];
        let (base, limit) = region_area(reg);
        let unused = if (base == 0x07FF_F000) && (limit == 0xFFF) {
            " (unused)"
        } else {
            ""
        };
        println!("  {}: {}", region as usize, region);
        println!("      ({:#010X} - {:#010X}){}", base, limit, unused);
    };

    println!("Flash Regions");
    print_region_info(RegionKind::Descriptor);
    print_region_info(RegionKind::Bios);
    print_region_info(RegionKind::ManagementEngine);
    print_region_info(RegionKind::Ethernet);
    print_region_info(RegionKind::PlatformData);
    print_region_info(RegionKind::EmbeddedController);
}
