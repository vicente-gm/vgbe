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

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

use vgbe::bus::MemoryBus;
use vgbe::cpu::registers::{FLAG_C, FLAG_H, FLAG_Z, Reg8, Reg16};
use vgbe::emulator::GameBoy;

const GAME_BOY_FRAME_M_CYCLES: usize = 17_556;

#[test]
fn test_gameboy_tick_advances_ppu_four_dots_per_cpu_mcycle() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));

    for _ in 0..114 {
        let _ = gameboy.tick();
    }

    assert_eq!(memory.borrow().read_byte(0xFF44), 1);
}

#[test]
fn test_gameboy_load_rom_initializes_cpu_post_boot_dmg() {
    let rom = String::from("roms/Tetris.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let gameboy = GameBoy::load_rom(&mut file, &rom);

    assert_eq!(gameboy.cpu().get_register8(Reg8::A), 0x01);
    assert_eq!(gameboy.cpu().get_register8(Reg8::F), FLAG_Z | FLAG_H | FLAG_C);
    assert_eq!(gameboy.cpu().get_register8(Reg8::B), 0x00);
    assert_eq!(gameboy.cpu().get_register8(Reg8::C), 0x13);
    assert_eq!(gameboy.cpu().get_register8(Reg8::D), 0x00);
    assert_eq!(gameboy.cpu().get_register8(Reg8::E), 0xD8);
    assert_eq!(gameboy.cpu().get_register8(Reg8::H), 0x01);
    assert_eq!(gameboy.cpu().get_register8(Reg8::L), 0x4D);
    assert_eq!(gameboy.cpu().get_register16(Reg16::SP), 0xFFFE);
    assert_eq!(gameboy.cpu().get_register16(Reg16::PC), 0x0100);
}

#[test]
fn test_gameboy_reports_emulation_frame_ready_at_game_boy_frame_rate() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));

    for _ in 0..(GAME_BOY_FRAME_M_CYCLES - 1) {
        let _ = gameboy.tick();
    }

    assert!(!gameboy.take_emulation_frame_ready());

    let _ = gameboy.tick();
    assert!(gameboy.take_emulation_frame_ready());
    assert!(!gameboy.take_emulation_frame_ready());
}
