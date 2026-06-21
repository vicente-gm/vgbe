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
use vgbe::*;
use vgbe::cpu::registers::*;

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::rc::Rc;
use vgbe::bus::MemoryBus;

static MAX_CYCLES: u32 = 500000;

fn create_cpu(file: &mut File, rom: &String) -> cpu::Cpu {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::load_rom(file, rom)));
    let mut my_cpu = cpu::Cpu::new(Rc::clone(&memory));

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

fn create_log(rom: &String) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("logs/{rom}.log"))
}

fn create_opcodes_log(rom: &String) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("logs/{rom}-opcodes.log"))
}

fn write_log(logs: &mut File, opcodes: &mut File, my_cpu: &mut cpu::Cpu) -> io::Result<()> {
    let pc: u16 = my_cpu.get_register16(Reg16::PC);
    let content = format!(
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
        my_cpu.get_register8(Reg8::A),
        my_cpu.get_register8(Reg8::F),
        my_cpu.get_register8(Reg8::B),
        my_cpu.get_register8(Reg8::C),
        my_cpu.get_register8(Reg8::D),
        my_cpu.get_register8(Reg8::E),
        my_cpu.get_register8(Reg8::H),
        my_cpu.get_register8(Reg8::L),
        my_cpu.get_register16(Reg16::SP),
        pc,
        my_cpu.read_memory(pc),
        my_cpu.read_memory(pc.wrapping_add(1)),
        my_cpu.read_memory(pc.wrapping_add(2)),
        my_cpu.read_memory(pc.wrapping_add(3)),
    );

    opcodes.write_all(format!("0b{:04b}_{:04b}: {}", my_cpu.opcode >> 4, my_cpu.opcode & 0x0F, content).as_bytes())?;
    logs.write_all(content.as_bytes())
}

#[test]
fn test_blargg_complete() {
    let rom: String = String::from("roms/test/blargg/cpu_instrs/cpu_instrs.gb");
    let file = File::open(&rom);

    let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
    let mut file = create_log(&String::from("blargg-cpu_instrs")).unwrap();
    let mut opcodes = create_opcodes_log(&String::from("blargg-cpu_instrs")).unwrap();

    for _ in 0..MAX_CYCLES {
        let _ = my_cpu.tick();
        if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
    }
}


#[cfg(test)]
mod blargg_test {
    use std::fs::File;
    use crate::{create_cpu, create_log, create_opcodes_log, write_log, MAX_CYCLES};

    #[test]
    fn test_blargg_01() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/01-special.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-01")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-01")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); } }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_02() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/02-interrupts.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-02")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-02")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_03() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/03-op sp,hl.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-03")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-03")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_04() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/04-op r,imm.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-04")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-04")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_05() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/05-op rp.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-05")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-05")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_06() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/06-ld r,r.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-06")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-06")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_07() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-07")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-07")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_08() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/08-misc instrs.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-08")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-08")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_09() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/09-op r,r.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-09")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-09")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_10() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/10-bit ops.gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-10")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-10")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }

    #[test]
    fn test_blargg_11() {
        let rom: String = String::from("roms/test/blargg/cpu_instrs/individual/11-op a,(hl).gb");
        let file = File::open(&rom);

        let mut my_cpu = create_cpu(&mut file.unwrap(), &rom);
        let mut file = create_log(&String::from("blargg-11")).unwrap();
        let mut opcodes = create_opcodes_log(&String::from("blargg-11")).unwrap();

        for _ in 0..MAX_CYCLES {
            if my_cpu.blocked == 0 { write_log(&mut file, &mut opcodes, &mut my_cpu).unwrap(); }
            let _ = my_cpu.tick();
        }
    }
}
