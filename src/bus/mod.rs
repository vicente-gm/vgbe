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

use crate::apu::{SharedApu, new_shared_dummy_apu};
use crate::cartridge::Cartridge;
use crate::tas::{TasInputState, TasKey};

mod boot;
mod dma;
mod io;
mod memory_map;
mod timer;

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

const TAC_ENABLE: u8 = 0b0000_0100;
const TAC_CLOCK_SELECT_MASK: u8 = 0b0000_0011;

#[derive(Debug)]
pub struct MemoryBus {
    cartridge: Cartridge,

    boot_rom: Option<Vec<u8>>,
    boot_rom_enabled: bool,

    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    ie: u8,
    apu: SharedApu,

    div_counter: u16,
    tas_input: TasInputState,
}

impl MemoryBus {
    pub fn load_rom(file: &mut File, name: &str) -> MemoryBus {
        let mut mem = MemoryBus {
            cartridge: Cartridge::new(file, name),
            boot_rom: None,
            boot_rom_enabled: false,
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
            apu: new_shared_dummy_apu(),
            div_counter: 0,
            tas_input: TasInputState::default(),
        };

        mem.init_post_boot_dmg();
        mem
    }

    pub fn init_mem_void() -> MemoryBus {
        let mut mem = MemoryBus {
            cartridge: Cartridge::new_void(),
            boot_rom: None,
            boot_rom_enabled: false,
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
            apu: new_shared_dummy_apu(),
            div_counter: 0,
            tas_input: TasInputState::default(),
        };

        mem.init_post_boot_dmg();
        mem
    }

    pub fn debug_copy_rom_to_vram(&mut self, rom_offset: usize, vram_addr: u16, len: usize) {
        for i in 0..len {
            let value = self.cartridge.read_rom_byte_abs(rom_offset + i);
            self.write_byte(vram_addr + i as u16, value);
        }
    }

    pub fn debug_copy_tiles_to_vram(
        &mut self,
        rom_offset: usize,
        first_tile: u16,
        num_tiles: usize,
    ) {
        let vram_addr = 0x8000 + first_tile * 16;
        let len = num_tiles * 16;
        self.debug_copy_rom_to_vram(rom_offset, vram_addr, len);
    }

    pub fn set_tas_key_state(&mut self, key: TasKey, pressed: bool) -> bool {
        self.tas_input.set(key, pressed)
    }

    pub fn tas_input_state(&self) -> TasInputState {
        self.tas_input
    }

    pub fn set_apu(&mut self, apu: SharedApu) {
        self.apu = apu;
    }
}
