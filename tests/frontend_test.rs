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

#![cfg(feature = "sdl")]

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
use vgbe::bus::*;
use vgbe::cpu::interrupts::*;
use vgbe::cpu::registers::{FLAG_C, FLAG_H, FLAG_Z, Reg8, Reg16};
use vgbe::frontend::window::Window;
use vgbe::instruction_logger::InstructionLogger;
use vgbe::*;

fn create_cpu(mem: Rc<RefCell<MemoryBus>>) -> cpu::Cpu {
    let mut my_cpu = cpu::Cpu::new(Rc::clone(&mem));

    // Initialize the CPU to run the tests
    let flags: u8 = FLAG_C | FLAG_H | FLAG_Z;
    my_cpu.set_register8(Reg8::A, 0x01);
    my_cpu.set_register8(Reg8::F, flags);
    my_cpu.set_register8(Reg8::B, 0x00);
    my_cpu.set_register8(Reg8::C, 0x13);
    my_cpu.set_register8(Reg8::D, 0x00);
    my_cpu.set_register8(Reg8::E, 0xD8);
    my_cpu.set_register8(Reg8::H, 0x01);
    my_cpu.set_register8(Reg8::L, 0x4D);
    my_cpu.set_register16(Reg16::SP, 0xFFFE);
    my_cpu.set_register16(Reg16::PC, 0x0100);

    my_cpu
}

fn create_ppu(mem: Rc<RefCell<MemoryBus>>) -> ppu::Ppu {
    let my_ppu = ppu::Ppu::new(Rc::clone(&mem));
    my_ppu
}

#[test]
#[ignore = "Opens an SDL window for manual frontend inspection."]
fn test_frontend() {
    let memory :Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut my_cpu = create_cpu(Rc::clone(&memory));
    let my_ppu = create_ppu(Rc::clone(&memory));

    let mut window = Window::new(true);

    let mut i: u8 = 0;

    while window.handle_events() {
        let _ = my_cpu.tick();

        if i == 100 {
            window.render(&my_cpu, &my_ppu);
            i = 0;
        }
        i += 1;
    }
}

#[test]
#[ignore = "Opens an SDL window for manual tiles panel inspection."]
fn test_tiles_panel() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    let mut test_tiles: Vec<u8> = Vec::new();

    // TILE 0: El original de la documentación (una forma extraña)
    test_tiles.extend_from_slice(&[
        0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56, 0x38,
        0x7C,
    ]);

    // TILE 1: Degradado de los 4 colores (Barras horizontales)
    // Color 0 (Blanco), 1 (Gris claro), 2 (Gris oscuro), 3 (Negro)
    test_tiles.extend_from_slice(&[
        0x00, 0x00, // Fila 0: Color 0 (00 binario)
        0x00, 0x00, // Fila 1: Color 0
        0xFF, 0x00, // Fila 2: Color 1 (01 binario)
        0xFF, 0x00, // Fila 3: Color 1
        0x00, 0xFF, // Fila 4: Color 2 (10 binario)
        0x00, 0xFF, // Fila 5: Color 2
        0xFF, 0xFF, // Fila 6: Color 3 (11 binario)
        0xFF, 0xFF, // Fila 7: Color 3
    ]);

    // TILE 2: Un recuadro/borde (útil para ver si los tiles se pegan bien)
    test_tiles.extend_from_slice(&[
        0xFF, 0xFF, // Top negro
        0x81, 0x81, // Lados negros, centro blanco
        0x81, 0x81,
        0x81, 0x81,
        0x81, 0x81,
        0x81, 0x81,
        0x81, 0x81,
        0xFF, 0xFF, // Bottom negro
    ]);

    // TILE 3: Tablero de ajedrez (píxeles alternos)
    test_tiles.extend_from_slice(&[
        0xAA, 0xAA, 0x55, 0x55, 0xAA, 0xAA, 0x55, 0x55,
        0xAA, 0xAA, 0x55, 0x55, 0xAA, 0xAA, 0x55, 0x55,
    ]);

    // TILE 4: Una "X" grande
    test_tiles.extend_from_slice(&[
        0x81, 0x81, 0x42, 0x42, 0x24, 0x24, 0x18, 0x18,
        0x18, 0x18, 0x24, 0x24, 0x42, 0x42, 0x81, 0x81,
    ]);

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        for (offset, byte) in test_tiles.iter().enumerate() {
            mem.write_byte(0x8000 + offset as u16, *byte);
        }
    }

    let mut my_cpu = create_cpu(Rc::clone(&memory));
    let my_ppu = create_ppu(Rc::clone(&memory));
    my_cpu.init_post_boot_dmg();

    let mut window = Window::new(true);

    let mut i: u8 = 0;

    while window.handle_events() {
        let _ = my_cpu.tick();

        if i == 100 {
            window.render(&my_cpu, &my_ppu);
            i = 0;
        }
        i += 1;
    }
}

#[test]
#[ignore = "Opens an SDL window for manual Tetris tiles inspection."]
fn test_tiles_tetris() {
    let rom = String::from("roms/Tetris.gb");
    let mut file = File::open(&rom)
        .expect("Could not open ROM file");

    let memory: Rc<RefCell<MemoryBus>> =
        Rc::new(RefCell::new(MemoryBus::load_rom(&mut file, &rom)));

    let mut my_cpu = create_cpu(Rc::clone(&memory));
    let my_ppu = create_ppu(Rc::clone(&memory));

    my_cpu.init_post_boot_dmg();

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        my_cpu.init_post_boot_dmg();

        mem.debug_copy_tiles_to_vram(0x3000, 0, 128);
        mem.debug_copy_tiles_to_vram(0x3800, 128, 128);
        mem.debug_copy_tiles_to_vram(0x4800, 256, 128);
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    while window.handle_events() {
        if let Some(executed_instruction) = my_cpu.tick() {
            instruction_logger
                .log_if_debug(window.is_debug_mode(), &executed_instruction, &my_cpu)
                .unwrap();
        }
        window.render(&my_cpu, &my_ppu);
    }
}

#[test]
fn test_empty_memory_cpu_ppu_vblank() {
    let memory: Rc<RefCell<MemoryBus>> =
        Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
    }

    let mut my_cpu = create_cpu(Rc::clone(&memory));
    let mut my_ppu = create_ppu(Rc::clone(&memory));

    my_cpu.init_post_boot_dmg();

    // Ejecutamos un frame completo hasta entrar en VBlank.
    //
    // 1 scanline = 456 dots
    // VBlank empieza en LY = 144
    // Por tanto: 456 * 144 ticks llevan desde LY=0 hasta LY=144.
    for _ in 0..(456 * 144) {
        let _ = my_cpu.tick();
        my_ppu.tick();
    }

    {
        let mem = memory.borrow();

        assert_eq!(mem.read_byte(0xFF44), 144);
        assert!(my_ppu.take_frame_ready());

        let iflag = mem.read_byte(INTERRUPT_FLAG_DIR);
        assert_ne!(iflag & (1 << INTERRUPT_VBLANK_BIT), 0);
    }
}
