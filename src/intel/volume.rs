// SPDX-License-Identifier: MIT

use bitflags::bitflags;
use plain::Plain;
use uefi::guid::Guid;

bitflags! {
    pub struct Attributes: u32 {
        const READ_DISABLED_CAP = 0x00000001;
        const READ_ENABLED_CAP = 0x00000002;
        const READ_STATUS = 0x00000004;
        const WRITE_DISABLED_CAP = 0x00000008;
        const WRITE_ENABLED_CAP = 0x00000010;
        const WRITE_STATUS = 0x00000020;
        const LOCK_CAP = 0x00000040;
        const LOCK_STATUS = 0x00000080;
        const STICKY_WRITE = 0x00000200;
        const MEMORY_MAPPED = 0x00000400;
        const ERASE_POLARITY = 0x00000800;
        const ALIGNMENT_CAP = 0x00008000;
        const ALIGNMENT_2 = 0x00010000;
        const ALIGNMENT_4 = 0x00020000;
        const ALIGNMENT_8 = 0x00040000;
        const ALIGNMENT_16 = 0x00080000;
        const ALIGNMENT_32 = 0x00100000;
        const ALIGNMENT_64 = 0x00200000;
        const ALIGNMENT_128 = 0x00400000;
        const ALIGNMENT_256 = 0x00800000;
        const ALIGNMENT_512 = 0x01000000;
        const ALIGNMENT_1K = 0x02000000;
        const ALIGNMENT_2K = 0x04000000;
        const ALIGNMENT_4K = 0x08000000;
        const ALIGNMENT_8K = 0x10000000;
        const ALIGNMENT_16K = 0x20000000;
        const ALIGNMENT_32K = 0x40000000;
        const ALIGNMENT_64K = 0x80000000;
    }
}

#[repr(packed)]
pub struct Header {
    pub zero_vector: [u8; 16],
    pub guid: Guid,
    pub length: u64,
    pub signature: [u8; 4],
    pub attributes: u32,
    pub header_length: u16,
    pub checksum: u16,
    pub reserved: [u8; 3],
    pub revision: u8,
}

impl Header {
    pub fn valid(&self) -> bool {
        self.signature == *b"_FVH"
    }

    pub fn attributes(&self) -> Attributes {
        Attributes::from_bits_truncate(self.attributes)
    }
}

unsafe impl Plain for Header {}

#[repr(packed)]
pub struct BlockEntry {
    pub num_blocks: u32,
    pub block_length: u32,
}

unsafe impl Plain for BlockEntry {}
