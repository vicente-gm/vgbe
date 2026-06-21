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

use std::fs::File;
use std::io::Read;

mod header;
mod mbc;

use header::*;
use mbc::*;

#[derive(Debug)]
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    header: CartridgeHeader,
    mbc: Mbc,
}

impl Cartridge {
    fn check_header(header: &CartridgeHeader, rom: &Vec<u8>) {
        if !header.is_header_checksum_valid(&rom) {
            panic!(
                "Invalid cartridge header checksum: stored={:#04X}, computed={:#04X}",
                header.header_checksum,
                CartridgeHeader::compute_header_checksum(&rom)
            );
        }

        if !header.is_global_checksum_valid(&rom) {
            log::warn!(
                "Warning: invalid global checksum: stored={:#06X}, computed={:#06X}",
                header.global_checksum,
                CartridgeHeader::compute_global_checksum(&rom)
            );
        }
    }

    pub fn new(file: &mut File, name: &str) -> Cartridge {
        let mut rom: Vec<u8> = Vec::new();

        file.read_to_end(&mut rom)
            .unwrap_or_else(|_| panic!("Could not read ROM file {}", name));

        assert!(
            rom.len() >= 0x150,
            "ROM file {} is too small to contain a valid Game Boy header",
            name
        );

        let header = CartridgeHeader::parse(&rom);
        Cartridge::check_header(&header, &rom);

        let ram = match header.mbc_type() {
            MbcType::Mbc2 => vec![0; 512],
            _ => vec![0; header.ram_size_bytes()],
        };

        let mbc = match header.mbc_type() {
            MbcType::RomOnly => Mbc::RomOnly,

            MbcType::Mbc1 => Mbc::Mbc1 {
                rom_bank: 1,
                ram_bank: 0,
                ram_enabled: false,
                banking_mode: 0,
            },

            MbcType::Mbc2 => Mbc::Mbc2 {
                rom_bank: 1,
                ram_enabled: false,
            },

            MbcType::Mbc3 => Mbc::Mbc3 {
                rom_bank: 1,
                ram_bank: 0,
                ram_enabled: false,
            },

            MbcType::Mbc5 => Mbc::Mbc5 {
                rom_bank: 1,
                ram_bank: 0,
                ram_enabled: false,
            },

            MbcType::Unknown(kind) => { // TODO: Other types are not supported yet
                panic!("Unsupported cartridge type: 0x{:02X}", kind);
            }
        };

        Cartridge {
            rom,
            ram,
            header,
            mbc,
        }
    }

    pub fn new_void() -> Cartridge {
        let rom = vec![0; 0x8000];
        let header = CartridgeHeader::parse(&rom);

        Cartridge {
            rom,
            ram: Vec::new(),
            header,
            mbc: Mbc::RomOnly,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match &self.mbc {
            Mbc::RomOnly => self.read_rom_only(address),
            Mbc::Mbc1 {
                rom_bank,
                ram_bank,
                ram_enabled,
                banking_mode,
            } => self.read_mbc1(address, *rom_bank, *ram_bank, *ram_enabled, *banking_mode),

            Mbc::Mbc2 { rom_bank, ram_enabled } => {
                self.read_mbc2(address, *rom_bank, *ram_enabled)
            }

            Mbc::Mbc3 {
                rom_bank,
                ram_bank,
                ram_enabled,
            } => self.read_mbc3(address, *rom_bank, *ram_bank, *ram_enabled),

            Mbc::Mbc5 {
                rom_bank,
                ram_bank,
                ram_enabled,
            } => self.read_mbc5(address, *rom_bank, *ram_bank, *ram_enabled),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match &mut self.mbc {
            Mbc::RomOnly => self.write_rom_only(address, value),

            Mbc::Mbc1 {
                rom_bank,
                ram_bank,
                ram_enabled,
                banking_mode,
            } => Self::write_mbc1(
                &mut self.ram,
                address,
                value,
                rom_bank,
                ram_bank,
                ram_enabled,
                banking_mode,
            ),

            Mbc::Mbc2 {
                rom_bank,
                ram_enabled,
            } => Self::write_mbc2(&mut self.ram, address, value, rom_bank, ram_enabled),

            Mbc::Mbc3 {
                rom_bank,
                ram_bank,
                ram_enabled,
            } => Self::write_mbc3(&mut self.ram, address, value, rom_bank, ram_bank, ram_enabled),

            Mbc::Mbc5 {
                rom_bank,
                ram_bank,
                ram_enabled,
            } => Self::write_mbc5(&mut self.ram, address, value, rom_bank, ram_bank, ram_enabled),
        }
    }

    pub fn header(&self) -> &CartridgeHeader {
        &self.header
    }

    fn read_rom_only(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0xA000..=0xBFFF => {
                let offset = (address - 0xA000) as usize;
                self.ram.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_rom_only(&mut self, address: u16, value: u8) {
        if let 0xA000..=0xBFFF = address {
            let offset = (address - 0xA000) as usize;
            if let Some(byte) = self.ram.get_mut(offset) {
                *byte = value;
            }
        }
    }

    fn read_mbc1(&self, address: u16, rom_bank: u8, ram_bank: u8, ram_enabled: bool, banking_mode: u8, ) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let upper = if banking_mode == 0 { 0 } else { (ram_bank as usize) << 5 };
                let bank = upper;
                let index = bank * 0x4000 + address as usize;
                self.rom.get(index).copied().unwrap_or(0xFF)
            }

            0x4000..=0x7FFF => {
                let mut bank = (rom_bank & 0x1F) as usize;
                if bank == 0 {
                    bank = 1;
                }

                let upper = (ram_bank as usize) << 5;
                let bank = if banking_mode == 0 { bank | upper } else { bank };
                let index = bank * 0x4000 + (address as usize - 0x4000);
                self.rom.get(index).copied().unwrap_or(0xFF)
            }

            0xA000..=0xBFFF => {
                if !ram_enabled {
                    return 0xFF;
                }

                let bank = if banking_mode == 0 { 0 } else { ram_bank as usize };
                let index = bank * 0x2000 + (address as usize - 0xA000);
                self.ram.get(index).copied().unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    fn write_mbc1(ram: &mut [u8], address: u16, value: u8, rom_bank: &mut u8, ram_bank: &mut u8, ram_enabled: &mut bool, banking_mode: &mut u8, ) {
        match address {
            0x0000..=0x1FFF => {
                *ram_enabled = (value & 0x0F) == 0x0A;
            }

            0x2000..=0x3FFF => {
                let mut bank = value & 0x1F;
                if bank == 0 {
                    bank = 1;
                }
                *rom_bank = bank;
            }

            0x4000..=0x5FFF => {
                *ram_bank = value & 0x03;
            }

            0x6000..=0x7FFF => {
                *banking_mode = value & 0x01;
            }

            0xA000..=0xBFFF => {
                if !*ram_enabled {
                    return;
                }

                let bank = if *banking_mode == 0 { 0 } else { *ram_bank as usize };
                let index = bank * 0x2000 + (address as usize - 0xA000);

                if let Some(byte) = ram.get_mut(index) {
                    *byte = value;
                }
            }

            _ => {}
        }
    }

    fn read_mbc2(&self, address: u16, rom_bank: u8, ram_enabled: bool) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),

            0x4000..=0x7FFF => {
                let bank = if rom_bank == 0 { 1 } else { rom_bank as usize };
                let index = bank * 0x4000 + (address as usize - 0x4000);
                self.rom.get(index).copied().unwrap_or(0xFF)
            }

            0xA000..=0xA1FF => {
                if !ram_enabled {
                    return 0xFF;
                }
                let index = (address as usize - 0xA000) & 0x01FF;
                self.ram.get(index).map(|v| v | 0xF0).unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    fn write_mbc2(ram: &mut [u8], address: u16, value: u8, rom_bank: &mut u8, ram_enabled: &mut bool, ) {
        match address {
            0x0000..=0x3FFF => {
                if (address & 0x0100) == 0 {
                    *ram_enabled = (value & 0x0F) == 0x0A;
                } else {
                    let mut bank = value & 0x0F;
                    if bank == 0 {
                        bank = 1;
                    }
                    *rom_bank = bank;
                }
            }

            0xA000..=0xA1FF => {
                if !*ram_enabled {
                    return;
                }

                let index = (address as usize - 0xA000) & 0x01FF;
                if let Some(byte) = ram.get_mut(index) {
                    *byte = value & 0x0F;
                }
            }

            _ => {}
        }
    }

    fn read_mbc3(&self, address: u16, rom_bank: u8, ram_bank: u8, ram_enabled: bool) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),

            0x4000..=0x7FFF => {
                let bank = if rom_bank == 0 { 1 } else { rom_bank as usize };
                let index = bank * 0x4000 + (address as usize - 0x4000);
                self.rom.get(index).copied().unwrap_or(0xFF)
            }

            0xA000..=0xBFFF => {
                if !ram_enabled {
                    return 0xFF;
                }

                let bank = ram_bank as usize;
                let index = bank * 0x2000 + (address as usize - 0xA000);
                self.ram.get(index).copied().unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    fn write_mbc3(ram: &mut [u8], address: u16, value: u8, rom_bank: &mut u8, ram_bank: &mut u8, ram_enabled: &mut bool, ) {
        match address {
            0x0000..=0x1FFF => {
                *ram_enabled = (value & 0x0F) == 0x0A;
            }

            0x2000..=0x3FFF => {
                let bank = value & 0x7F;
                *rom_bank = if bank == 0 { 1 } else { bank };
            }

            0x4000..=0x5FFF => {
                *ram_bank = value;
            }

            0xA000..=0xBFFF => {
                if !*ram_enabled {
                    return;
                }

                if *ram_bank <= 0x03 {
                    let bank = *ram_bank as usize;
                    let index = bank * 0x2000 + (address as usize - 0xA000);
                    if let Some(byte) = ram.get_mut(index) {
                        *byte = value;
                    }
                }
            }

            _ => {}
        }
    }

    fn read_mbc5(&self, address: u16, rom_bank: u16, ram_bank: u8, ram_enabled: bool) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),

            0x4000..=0x7FFF => {
                let bank = rom_bank as usize;
                let index = bank * 0x4000 + (address as usize - 0x4000);
                self.rom.get(index).copied().unwrap_or(0xFF)
            }

            0xA000..=0xBFFF => {
                if !ram_enabled {
                    return 0xFF;
                }

                let bank = ram_bank as usize;
                let index = bank * 0x2000 + (address as usize - 0xA000);
                self.ram.get(index).copied().unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    fn write_mbc5(ram: &mut [u8], address: u16, value: u8, rom_bank: &mut u16, ram_bank: &mut u8, ram_enabled: &mut bool, ) {
        match address {
            0x0000..=0x1FFF => {
                *ram_enabled = (value & 0x0F) == 0x0A;
            }

            0x2000..=0x2FFF => {
                *rom_bank = (*rom_bank & 0x100) | value as u16;
            }

            0x3000..=0x3FFF => {
                *rom_bank = (*rom_bank & 0x0FF) | (((value & 0x01) as u16) << 8);
            }

            0x4000..=0x5FFF => {
                *ram_bank = value & 0x0F;
            }

            0xA000..=0xBFFF => {
                if !*ram_enabled {
                    return;
                }

                let bank = *ram_bank as usize;
                let index = bank * 0x2000 + (address as usize - 0xA000);
                if let Some(byte) = ram.get_mut(index) {
                    *byte = value;
                }
            }

            _ => {}
        }
    }

    pub fn read_rom_byte_abs(&self, offset: usize) -> u8 {
        self.rom.get(offset).copied().unwrap_or(0xFF)
    }
}
