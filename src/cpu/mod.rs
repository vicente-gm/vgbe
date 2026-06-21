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

//! Game Boy CPU implementation.
//!
//! This module implements the core CPU used by the emulator. It is responsible
//! for maintaining the CPU state, fetching and executing instructions, and
//! interacting with the memory bus.
//!
//! The CPU executes instructions cycle by cycle using the `tick` method.
//! Each tick advances the internal cycle counter and may execute or continue
//! the execution of the current instruction.

pub mod registers;
pub mod interrupts;
pub mod instructions;

pub use instructions::{DisassembledInstruction, Instruction};

use registers::*;
use interrupts::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

use instructions::OPCODES_TABLE;
use instructions::CB_OPCODES_TABLE;
use instructions::unknown;

use crate::bus::MemoryBus;

#[derive(Debug, Copy, Clone)]
pub struct ExecutedInstruction {
    pub address: u16,
    pub instr: Instruction,
}

/// Represents the Game Boy CPU and its internal state.
///
/// The CPU stores all main registers, flags, the program counter,
/// stack pointer, and the memory bus used to communicate with the rest of
/// the system.
///
/// Instructions are fetched and executed through the `tick` method, which
/// simulates a single CPU cycle.
#[derive(Debug)]
pub struct Cpu {
    /// Accumulator register.
    a: u8,

    /// Flags register.
    f: u8,

    /// B register.
    b: u8,

    /// C register.
    c: u8,

    /// D register.
    d: u8,

    /// E register.
    e: u8,

    /// H register.
    h: u8,

    /// L register.
    l: u8,

    /// Stack pointer.
    sp: u16,

    /// Program counter.
    pc: u16,

    /// Interrupt Master Enable flag. It controls whether any interrupt handlers are called.
    ime: bool,

    /// Indicates if IME is pending to be enabled.
    ime_delay: u8,

    /// Indicates whether the CPU is in low-power consumption mode.
    halted: bool,

    /// Indicates whether the CPU has to implement the HALT bug.
    halt_bug: bool,

    /// Indicates whether the CPU is in super low-power consumption mode.
    stopped: bool,

    /// Pending frontend joypad event that can wake STOP mode.
    joypad_stop_wake_requested: bool,

    /// Total number of cycles executed.
    cycles: usize,

    /// Memory bus used to access system memory.
    bus: Rc<RefCell<MemoryBus>>,

    /// Remaining cycles before the CPU can fetch the next instruction.
    ///
    /// Some instructions take multiple cycles to complete. During that time
    /// the CPU is considered "blocked".
    pub blocked: u8,

    /// Current instruction being executed.
    instr_queue: VecDeque<DisassembledInstruction>,

    /// Opcode of the current instruction.
    pub opcode: u8
}

impl Cpu {
    pub const NUM_DISASSEMBLED_INSTRUCTIONS: usize = 20;

    /// Creates a new CPU instance.
    ///
    /// The program counter is initialized to `0x100`, which corresponds to the
    /// start of the Game Boy cartridge code after the boot ROM.
    ///
    /// # Arguments
    ///
    /// * `rom` - Memory bus connected to the cartridge and system memory.
    ///
    /// # Returns
    ///
    /// A new initialized `Cpu`.
    pub fn new(rom: Rc<RefCell<MemoryBus>>) -> Cpu {
        Cpu {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0x100,  // Memory direction of the cartridge header
            ime: false,
            ime_delay: 0,
            halted: false,
            halt_bug: false,
            stopped: false,
            joypad_stop_wake_requested: false,
            cycles: 0,
            bus: Rc::clone(&rom),
            blocked: 0,
            instr_queue: {
                let mut queue: VecDeque<DisassembledInstruction> = VecDeque::new();
                for _ in 0..Self::NUM_DISASSEMBLED_INSTRUCTIONS{
                    queue.push_back(DisassembledInstruction {address: 0x0, instr: Instruction { func: unknown, cycles: 0, bytes: 0, asm: "UNKNOWN" }});
                }
                queue
            },
            opcode: 0
        }
    }

    /// Returns the AF register pair.
    #[inline(always)]
    fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    /// Sets the AF register pair.
    ///
    /// The lower 4 bits of `F` are always cleared, as they are unused in the Game Boy CPU.
    #[inline(always)]
    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value as u8) & 0xF0;
    }

    /// Returns the BC register pair.
    #[inline(always)]
    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Sets the BC register pair.
    #[inline(always)]
    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    /// Returns the DE register pair.
    #[inline(always)]
    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Sets the DE register pair.
    #[inline(always)]
    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    /// Returns the HL register pair.
    #[inline(always)]
    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Sets the HL register pair.
    #[inline(always)]
    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    /// Set the IME flag.
    #[inline(always)]
    fn set_ime(&mut self, value: bool) { self.ime = value; }

    /// Returns the IME flag.
    #[inline(always)]
    fn get_ime(&mut self) -> bool { self.ime }

    /// Set the IME_enabled variable.
    #[inline(always)]
    fn set_ime_delay(&mut self, value: u8) { self.ime_delay = value; }

    /// Returns Interrupt Enable register
    #[inline(always)]
    fn get_ie(&mut self) -> u8 {  self.bus.borrow().read_byte(INTERRUPT_ENABLE_DIR) }

    /// Returns Interrupt Flag register
    #[inline(always)]
    fn get_if(&mut self) -> u8 { self.bus.borrow().read_byte(INTERRUPT_FLAG_DIR) }

    /// Set the halted state of the CPU
    #[inline(always)]
    fn set_halted(&mut self, value: bool) { self.halted = value; }

    /// Set the halt_bug variable.
    #[inline(always)]
    fn set_halt_bug(&mut self, value: bool) { self.halt_bug = value; }

    /// Returns the halt_bug variable.
    #[inline(always)]
    fn get_halt_bug(&mut self) -> bool { self.halt_bug }

    /// Returns the halted state of the CPU.
    #[inline(always)]
    fn is_halted(&mut self) -> bool { self.halted }

    /// Set the halted state of the CPU.
    #[inline(always)]
    fn set_stopped(&mut self, value: bool) { self.stopped = value; }

    /// Returns the halted state of the CPU.
    #[inline(always)]
    fn is_stopped(&mut self) -> bool { self.stopped }

    /// Returns whether the CPU is blocked executing an instruction.
    pub fn is_blocked(&self) -> bool {
        self.blocked > 0 && !self.halted && !self.stopped
    }

    /// Inserts one instruction to the back of the queue and pop one from the front.
    #[inline(always)]
    fn insert_instruction_to_queue(&mut self, addr: u16, instr: Instruction) {
        self.instr_queue.push_back(DisassembledInstruction {address: addr, instr: instr});
        self.instr_queue.pop_front();
    }

    /// Returns the instruction queue.
    pub fn get_instruction_queue(&self) -> &VecDeque<DisassembledInstruction> {
        &self.instr_queue
    }

    pub fn interrupt_master_enabled(&self) -> bool {
        self.ime
    }

    pub fn interrupt_enable(&self) -> u8 {
        self.bus.borrow().read_byte(INTERRUPT_ENABLE_DIR)
    }

    pub fn interrupt_flag(&self) -> u8 {
        self.bus.borrow().read_byte(INTERRUPT_FLAG_DIR)
    }

    /// Sets the value of a 16 bits CPU register.
    ///
    /// When writing to `AF`, the lower four bits of `F` are always cleared,
    /// since they are unused in the Game Boy CPU.
    ///
    /// # Arguments
    ///
    /// * `reg` - Register identifier.
    /// * `value` - Value to write to the register.
    #[inline]
    pub fn set_register16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::AF => self.set_af(value),
            Reg16::BC => self.set_bc(value),
            Reg16::DE => self.set_de(value),
            Reg16::HL => self.set_hl(value),
            Reg16::SP => self.sp = value,
            Reg16::PC => self.pc = value,
        }
    }

    /// Sets the value of a 8 bits CPU register.
    ///
    /// When writing to `F`, the lower four bits are always cleared,
    /// since they are unused in the Game Boy CPU.
    ///
    /// # Arguments
    ///
    /// * `reg` - Register identifier.
    /// * `value` - Value to write to the register.
    #[inline]
    pub fn set_register8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::F => self.f = value & 0xF0,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
        }
    }

    /// Returns the value of a 16 bits CPU register.
    ///
    /// # Arguments
    ///
    /// * `reg` - Register identifier.
    ///
    /// # Returns
    ///
    /// The current value stored in the register.
    #[inline]
    pub fn get_register16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => self.af(),
            Reg16::BC => self.bc(),
            Reg16::DE => self.de(),
            Reg16::HL => self.hl(),
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
        }
    }

    /// Returns the value of a 8 bits CPU register.
    ///
    /// # Arguments
    ///
    /// * `reg` - Register identifier.
    ///
    /// # Returns
    ///
    /// The current value stored in the register.
    #[inline]
    pub fn get_register8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::F => self.f,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn request_joypad_interrupt(&mut self) {
        self.joypad_stop_wake_requested = true;

        self.bus
            .borrow_mut()
            .request_interrupt(Interrupt::JOYPAD);
    }

    /// Advances the CPU by one clock cycle.
    ///
    /// This function performs the instruction fetch-decode-execute cycle.
    /// If the CPU is not currently blocked by a multi-cycle instruction,
    /// a new opcode is fetched and executed.
    ///
    /// Prefixed (`0xCB`) instructions are handled using a separate opcode table.
    pub fn tick(&mut self) -> Option<ExecutedInstruction> {
        let mut executed_instruction = None;

        self.cycles += 1;

        self.bus.borrow_mut().tick_timer();

        if self.is_stopped() {
            if !self.joypad_stop_wake_requested {
                return None;
            }

            self.joypad_stop_wake_requested = false;
            self.set_stopped(false);
        }

        let pending_interrupts = (self.get_ie() & self.get_if()) != 0;

        if self.is_halted() {
            if pending_interrupts {
                self.set_halted(false); // Pending interrupts wake up HALT, even when not handled

                if self.handle_interrupts() {
                    self.blocked = self.blocked.saturating_sub(1);
                }
            }

            return None;
        }

        if self.blocked == 0 {
            if !self.handle_interrupts() {
                let mut pc = self.pc;
                self.opcode = self.read_pc();

                let instr = match self.opcode {
                    0xCB => {
                        pc = self.pc;
                        self.opcode = self.read_pc();
                        CB_OPCODES_TABLE[self.opcode as usize]
                    }
                    _ => OPCODES_TABLE[self.opcode as usize],
                };

                let extra_cycles: u8 = (instr.func)(self);
                self.blocked = instr.cycles + extra_cycles;

                self.insert_instruction_to_queue(pc, instr);
                executed_instruction = Some(ExecutedInstruction { address: pc, instr });

                if self.ime_delay > 0 {
                    self.ime_delay = self.ime_delay.saturating_sub(1);
                    if self.ime_delay == 0 {
                        self.set_ime(true);
                    }
                }
            }
        }

        self.blocked = self.blocked.saturating_sub(1);
        executed_instruction
    }

    /// Handles pending CPU interrupts if they are enabled.
    ///
    /// This function checks whether the Interrupt Master Enable (IME) flag is set
    /// and whether any interrupt is both requested and enabled (`IE & IF != 0`).
    /// If so, the highest-priority pending interrupt is serviced.
    ///
    /// Servicing an interrupt performs the following operations:
    /// - Disables further interrupts by clearing the IME flag.
    /// - Clears the corresponding interrupt request bit in the IF register.
    /// - Pushes the current program counter (PC) onto the stack.
    /// - Jumps to the interrupt service routine (ISR) vector address.
    /// - Blocks the CPU for the duration of the interrupt entry sequence.
    ///
    /// Interrupts are processed according to their hardware priority:
    /// VBLANK -> LCD -> TIMER -> SERIAL -> JOYPAD.
    ///
    /// # Returns
    ///
    /// `true` if an interrupt was serviced and execution flow was redirected
    /// to its handler, otherwise `false`.
    #[inline]
    fn handle_interrupts(&mut self) -> bool {
        if !self.ime { return false; }

        let ie = self.get_ie();
        let iflag = self.get_if();
        let pending = ie & iflag;

        if pending == 0 { return false; }

        for &(interrupt, bit, vector) in INTERRUPTS_TABLE {
            if pending & (1 << bit) != 0 {
                self.set_ime(false);

                self.bus.borrow_mut()
                    .clear_interrupt_request(interrupt);

                let pc = self.pc;

                self.sp = self.sp.wrapping_sub(1);
                self.write_memory(self.sp, (pc >> 8) as u8);

                self.sp = self.sp.wrapping_sub(1);
                self.write_memory(self.sp, pc as u8);

                self.pc = vector;
                self.blocked = 5;

                return true;
            }
        }

        false
    }

    /// Increments the program counter.
    #[inline(always)]
    fn pc_inc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// Reads a byte from the address pointed to by the program counter
    /// and then increments the program counter.
    ///
    /// # Returns
    ///
    /// The byte read from memory.
    #[inline(always)]
    fn read_pc(&mut self) -> u8 {
        let value: u8 = self.read_memory(self.pc);

        if !self.get_halt_bug() {
            self.pc_inc();
        } else {
            self.set_halt_bug(false);
        }

        value
    }

    /// Reads a byte from memory.
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address to read.
    ///
    /// # Returns
    ///
    /// The byte stored at the given address.
    #[inline(always)]
    pub fn read_memory(&mut self, address: u16) -> u8 {
        self.bus.borrow().read_byte(address)
    }

    /// Writes a byte to memory.
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address to write.
    /// * `value` - Value to store.
    #[inline(always)]
    pub fn write_memory(&mut self, address: u16, value: u8) {
        self.bus.borrow_mut().write_byte(address, value);
    }

    pub fn init_post_boot_dmg(&mut self) {
        self.set_register8(Reg8::A, 0x01);
        self.set_register8(Reg8::F, 0xB0);

        self.set_register8(Reg8::B, 0x00);
        self.set_register8(Reg8::C, 0x13);

        self.set_register8(Reg8::D, 0x00);
        self.set_register8(Reg8::E, 0xD8);

        self.set_register8(Reg8::H, 0x01);
        self.set_register8(Reg8::L, 0x4D);

        self.set_register16(Reg16::SP, 0xFFFE);
        self.set_register16(Reg16::PC, 0x0100);

        self.ime = false;
        self.ime_delay = 0;
        self.halted = false;
        self.stopped = false;
        self.joypad_stop_wake_requested = false;
    }
}
