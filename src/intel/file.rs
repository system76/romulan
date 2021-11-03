// SPDX-License-Identifier: MIT

use bitflags::bitflags;
use plain::Plain;
use uefi::guid::Guid;

#[derive(Debug)]
pub enum HeaderKind {
    Raw,
    Freeform,
    SecurityCore,
    PeiCore,
    DxeCore,
    Peim,
    Driver,
    CombinedPeimDriver,
    Application,
    Mm,
    VolumeImage,
    CombinedMmDxe,
    MmCore,
    MmStandalone,
    MmCoreStandalone,
    Oem(u8),
    Debug(u8),
    Ffs(u8),
    Unknown(u8)
}

bitflags! {
    pub struct Attributes: u8 {
        const ATTRIB_TAIL_PRESENT = 0x01;
        const ATTRIB_RECOVERY = 0x02;
        const ATTRIB_HEADER_EXTENSION = 0x04;
        const ATTRIB_CHECKSUM = 0x40;
    }
}

bitflags! {
    pub struct State: u8 {
        const STATE_HEADER_CONSTRUCTION = 0x01;
        const STATE_HEADER_VALID = 0x02;
        const STATE_DATA_VALID = 0x04;
        const STATE_MARKED_FOR_UPDATE = 0x08;
        const STATE_DELETED = 0x10;
        const STATE_HEADER_INVALID = 0x20;
    }
}

#[repr(packed)]
pub struct Header {
    pub guid: Guid,
    pub integrity_check: u16,
    kind: u8,
    attributes: u8,
    size: [u8; 3],
    state: u8,
}

impl Header {
    pub fn kind(&self) -> HeaderKind {
        match self.kind {
            0x01 => HeaderKind::Raw,
            0x02 => HeaderKind::Freeform,
            0x03 => HeaderKind::SecurityCore,
            0x04 => HeaderKind::PeiCore,
            0x05 => HeaderKind::DxeCore,
            0x06 => HeaderKind::Peim,
            0x07 => HeaderKind::Driver,
            0x08 => HeaderKind::CombinedPeimDriver,
            0x09 => HeaderKind::Application,
            0x0A => HeaderKind::Mm,
            0x0B => HeaderKind::VolumeImage,
            0x0C => HeaderKind::CombinedMmDxe,
            0x0D => HeaderKind::MmCore,
            0x0E => HeaderKind::MmStandalone,
            0x0F => HeaderKind::MmCoreStandalone,
            oem @ 0xC0..=0xDF => HeaderKind::Oem(oem),
            debug @ 0xE0..=0xEF => HeaderKind::Debug(debug),
            ffs @ 0xF0..=0xFF => HeaderKind::Ffs(ffs),
            unknown => HeaderKind::Unknown(unknown)
        }
    }

    pub fn size(&self) -> usize {
        self.size[0] as usize | (self.size[1] as usize) << 8 | (self.size[2] as usize) << 16
    }

    pub fn attributes(&self) -> Attributes {
        Attributes::from_bits_truncate(self.attributes)
    }

    pub fn alignment(&self) -> u8 {
        (self.attributes & 0x38) >> 3
    }

    pub fn state(&self, polarity: bool) -> State {
        State::from_bits_truncate(if polarity {
            ! self.state
        } else {
            self.state
        })
    }

    pub fn sectioned(&self) -> bool {
        match self.kind() {
            HeaderKind::Raw => false,
            HeaderKind::Freeform => true,
            HeaderKind::SecurityCore => false,
            HeaderKind::PeiCore => true,
            HeaderKind::DxeCore => true,
            HeaderKind::Peim => true,
            HeaderKind::Driver => true,
            HeaderKind::CombinedPeimDriver => true,
            HeaderKind::Application => true,
            HeaderKind::Mm => true,
            HeaderKind::VolumeImage => true,
            HeaderKind::CombinedMmDxe => true,
            HeaderKind::MmCore => true,
            HeaderKind::MmStandalone => true,
            HeaderKind::MmCoreStandalone => false,
            HeaderKind::Oem(_oem) => false,
            HeaderKind::Debug(_debug) => false,
            HeaderKind::Ffs(_ffs) => false,
            HeaderKind::Unknown(_unknown) => false
        }
    }
}

unsafe impl Plain for Header {}
