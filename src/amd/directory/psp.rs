use alloc::{boxed::Box, string::String, vec::Vec};
use core::mem;
use serde::{Deserialize, Serialize};
use zerocopy::{AsBytes, FromBytes, LayoutVerified as LV};

use super::{ComboDirectoryEntry, ComboDirectoryHeader, DirectoryHeader};

#[derive(AsBytes, FromBytes, Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
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

impl PspDirectoryEntry {
    pub fn data(&self, data: &[u8]) -> Result<Box<[u8]>, String> {
        if self.size == 0xFFFFFFFF {
            return Ok(vec![
                (self.value) as u8,
                (self.value >> 8) as u8,
                (self.value >> (2 * 8)) as u8,
                (self.value >> (3 * 8)) as u8,
                (self.value >> (4 * 8)) as u8,
                (self.value >> (5 * 8)) as u8,
                (self.value >> (6 * 8)) as u8,
                (self.value >> (7 * 8)) as u8,
            ]
            .into_boxed_slice());
        }

        let start = (self.value & 0x1FFFFFF) as usize;
        let end = start + self.size as usize;
        if end <= data.len() {
            Ok(data[start..end].to_vec().into_boxed_slice())
        } else {
            Err(format!(
                "PSP directory entry invalid: {:08X}:{:08X}",
                start, end
            ))
        }
    }

    pub fn description(&self) -> &'static str {
        match self.kind {
            0x00 => "AMD Public Key",
            0x01 => "PSP Boot Loader",
            0x02 => "PSP Secure OS",
            0x03 => "PSP Recovery Boot Loader",
            0x04 => "PSP Non-volatile Data",
            0x08 => "SMU Firmware",
            0x09 => "AMD Secure Debug Key",
            0x0A => "OEM Public Key",
            0x0B => "PSP Soft Fuse Chain",
            0x0C => "PSP Trustlet",
            0x0D => "PSP Trustlet Public Key",
            0x12 => "SMU Firmware",
            0x13 => "PSP Early Secure Unlock Debug",
            0x20 => "IP Discovery",
            0x21 => "Wrapped iKEK",
            0x22 => "PSP Token Unlock",
            0x24 => "Security Policy",
            0x25 => "MP2 Firmware",
            0x26 => "MP2 Firmware Part 2",
            0x27 => "User Mode Unit Test",
            0x28 => "System Driver",
            0x29 => "KVM Image",
            0x2A => "MP5 Firmware",
            0x2B => "Embedded Firmware Signature",
            0x2C => "TEE Write-once NVRAM",
            0x2D => "External Chipset PSP Boot Loader",
            0x2E => "External Chipset MP0 Firmware",
            0x2F => "External Chipset MP1 Firmware",
            0x30 => "PSP AGESA Binary 0",
            0x31 => "PSP AGESA Binary 1",
            0x32 => "PSP AGESA Binary 2",
            0x33 => "PSP AGESA Binary 3",
            0x34 => "PSP AGESA Binary 4",
            0x35 => "PSP AGESA Binary 5",
            0x36 => "PSP AGESA Binary 6",
            0x37 => "PSP AGESA Binary 7",
            0x38 => "SEV Data",
            0x39 => "SEV Code",
            0x3A => "Processor Serial Number Allow List",
            0x3B => "SERDES Microcode",
            0x3C => "VBIOS Pre-load",
            0x3D => "WLAN Umac",
            0x3E => "WLAN Imac",
            0x3F => "WLAN Bluetooth",
            0x40 => "PSP Level 2 Directory",
            0x41 => "External Chipset MP0 Boot Loader",
            0x42 => "DXIO PHY SRAM Firmware",
            0x43 => "DXIO PHY SRAM Firmware Public Key",
            0x44 => "USB PHY Firmware",
            0x45 => "Security Policy for tOS",
            0x46 => "External Chipset PSP Boot Loader",
            0x47 => "DRTM TA",
            0x48 => "Recovery L2A PSP Directory",
            0x49 => "Recovery L2 BIOS Directory",
            0x4A => "Recovery L2B PSP Directory",
            0x4C => "External Chipset Security Policy",
            0x4D => "External Chipset Secure Debug Unlock",
            0x4E => "PMU Public Key",
            0x4F => "UMC Firmware",
            0x50 => "PSP Boot Loader Public Keys Table",
            0x51 => "PSP tOS Public Keys Table",
            0x52 => "OEM PSP Boot Loader Application",
            0x53 => "OEM PSP Boot Loader Application Public Key",
            0x54 => "PSP RPMC NVRAM",
            0x55 => "PSP Boot Loader Anti-rollback",
            0x56 => "PSP Secure OS Anti-rollback",
            0x57 => "CVIP Configuration Table",
            0x58 => "DMCU-ERAM",
            0x59 => "DMCU-ISR",
            0x5A => "MSMU Binary 0",
            0x5B => "MSMU Binary 1",
            0x73 => "PSP Boot Loader AB",
            0x80 => "OEM Sys-TA",
            0x81 => "OEM Sys-TA Signing Key",
            _ => "Unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PspDirectory {
    header: DirectoryHeader,
    entries: Vec<PspDirectoryEntry>,
}

impl<'a> PspDirectory {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"$PSP" || &data[..4] == b"$PL2" {
            let header =
                DirectoryHeader::read_from_prefix(data).ok_or("PSP directory header invalid")?;

            let hs = mem::size_of::<DirectoryHeader>();
            let (entries, _) = LV::<_, [PspDirectoryEntry]>::new_slice_from_prefix(
                &data[hs..],
                header.entries as usize,
            )
            .ok_or("PSP directory entries invalid")?;

            return Ok(Self {
                header,
                entries: entries.to_vec(),
            });
        }

        Err(format!("PSP directory header not found"))
    }

    pub fn header(&self) -> DirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> Vec<PspDirectoryEntry> {
        self.entries.clone()
    }
}

pub struct PspComboDirectory {
    header: ComboDirectoryHeader,
    entries: Vec<ComboDirectoryEntry>,
}

impl<'a> PspComboDirectory {
    pub fn new(data: &'a [u8]) -> Result<Self, String> {
        if &data[..4] == b"2PSP" {
            let header =
                ComboDirectoryHeader::read_from_prefix(data).ok_or("PSP combo header invalid")?;

            let hs = mem::size_of::<ComboDirectoryHeader>();
            let (entries, _) = LV::<_, [ComboDirectoryEntry]>::new_slice_from_prefix(
                &data[hs..],
                header.entries as usize,
            )
            .ok_or("PSP combo entries invalid")?;

            return Ok(Self {
                header,
                entries: entries.to_vec(),
            });
        }

        Err(format!("PSP combo header not found"))
    }

    pub fn header(&self) -> ComboDirectoryHeader {
        self.header
    }

    pub fn entries(&self) -> Vec<ComboDirectoryEntry> {
        self.entries.clone()
    }
}
