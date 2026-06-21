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

use super::*;

impl MemoryBus {
    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00FF if self.boot_rom_enabled => self
                .boot_rom
                .as_ref()
                .and_then(|rom| rom.get(address as usize))
                .copied()
                .unwrap_or(0xFF),

            0x0000..=0x7FFF => self.cartridge.read(address),
            0xA000..=0xBFFF => self.cartridge.read(address),

            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00..=0xFF7F => self.read_io(address),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie,
            _ => !unreachable!(),
        }
    }

    #[inline]
    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cartridge.write(address, value),
            0xA000..=0xBFFF => self.cartridge.write(address, value),

            0x8000..=0x9FFF => {
                self.vram[(address - 0x8000) as usize] = value;
            }

            0xC000..=0xDFFF => {
                self.wram[(address - 0xC000) as usize] = value;
            }

            0xE000..=0xFDFF => {
                self.wram[(address - 0xE000) as usize] = value;
            }

            0xFE00..=0xFE9F => {
                self.oam[(address - 0xFE00) as usize] = value;
            }

            0xFEA0..=0xFEFF => {}
            0xFF00..=0xFF7F => self.write_io(address, value),

            0xFF80..=0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = value;
            }

            0xFFFF => {
                self.ie = value;
            }

            _ => !unreachable!(),
        }
    }
}
