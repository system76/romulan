// SPDX-License-Identifier: MIT

use alloc::string::String;
use core::mem;
use zerocopy::LayoutVerified;

pub mod directory;
pub mod flash;

pub struct Rom<'a> {
    data: &'a [u8],
    signature: &'a flash::Signature,
}

impl<'a> Rom<'a> {
    pub fn new(data: &'a [u8]) -> Result<Rom, String> {
        let mut i = 0;
        // TODO: Can we just iterate over chunks? The last one may be too short.
        /*
        for block in data.chunks(0x1000) {
        }
        */
        // TODO: Handle errors?
        while i + mem::size_of::<flash::Signature>() <= data.len() {
            if data[i..i + 4] == [0xaa, 0x55, 0xaa, 0x55] {
                let lv: LayoutVerified<_, flash::Signature> =
                    LayoutVerified::new_unaligned_from_prefix(&data[i..])
                        .unwrap()
                        .0;
                return Ok(Rom {
                    data: &data[i..],
                    signature: lv.into_ref(),
                    // .map_err(|err| format!("Flash signature invalid: {:?}", err))?,
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
