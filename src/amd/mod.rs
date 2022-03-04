// SPDX-License-Identifier: MIT

use alloc::string::String;
use core::mem;

pub mod flash;
pub mod directory;

pub struct Rom<'a> {
    data: &'a [u8],
    signature: &'a flash::Signature,
}

impl<'a> Rom<'a> {
    pub fn new(data: &'a [u8]) -> Result<Rom, String> {
        let mut i = 0;

        while i + mem::size_of::<flash::Signature>() <= data.len() {
            if data[i..i + 4] == [0xaa, 0x55, 0xaa, 0x55] {
                return Ok(Rom {
                    data: &data[i..],
                    signature: plain::from_bytes(&data[i..]).map_err(|err| {
                        format!("Flash signature invalid: {:?}", err)
                    })?
                });
            }

            i += 0x1000;
        }

        Err(format!("Flash signature not found"))
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn signature(&self) -> &'a flash::Signature {
        self.signature
    }
}
