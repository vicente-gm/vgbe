// SPDX-License-Identifier: GPL-3.0-or-later
//
// VGBE - Vicente's Game Boy Emulator
// Copyright (C) 2026 Vicente <vicente.gnzmls@gmail.com>
//
// This file is part of VGBE.
//
// VGBE is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// VGBE is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with VGBE. If not, see <https://www.gnu.org/licenses/>.

use crate::cartridge::mbc::*;

#[derive(Debug, Clone)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: u8,
    pub rom_size_code: u8,
    pub ram_size_code: u8,
    pub cgb_flag: u8,
    pub sgb_flag: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl CartridgeHeader {
    const HEADER_CHECKSUM_START: usize = 0x0134;
    const HEADER_CHECKSUM_END: usize = 0x014C;
    const HEADER_CHECKSUM_ADDR: usize = 0x014D;

    const GLOBAL_CHECKSUM_HIGH_ADDR: usize = 0x014E;
    const GLOBAL_CHECKSUM_LOW_ADDR: usize = 0x014F;

    pub fn parse(rom: &[u8]) -> Self {
        assert!(
            rom.len() > CartridgeHeader::GLOBAL_CHECKSUM_LOW_ADDR,
            "ROM too small to contain a valid Game Boy cartridge header"
        );

        let title_bytes = &rom[0x0134..=0x0143];
        let title_end = title_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(title_bytes.len());

        let title = String::from_utf8_lossy(&title_bytes[..title_end]).to_string();

        Self {
            title,
            cartridge_type: rom[0x0147],
            rom_size_code: rom[0x0148],
            ram_size_code: rom[0x0149],
            cgb_flag: rom[0x0143],
            sgb_flag: rom[0x0146],
            header_checksum: rom[CartridgeHeader::HEADER_CHECKSUM_ADDR],
            global_checksum: Self::stored_global_checksum(rom),
        }
    }

    pub fn compute_header_checksum(rom: &[u8]) -> u8 {
        rom[CartridgeHeader::HEADER_CHECKSUM_START..=CartridgeHeader::HEADER_CHECKSUM_END]
            .iter()
            .fold(0u8, |checksum, &byte| {
                checksum
                    .wrapping_sub(byte)
                    .wrapping_sub(1)
            })
    }

    pub fn compute_global_checksum(rom: &[u8]) -> u16 {
        rom.iter()
            .enumerate()
            .filter(|(addr, _)| {
                *addr != CartridgeHeader::GLOBAL_CHECKSUM_HIGH_ADDR && *addr != CartridgeHeader::GLOBAL_CHECKSUM_LOW_ADDR
            })
            .fold(0u16, |checksum, (_, &byte)| {
                checksum.wrapping_add(byte as u16)
            })
    }

    pub fn stored_global_checksum(rom: &[u8]) -> u16 {
        ((rom[CartridgeHeader::GLOBAL_CHECKSUM_HIGH_ADDR] as u16) << 8)
            | rom[CartridgeHeader::GLOBAL_CHECKSUM_LOW_ADDR] as u16
    }

    pub fn is_header_checksum_valid(&self, rom: &[u8]) -> bool {
        Self::compute_header_checksum(rom) == self.header_checksum
    }

    pub fn is_global_checksum_valid(&self, rom: &[u8]) -> bool {
        Self::compute_global_checksum(rom) == self.global_checksum
    }

    pub fn mbc_type(&self) -> MbcType {
        match self.cartridge_type {
            0x00 => MbcType::RomOnly,

            0x01 | 0x02 | 0x03 => MbcType::Mbc1,

            0x05 | 0x06 => MbcType::Mbc2,

            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => MbcType::Mbc3,

            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => MbcType::Mbc5,

            other => MbcType::Unknown(other),   // TODO: Other types are not supported yet
        }
    }

    pub fn rom_size_bytes(&self) -> usize {
        match self.rom_size_code {
            0x00 => 32 * 1024,
            0x01 => 64 * 1024,
            0x02 => 128 * 1024,
            0x03 => 256 * 1024,
            0x04 => 512 * 1024,
            0x05 => 1024 * 1024,
            0x06 => 2 * 1024 * 1024,
            0x07 => 4 * 1024 * 1024,
            0x08 => 8 * 1024 * 1024,
            0x52 => 72 * 16 * 1024,
            0x53 => 80 * 16 * 1024,
            0x54 => 96 * 16 * 1024,
            _ => 0,
        }
    }

    pub fn ram_size_bytes(&self) -> usize {
        match self.ram_size_code {
            0x00 => 0,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        }
    }

    pub fn has_battery(&self) -> bool {
        matches!(
            self.cartridge_type,
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22
        )
    }
}
