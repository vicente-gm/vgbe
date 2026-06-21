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

use crate::ppu::registers::*;

use super::*;

impl MemoryBus {
    pub fn load_boot_rom(&mut self, data: Vec<u8>) {
        self.boot_rom = Some(data);
        self.boot_rom_enabled = true;
    }

    pub fn init_post_boot_dmg(&mut self) {
        self.div_counter = 0;

        self.write_byte(0xFF04, 0x00);
        self.write_byte(0xFF05, 0x00);
        self.write_byte(0xFF06, 0x00);
        self.write_byte(0xFF07, 0x00);

        self.init_post_boot_apu_dmg();

        self.write_byte(LCDC_ADDR, 0x91);
        self.set_stat_mode_from_ppu(STAT_MODE_OAM);
        self.set_ly(0x00);
        self.write_byte(SCY_ADDR, 0x00);
        self.write_byte(SCX_ADDR, 0x00);
        self.write_byte(LYC_ADDR, 0x00);
        self.write_byte(BGP_ADDR, 0xFC);
        self.write_byte(OBP0_ADDR, 0xFF);
        self.write_byte(OBP1_ADDR, 0xFF);
        self.write_byte(WY_ADDR, 0x00);
        self.write_byte(WX_ADDR, 0x00);

        self.write_byte(0xFFFF, 0x00);
        self.write_byte(0xFF0F, 0xE1);
    }

    pub(crate) fn init_post_boot_apu_dmg(&mut self) {
        self.write_byte(0xFF26, 0x80);

        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);

        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0x00);
        self.write_byte(0xFF19, 0xBF);

        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xBF);

        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0x00);
        self.write_byte(0xFF22, 0x00);
        self.write_byte(0xFF23, 0xBF);

        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
    }
}
