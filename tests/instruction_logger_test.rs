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
use vgbe::cpu::ExecutedInstruction;
use vgbe::emulator::GameBoy;
use vgbe::instruction_logger::InstructionLogger;

fn run_until_instruction(gameboy: &mut GameBoy) -> ExecutedInstruction {
    loop {
        if let Some(executed_instruction) = gameboy.tick() {
            return executed_instruction;
        }
    }
}

#[test]
fn test_debug_instruction_logger_creates_timestamped_rom_log() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));
    let rom = format!("roms/logger-smoke-{}.gb", std::process::id());
    let mut logger = InstructionLogger::new(&rom);

    let executed_instruction = run_until_instruction(&mut gameboy);
    logger
        .log_if_debug(true, &executed_instruction, gameboy.cpu())
        .unwrap();
    logger.flush().unwrap();

    let path = logger.path().unwrap().to_path_buf();
    let file_name = path.file_name().unwrap().to_string_lossy();

    assert!(path.starts_with("logs"));
    assert!(file_name.contains("logger-smoke"));
    assert!(file_name.ends_with(".log"));

    drop(logger);

    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("0x0100:"));
    assert!(contents.contains("\n\tA:"));
    assert!(contents.contains("  F:"));
    assert!(contents.contains("\n\tIME:"));
    assert!(contents.contains("  IE:"));
    assert!(contents.contains("  IF:"));
    assert!(contents.ends_with("\n\n"));

    fs::remove_file(path).unwrap();
}

#[test]
fn test_instruction_logger_does_not_create_file_when_debug_is_disabled() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));
    let rom = format!("roms/logger-disabled-{}.gb", std::process::id());
    let mut logger = InstructionLogger::new(&rom);

    let executed_instruction = run_until_instruction(&mut gameboy);
    logger
        .log_if_debug(false, &executed_instruction, gameboy.cpu())
        .unwrap();

    assert!(logger.path().is_none());
}

#[test]
fn test_instruction_logger_can_include_timing_context() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(Rc::clone(&memory));

    let executed_instruction = run_until_instruction(&mut gameboy);
    let entry = InstructionLogger::format_entry_with_timing(
        &executed_instruction,
        gameboy.cpu(),
        gameboy.cycle(),
        gameboy.ppu().dots(),
    );

    assert!(entry.contains("\n\tTIMING:M-CYCLE:"));
    assert!(entry.contains(" DOTS:"));
    assert!(entry.ends_with("\n\n"));
}
