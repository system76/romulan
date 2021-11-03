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
pub struct PspDirectoryEntry {
    /// 0x00: type of entry
    pub kind: u8,
    /// 0x01: used to filter entries by model
    pub sub_program: u8,
    /// 0x02: specifies which ROM contains the entry
    pub rom_id: u8,
    pub rsvd_03: u8,
    /// 0x04: size of the entry
    pub size: u32,
    /// 0x08: location or value of the entry
    pub value: u64,
}

unsafe impl Plain for PspDirectoryEntry {}

pub struct PspDirectory<'a> {
    header: &'a DirectoryHeader,
    entries: &'a [PspDirectoryEntry]
}

impl<'a> PspDirectory<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"$PSP" {
            let header: &DirectoryHeader = plain::from_bytes(&data).map_err(|err| {
                format!("PSP directory header invalid: {:?}", err)
            })?;

            return Ok(Self {
                header: header,
                entries: plain::slice_from_bytes_len(
                    &data[mem::size_of::<DirectoryHeader>()..],
                    header.entries as usize
                ).map_err(|err| {
                    format!("PSP directory entries invalid: {:?}", err)
                })?
            });
        }

        Err(format!("PSP directory header not found"))
    }

    pub fn header(&self) -> &'a DirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> &'a [PspDirectoryEntry] {
        self.entries
    }
}

pub struct PspComboDirectory<'a> {
    header: &'a ComboDirectoryHeader,
    entries: &'a [ComboDirectoryEntry]
}

impl<'a> PspComboDirectory<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"2PSP" {
            let header: &ComboDirectoryHeader = plain::from_bytes(&data).map_err(|err| {
                format!("PSP combo header invalid: {:?}", err)
            })?;

            return Ok(Self {
                header: header,
                entries: plain::slice_from_bytes_len(
                    &data[mem::size_of::<ComboDirectoryHeader>()..],
                    header.entries as usize
                ).map_err(|err| {
                    format!("PSP combo entries invalid: {:?}", err)
                })?
            });
        }

        Err(format!("PSP combo header not found"))
    }

    pub fn header(&self) -> &'a ComboDirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> &'a [ComboDirectoryEntry] {
        self.entries
    }
}
