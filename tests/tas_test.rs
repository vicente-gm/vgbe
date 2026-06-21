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
use std::fs;
use std::rc::Rc;

use vgbe::bus::MemoryBus;
use vgbe::cpu::interrupts::INTERRUPT_JOYPAD_BIT;
use vgbe::emulator::GameBoy;
use vgbe::tas::{TasKey, parse_tas_file};

#[test]
fn test_parse_tas_file_format() {
    let events = parse_tas_file("0 w 1\n12 k 0\n").unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].cycle, 0);
    assert_eq!(events[0].key, TasKey::W);
    assert!(events[0].pressed);
    assert_eq!(events[1].cycle, 12);
    assert_eq!(events[1].key, TasKey::K);
    assert!(!events[1].pressed);
}

#[test]
fn test_gameboy_tas_file_updates_joypad_at_cpu_cycles() {
    let tas_path = std::env::temp_dir().join(format!("vgbe-tas-{}.txt", std::process::id()));
    fs::write(&tas_path, "0 k 1\n2 k 0\n").unwrap();

    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));
    gameboy.load_tas_file(&tas_path).unwrap();

    let _ = gameboy.tick();
    {
        let mem = memory.borrow();
        assert_eq!(mem.read_byte(0xFF00) & 0x01, 0);
        assert_ne!(mem.read_byte(0xFF0F) & (1 << INTERRUPT_JOYPAD_BIT), 0);
    }

    let _ = gameboy.tick();
    let _ = gameboy.tick();
    assert_eq!(memory.borrow().read_byte(0xFF00) & 0x01, 1);

    fs::remove_file(tas_path).unwrap();
}

#[test]
fn test_direction_keys_read_as_active_low_joypad_bits() {
    let cases = [
        (TasKey::D, 0b1110),
        (TasKey::A, 0b1101),
        (TasKey::W, 0b1011),
        (TasKey::S, 0b0111),
    ];

    for (key, expected_low_nibble) in cases {
        let mut memory = MemoryBus::init_mem_void();
        memory.write_byte(0xFF00, 0x20); // Select Game Boy directions.

        memory.set_tas_key_state(key, true);

        assert_eq!(memory.read_byte(0xFF00) & 0x0F, expected_low_nibble);
    }
}

#[test]
fn test_w_key_is_ignored_when_buttons_are_selected() {
    let mut memory = MemoryBus::init_mem_void();
    memory.write_byte(0xFF00, 0x10); // Select Game Boy buttons, not directions.

    memory.set_tas_key_state(TasKey::W, true);

    assert_eq!(memory.read_byte(0xFF00) & 0x0F, 0x0F);
}

#[test]
fn test_optional_tas_file_can_be_absent() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));

    gameboy.maybe_load_tas_file(None::<&str>).unwrap();
    let _ = gameboy.tick();

    assert_eq!(gameboy.cycle(), 1);
}
