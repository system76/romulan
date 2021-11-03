// SPDX-License-Identifier: MIT

use alloc::string::String;
use core::{fmt, mem};

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum RegionKind {
    Descriptor = 0,
    Bios = 1,
    ManagementEngine = 2,
    Ethernet = 3,
    PlatformData = 4,
    Reserved5 = 5,
    Reserved6 = 6,
    Reserved7 = 7,
    EmbeddedController = 8,
}

impl fmt::Display for RegionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RegionKind::Descriptor => "Flash Descriptor",
            RegionKind::Bios => "BIOS",
            RegionKind::ManagementEngine => "Intel ME",
            RegionKind::Ethernet => "GbE",
            RegionKind::PlatformData => "Platform Data",
            RegionKind::EmbeddedController => "EC",
            _ => "Reserved",
        };
        write!(f, "{}", name)
    }
}

pub const HAP: u32 = 0x10000;

pub mod file;
pub mod flash;
pub mod section;
pub mod volume;

pub struct Rom<'a> {
    data: &'a [u8],
    descriptor: &'a flash::Descriptor,
}

impl<'a> Rom<'a> {
    pub fn new(data: &'a [u8]) -> Result<Rom, String> {
        let mut i = 16;

        while i + mem::size_of::<flash::Descriptor>() <= data.len() {
            if data[i..i + 4] == [0x5a, 0xa5, 0xf0, 0x0f] {
                return Ok(Rom {
                    data: &data[i - 16..],
                    descriptor: plain::from_bytes(&data[i..]).map_err(|err| {
                        format!("Flash descriptor invalid: {:?}", err)
                    })?
                });
            }

            i += 4;
        }

        Err(format!("Flash descriptor not found"))
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn flash_descriptor(&self) -> &'a flash::Descriptor {
        self.descriptor
    }

    pub fn flash_region(&self) -> Result<&'a flash::Region, String> {
        let offset = (((self.descriptor.map0 >> 16) & 0xff) << 4) as usize;

        if offset >= self.data.len() {
            return Err(format!("Flash region table truncated"))
        }

        plain::from_bytes(&self.data[offset..]).map_err(|err| {
            format!("Flash region table invalid: {:?}", err)
        })
    }

    pub fn flash_pchstrap(&self) -> Result<&'a flash::PchStrap, String> {
        let offset = (((self.descriptor.map1 >> 16) & 0xff) << 4) as usize;

        if offset >= self.data.len() {
            return Err(format!("PCHSTRAP table truncated"))
        }

        plain::from_bytes(&self.data[offset..]).map_err(|err| {
            format!("PCHSTRAP table invalid: {:?}", err)
        })
    }

    pub fn high_assurance_platform(&self) -> Result<bool, String> {
        let pchstrap = self.flash_pchstrap()?;
        Ok(pchstrap.data[0] & HAP == HAP)
    }

    pub fn get_region_base_limit(&self, kind: RegionKind) -> Result<Option<(usize, usize)>, String> {
        let frba = self.flash_region()?;

        let reg = frba.data[kind as usize];

        let base_mask = 0x7fff;
        let limit_mask = base_mask << 16;

    	let base = (reg & base_mask) << 12;
    	let limit = ((reg & limit_mask) >> 4) | 0xfff;

        if limit > base {
            Ok(Some((base as usize, limit as usize)))
        } else {
            Ok(None)
        }
    }

    pub fn get_region(&self, kind: RegionKind) -> Result<Option<&'a [u8]>, String> {
        if let Some((base, limit)) = self.get_region_base_limit(kind)? {
            if (limit as usize) < self.data.len() {
                Ok(Some(&self.data[base as usize..limit as usize + 1]))
            } else {
                Err(format!("{:?} region invalid: {} >= {}", kind, limit, self.data.len()))
            }
        } else {
            Ok(None)
        }
    }

    pub fn bios(&self) -> Result<Option<Bios<'a>>, String> {
        if let Some(data) = self.get_region(RegionKind::Bios)? {
            Ok(Some(Bios { data }))
        } else {
            Ok(None)
        }
    }

    pub fn me(&self) -> Result<Option<Me<'a>>, String> {
        if let Some(data) = self.get_region(RegionKind::ManagementEngine)? {
            Ok(Some(Me { data }))
        } else {
            Ok(None)
        }
    }
}

pub struct Bios<'a> {
    data: &'a [u8],
}

impl<'a> Bios<'a> {
    pub fn new(data: &'a [u8]) -> Result<Bios, String> {
        Ok(Bios { data })
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn volumes(&self) -> BiosVolumes {
        BiosVolumes::new(self.data)
    }
}

pub struct BiosVolumes<'a> {
    data: &'a [u8],
    i: usize,
}

impl<'a> BiosVolumes<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            i: 0
        }
    }
}

impl<'a> Iterator for BiosVolumes<'a> {
    type Item = BiosVolume<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i + mem::size_of::<volume::Header>() <= self.data.len() {
            let header_data = &self.data[self.i..];
            let header = plain::from_bytes::<volume::Header>(header_data).unwrap();

            if header.valid() {
                self.i += header.length as usize;

                /*
                self.i += mem::size_of::<volume::Header>();

                while self.i + mem::size_of::<volume::BlockEntry>() <= self.data.len() {
                    let block_entry = plain::from_bytes::<volume::BlockEntry>(&self.data[self.i..]).unwrap();
                    self.i += mem::size_of::<volume::BlockEntry>();

                    if block_entry.num_blocks == 0 && block_entry.block_length == 0 {
                        break;
                    } else {
                        println!("    {}, {}", block_entry.num_blocks, block_entry.block_length);
                    }
                }

                self.i += header.header_length as usize - mem::size_of::<volume::Header>();
                */

                return Some(BiosVolume {
                    header,
                    data: &header_data[header.header_length as usize .. header.length as usize]
                });
            } else {
                self.i += 8;
            }
        }

        None
    }
}

pub struct BiosVolume<'a> {
    header: &'a volume::Header,
    data: &'a [u8],
}

impl<'a> BiosVolume<'a> {
    pub fn header(&self) -> &'a volume::Header {
        self.header
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn files(&self) -> BiosFiles {
        BiosFiles::new(self.data)
    }
}

pub struct BiosFiles<'a> {
    data: &'a [u8],
    i: usize,
}

impl<'a> BiosFiles<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            i: 0
        }
    }
}

impl<'a> Iterator for BiosFiles<'a> {
    type Item = BiosFile<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i + mem::size_of::<file::Header>() <= self.data.len() {
            let header_data = &self.data[self.i..];
            let header = plain::from_bytes::<file::Header>(header_data).unwrap();

            if header.size() == 0xFFFFFF {
                self.i = self.data.len();
                None
            } else {
                self.i += ((header.size() + 7) / 8) * 8;

                Some(BiosFile {
                    header,
                    data: &header_data[mem::size_of::<file::Header>() .. header.size()]
                })
            }
        } else {
            None
        }
    }
}

pub struct BiosFile<'a> {
    header: &'a file::Header,
    data: &'a [u8],
}

impl<'a> BiosFile<'a> {
    pub fn header(&self) -> &'a file::Header {
        self.header
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn sections(&self) -> BiosSections {
        BiosSections::new(self.data)
    }
}

pub struct BiosSections<'a> {
    data: &'a [u8],
    i: usize,
}

impl<'a> BiosSections<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            i: 0
        }
    }
}

impl<'a> Iterator for BiosSections<'a> {
    type Item = BiosSection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i + mem::size_of::<section::Header>() <= self.data.len() {
            let header_data = &self.data[self.i..];
            let header = plain::from_bytes::<section::Header>(header_data).unwrap();

            if header.size() == 0xFFFFFF {
                self.i = self.data.len();
                None
            } else {

                self.i += ((header.size() + 3) / 4) * 4;

                Some(BiosSection {
                    header,
                    data: &header_data[mem::size_of::<section::Header>() .. header.size()]
                })
            }
        } else {
            None
        }
    }
}

pub struct BiosSection<'a> {
    header: &'a section::Header,
    data: &'a [u8],
}

impl<'a> BiosSection<'a> {
    pub fn header(&self) -> &'a section::Header {
        self.header
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

pub struct Me<'a> {
    data: &'a [u8],
}

impl<'a> Me<'a> {
    pub fn new(data: &'a [u8]) -> Result<Me, String> {
        Ok(Me { data })
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn version(&self) -> Option<String> {
        let mut i = 0;
        while i + 4 <= self.data.len() {
            if &self.data[i..i + 4] == b"$FPT" {
                break;
            }
            i += 1;
        }

        if i + 0x20 <= self.data.len() {
            let mut version = String::new();

            let bytes = &self.data[i + 0x18..i + 0x20];
            for part in bytes.chunks(2) {
                if ! version.is_empty() {
                    version.push('.');
                }
                version.push_str(&format!("{}", part[0] as u16 | (part[1] as u16) << 8));
            }

            Some(version)
        } else {
            None
        }
    }

    pub fn modules(&self) -> Option<u32> {
        if self.data.len() >= 0x18 {
            let bytes = &self.data[0x14..0x18];
            Some(bytes[0] as u32 | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16 | (bytes[3] as u32) << 24)
        } else {
            None
        }
    }
}
