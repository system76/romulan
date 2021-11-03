// SPDX-License-Identifier: MIT

use plain::Plain;

#[repr(packed)]
pub struct Descriptor {
    pub valsig: u32,
    pub map0: u32,
    pub map1: u32,
    pub map2: u32,
    _reserved: [u8; 0xefc - 0x20],
    pub umap1: u32,
}

unsafe impl Plain for Descriptor {}

#[repr(packed)]
pub struct Region {
    pub data: [u32; 9],
}

unsafe impl Plain for Region {}

#[repr(packed)]
pub struct Component {
    pub comp: u32,
    pub ill: u32,
    pub pb: u32,
}

unsafe impl Plain for Component {}

#[repr(packed)]
pub struct PchStrap {
    pub data: [u32; 18],
}

unsafe impl Plain for PchStrap {}

#[repr(packed)]
pub struct Master {
    pub mstr1: u32,
    pub mstr2: u32,
    pub mstr3: u32,
    pub mstr4: u32,
    pub mstr5: u32,
}

unsafe impl Plain for Master {}

#[repr(packed)]
pub struct ProcStrap {
    pub data: [u32; 8]
}

unsafe impl Plain for ProcStrap {}
