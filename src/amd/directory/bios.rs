use alloc::string::String;
use core::mem;
use plain::Plain;

use super::{
    ComboDirectoryEntry,
    ComboDirectoryHeader,
    DirectoryHeader
};

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct BiosDirectoryEntry {
    /// 0x00: type of entry
    pub kind: u8,
    /// 0x01: memory region security attributes
    pub region_kind: u8,
    /// 0x02: flags (specific to type of directory)
    pub flags: u8,
    /// 0x03: used to filter entries by model
    pub sub_program: u8,
    /// 0x04: size of the entry
    pub size: u32,
    /// 0x08: source address
    pub source: u64,
    /// 0x10: destination address
    pub destination: u64,
}

unsafe impl Plain for BiosDirectoryEntry {}

pub struct BiosDirectory<'a> {
    header: &'a DirectoryHeader,
    entries: &'a [BiosDirectoryEntry]
}

impl<'a> BiosDirectory<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"$BHD" || &data[..4] == b"$BL2" {
            let header: &DirectoryHeader = plain::from_bytes(&data).map_err(|err| {
                format!("BIOS directory header invalid: {:?}", err)
            })?;

            return Ok(Self {
                header: header,
                entries: plain::slice_from_bytes_len(
                    &data[mem::size_of::<DirectoryHeader>()..],
                    header.entries as usize
                ).map_err(|err| {
                    format!("BIOS directory entries invalid: {:?}", err)
                })?
            });
        }

        Err(format!("BIOS directory header not found"))
    }

    pub fn header(&self) -> &'a DirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> &'a [BiosDirectoryEntry] {
        self.entries
    }
}

pub struct BiosComboDirectory<'a> {
    header: &'a ComboDirectoryHeader,
    entries: &'a [ComboDirectoryEntry]
}

impl<'a> BiosComboDirectory<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"2BHD" {
            let header: &ComboDirectoryHeader = plain::from_bytes(&data).map_err(|err| {
                format!("BIOS combo header invalid: {:?}", err)
            })?;

            return Ok(Self {
                header: header,
                entries: plain::slice_from_bytes_len(
                    &data[mem::size_of::<ComboDirectoryHeader>()..],
                    header.entries as usize
                ).map_err(|err| {
                    format!("BIOS combo entries invalid: {:?}", err)
                })?
            });
        }

        Err(format!("BIOS combo header not found"))
    }

    pub fn header(&self) -> &'a ComboDirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> &'a [ComboDirectoryEntry] {
        self.entries
    }
}
