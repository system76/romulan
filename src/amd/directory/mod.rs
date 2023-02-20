use alloc::string::String;
use plain::Plain;
use serde::{Deserialize, Serialize};
use zerocopy::{AsBytes, FromBytes, Unaligned};

pub use self::bios::*;
pub use self::psp::*;

mod bios;
mod psp;

pub enum Directory<'a> {
    Bios(BiosDirectory<'a>),
    BiosCombo(BiosComboDirectory<'a>),
    BiosLevel2(BiosDirectory<'a>),
    Psp(PspDirectory<'a>),
    PspCombo(PspComboDirectory<'a>),
    PspLevel2(PspDirectory<'a>),
}

impl<'a> Directory<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        match &data[..4] {
            b"$BHD" => BiosDirectory::new(data).map(Self::Bios),
            b"2BHD" => BiosComboDirectory::new(data).map(Self::BiosCombo),
            b"$BL2" => BiosDirectory::new(data).map(Self::BiosLevel2),
            b"$PSP" => PspDirectory::new(data).map(Self::Psp),
            b"2PSP" => PspComboDirectory::new(data).map(Self::PspCombo),
            b"$PL2" => PspDirectory::new(data).map(Self::PspLevel2),
            unknown => Err(format!("unknown directory signature {:X?}", unknown)),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(packed)]
pub struct DirectoryHeader {
    /// 0x00: Magic of directory ("$BHD" or "$PSP")
    pub magic: u32,
    /// 0x04: CRC of all directory data after this
    pub checksum: u32,
    /// 0x08: number of entries
    pub entries: u32,
    pub rsvd_0c: u32,
}

unsafe impl Plain for DirectoryHeader {}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(packed)]
pub struct ComboDirectoryHeader {
    /// 0x00: Magic of directory ("2BHD" or "2PSP")
    pub magic: u32,
    /// 0x04: CRC of all directory data after this
    pub checksum: u32,
    /// 0x08: number of entries
    pub entries: u32,
    /// 0x0c: 0 for dynamic look up through all entries, 1 for PSP or chip ID match.
    /// Only for PSP combo directory
    pub look_up_mode: u32,
    pub rsvd_10: u32,
    pub rsvd_14: u32,
    pub rsvd_18: u32,
    pub rsvd_1c: u32,
}

unsafe impl Plain for ComboDirectoryHeader {}

#[derive(AsBytes, Unaligned, FromBytes, Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(packed)]
pub struct ComboDirectoryEntry {
    /// 0x00: 0 to compare PSP ID, 1 to compare chip ID
    pub id_select: u32,
    /// 0x04: PSP or chip ID
    pub id: u32,
    /// 0x08: Address of directory
    pub directory: u64,
}

unsafe impl Plain for ComboDirectoryEntry {}
