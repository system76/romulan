// SPDX-License-Identifier: MIT

use plain::Plain;
use uefi::guid::Guid;

#[derive(Debug)]
pub enum HeaderKind {
    Compression,
    GuidDefined,
    Disposable,
    Pe32,
    Pic,
    Te,
    DxeDepex,
    Version,
    UserInterface,
    Compatibility16,
    VolumeImage,
    Freeform,
    Raw,
    PeiDepex,
    MmDepex,
    Unknown(u8)
}

//TODO: Extended size
#[repr(packed)]
pub struct Header {
    size: [u8; 3],
    kind: u8,
}

impl Header {
    pub fn size(&self) -> usize {
        self.size[0] as usize | (self.size[1] as usize) << 8 | (self.size[2] as usize) << 16
    }

    pub fn kind(&self) -> HeaderKind {
        match self.kind {
            0x01 => HeaderKind::Compression,
            0x02 => HeaderKind::GuidDefined,
            0x03 => HeaderKind::Disposable,
            0x10 => HeaderKind::Pe32,
            0x11 => HeaderKind::Pic,
            0x12 => HeaderKind::Te,
            0x13 => HeaderKind::DxeDepex,
            0x14 => HeaderKind::Version,
            0x15 => HeaderKind::UserInterface,
            0x16 => HeaderKind::Compatibility16,
            0x17 => HeaderKind::VolumeImage,
            0x18 => HeaderKind::Freeform,
            0x19 => HeaderKind::Raw,
            0x1B => HeaderKind::PeiDepex,
            0x1C => HeaderKind::MmDepex,
            unknown => HeaderKind::Unknown(unknown)
        }
    }
}

unsafe impl Plain for Header {}

#[repr(packed)]
pub struct GuidDefined {
    pub guid: Guid,
    pub data_offset: u16,
    pub attributes: u16,
}

unsafe impl Plain for GuidDefined {}
