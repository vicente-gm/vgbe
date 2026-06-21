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

use crate::cpu;
use cpu::Cpu;
use cpu::registers::*;

pub static OPCODES_TABLE: [Instruction; 256] = build_opcodes_table();
pub static CB_OPCODES_TABLE: [Instruction; 256] = build_cb_opcodes_table();

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub func: fn(&mut Cpu) -> u8,
    pub cycles: u8,
    pub bytes: u8,
    pub asm: &'static str,
}

#[derive(Debug, Copy, Clone)]
pub struct DisassembledInstruction {
    pub address: u16,
    pub instr: Instruction
}

#[inline(always)]
fn get_register8_from_opcode(opcode: u8, start: u8) -> Reg8 {
    assert!(start <= 7, "The start position of register index must be between 0 and 7.");
    assert!((start + 3) <= 8, "Can't read register out of range.");

    let reg: u8 = (opcode >> start) & ((1 << 3) - 1);

    match reg {
        0 => Reg8::B,
        1 => Reg8::C,
        2 => Reg8::D,
        3 => Reg8::E,
        4 => Reg8::H,
        5 => Reg8::L,
        7 => Reg8::A,
        _ => unreachable!("Invalid register in opcode: {}", opcode) // This case includes the byte pointed by HL
    }
}

#[inline(always)]
fn get_register16_from_opcode(opcode: u8, start: u8) -> Reg16 {
    assert!(start <= 7, "The start position of register index must be between 0 and 7.");
    assert!((start + 2) <= 8, "Can't read register out of range.");

    let reg: u8 = (opcode >> start) & ((1 << 2) - 1);

    match reg {
        0 => Reg16::BC,
        1 => Reg16::DE,
        2 => Reg16::HL,
        3 => Reg16::SP,
        _ => unreachable!("Invalid register in opcode: {}", opcode)
    }
}

/// Returns whether is overflow in a specific bit when a + b
#[inline(always)]
fn overflow_in_bit(a: u16, b: u16, bit: u8) -> bool {
    assert!(bit <= 15, "The bit index must be between 0 and 15.");

    let mask: u32 = (1u32 << (bit.wrapping_add(1))) - 1;

    ((a as u32 & mask) + (b as u32 & mask)) > mask
}

/// Returns whether is borrow from a specific bit when a - b
#[inline(always)]
fn borrow_in_bit(a: u16, b: u16, bit: u8) -> bool {
    assert!(bit <= 15, "The bit index must be between 0 and 15.");

    let mask: u32 = (1u32 << (bit.wrapping_add(1))) - 1;

    ((a as u32) & mask) < ((b as u32) & mask)
}

const fn build_opcodes_table() -> [Instruction; 256] {
    // Initialize the table with default non-valid instructions
    let mut table: [Instruction; 256] = [Instruction {
        func: unknown,
        cycles: 0,
        bytes: 0,
        asm: "UNKNOWN",
    }; 256];

    // ---Block 0
    // NOP
    table[0b0000_0000] = Instruction { func: nop, cycles: 1, bytes: 1, asm: "NOP", };

    // LD R16, IMM16
    table[0b0000_0001] = Instruction { func: ld_r16_imm16, cycles: 3, bytes: 3, asm: "LD BC, IMM16" };
    table[0b0001_0001] = Instruction { func: ld_r16_imm16, cycles: 3, bytes: 3, asm: "LD DE, IMM16" };
    table[0b0010_0001] = Instruction { func: ld_r16_imm16, cycles: 3, bytes: 3, asm: "LD HL, IMM16" };
    table[0b0011_0001] = Instruction { func: ld_r16_imm16, cycles: 3, bytes: 3, asm: "LD SP, IMM16" };

    // LD [R16MEM], A
    table[0b0000_0010] = Instruction { func: ld_r16mem_a, cycles: 2, bytes: 1, asm: "LD [BC], A", };
    table[0b0001_0010] = Instruction { func: ld_r16mem_a, cycles: 2, bytes: 1, asm: "LD [DE], A", };
    table[0b0010_0010] = Instruction { func: ld_r16mem_a, cycles: 2, bytes: 1, asm: "LD [HL+], A", };
    table[0b0011_0010] = Instruction { func: ld_r16mem_a, cycles: 2, bytes: 1, asm: "LD [HL-], A" };

    // LD A, [R16MEM]
    table[0b0000_1010] = Instruction { func: ld_a_r16mem, cycles: 2, bytes: 1, asm: "LD A, [BC]" };
    table[0b0001_1010] = Instruction { func: ld_a_r16mem, cycles: 2, bytes: 1, asm: "LD A, [DE]" };
    table[0b0010_1010] = Instruction { func: ld_a_r16mem, cycles: 2, bytes: 1, asm: "LD A, [HL+]" };
    table[0b0011_1010] = Instruction { func: ld_a_r16mem, cycles: 2, bytes: 1, asm: "LD A, [HL-]" };

    // LD [IMM16], SP
    table[0b0000_1000] = Instruction { func: ld_imm16_sp, cycles: 5, bytes: 3, asm: "LD IMM16, SP" };

    // INC R16
    table[0b0000_0011] = Instruction { func: inc_r16, cycles: 2, bytes: 1, asm: "INC BC" };
    table[0b0001_0011] = Instruction { func: inc_r16, cycles: 2, bytes: 1, asm: "INC DE" };
    table[0b0010_0011] = Instruction { func: inc_r16, cycles: 2, bytes: 1, asm: "INC HL" };
    table[0b0011_0011] = Instruction { func: inc_r16, cycles: 2, bytes: 1, asm: "INC SP" };

    // DEC R16
    table[0b0000_1011] = Instruction { func: dec_r16, cycles: 2, bytes: 1, asm: "DEC BC" };
    table[0b0001_1011] = Instruction { func: dec_r16, cycles: 2, bytes: 1, asm: "DEC DE" };
    table[0b0010_1011] = Instruction { func: dec_r16, cycles: 2, bytes: 1, asm: "DEC HL" };
    table[0b0011_1011] = Instruction { func: dec_r16, cycles: 2, bytes: 1, asm: "DEC SP" };

    // ADD HL, R16
    table[0b0000_1001] = Instruction { func: add_hl_r16, cycles: 2, bytes: 1, asm: "ADD HL, BC" };
    table[0b0001_1001] = Instruction { func: add_hl_r16, cycles: 2, bytes: 1, asm: "ADD HL, DE" };
    table[0b0010_1001] = Instruction { func: add_hl_r16, cycles: 2, bytes: 1, asm: "ADD HL, HL" };
    table[0b0011_1001] = Instruction { func: add_hl_r16, cycles: 2, bytes: 1, asm: "ADD HL, SP" };

    // INC R8
    table[0b0000_0100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC B" };
    table[0b0000_1100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC C" };
    table[0b0001_0100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC D" };
    table[0b0001_1100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC E" };
    table[0b0010_0100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC H" };
    table[0b0010_1100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC L" };
    table[0b0011_1100] = Instruction { func: inc_r8, cycles: 1, bytes: 1, asm: "INC A" };

    // INC [HL]
    table[0b0011_0100] = Instruction { func: inc_hlmem, cycles: 3, bytes: 1, asm: "INC [HL]" };

    // DEC R8
    table[0b0000_0101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC B" };
    table[0b0000_1101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC C" };
    table[0b0001_0101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC D" };
    table[0b0001_1101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC E" };
    table[0b0010_0101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC H" };
    table[0b0010_1101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC L" };
    table[0b0011_1101] = Instruction { func: dec_r8, cycles: 1, bytes: 1, asm: "DEC A" };

    // DEC [HL]
    table[0b0011_0101] = Instruction { func: dec_hlmem, cycles: 3, bytes: 1, asm: "DEC [HL]" };

    // LD R8, IMM8
    table[0b0000_0110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD B, IMM8" };
    table[0b0000_1110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD C, IMM8" };
    table[0b0001_0110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD D, IMM8" };
    table[0b0001_1110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD E, IMM8" };
    table[0b0010_0110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD H, IMM8" };
    table[0b0010_1110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD L, IMM8" };
    table[0b0011_1110] = Instruction { func: ld_r8_imm8, cycles: 2, bytes: 2, asm: "LD A, IMM8" };

    // LD [HL], IMM8
    table[0b0011_0110] = Instruction { func: ld_hlmem_imm8, cycles: 3, bytes: 2, asm: "LD [HL], IMM8" };

    // RLCA
    table[0b0000_0111] = Instruction { func: rlca, cycles: 1, bytes: 1, asm: "RLCA" };

    // RRCA
    table[0b0000_1111] = Instruction { func: rrca, cycles: 1, bytes: 1, asm: "RRCA" };

    // RLA
    table[0b0001_0111] = Instruction { func: rla, cycles: 1, bytes: 1, asm: "RLA" };

    // RRA
    table[0b0001_1111] = Instruction { func: rra, cycles: 1, bytes: 1, asm: "RRA" };

    // DAA
    table[0b0010_0111] = Instruction { func: daa, cycles: 1, bytes: 1, asm: "DAA" };

    // CPL
    table[0b0010_1111] = Instruction { func: cpl, cycles: 1, bytes: 1, asm: "CPL" };

    // SCF
    table[0b0011_0111] = Instruction { func: scf, cycles: 1, bytes: 1, asm: "SCF" };

    // CCF
    table[0b0011_1111] = Instruction { func: ccf, cycles: 1, bytes: 1, asm: "CCF" };

    // JR IMM8
    table[0b0001_1000] = Instruction { func: jr_imm8, cycles: 3, bytes: 2, asm: "JR IMM8" };

    // JR COND, IMM8
    table[0b0010_0000] = Instruction { func: jr_cond_imm8, cycles: 2, bytes: 2, asm: "JR NZ, IMM8" };
    table[0b0010_1000] = Instruction { func: jr_cond_imm8, cycles: 2, bytes: 2, asm: "JR Z, IMM8" };
    table[0b0011_0000] = Instruction { func: jr_cond_imm8, cycles: 2, bytes: 2, asm: "JR NC, IMM8" };
    table[0b0011_1000] = Instruction { func: jr_cond_imm8, cycles: 2, bytes: 2, asm: "JR C, IMM8" };

    // STOP
    table[0b0001_0000] = Instruction { func: stop, cycles: 0, bytes: 2, asm: "STOP" };

    // ---Block 1: 8-bit register-to-register loads
    // LD R8, R8 -> Exception: ld [hl], [hl] instead yields halt instruction
    table[0b0100_0000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, B" };
    table[0b0100_0001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, C" };
    table[0b0100_0010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, D" };
    table[0b0100_0011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, E" };
    table[0b0100_0100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, H" };
    table[0b0100_0101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, L" };
    table[0b0100_0111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD B, A" };
    table[0b0100_1000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, B" };
    table[0b0100_1001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, C" };
    table[0b0100_1010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, D" };
    table[0b0100_1011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, E" };
    table[0b0100_1100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, H" };
    table[0b0100_1101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, L" };
    table[0b0100_1111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD C, A" };
    table[0b0101_0000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, B" };
    table[0b0101_0001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, C" };
    table[0b0101_0010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, D" };
    table[0b0101_0011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, E" };
    table[0b0101_0100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, H" };
    table[0b0101_0101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, L" };
    table[0b0101_0111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD D, A" };
    table[0b0101_1000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, B" };
    table[0b0101_1001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, C" };
    table[0b0101_1010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, D" };
    table[0b0101_1011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, E" };
    table[0b0101_1100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, H" };
    table[0b0101_1101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, L" };
    table[0b0101_1111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD E, A" };
    table[0b0110_0000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, B" };
    table[0b0110_0001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, C" };
    table[0b0110_0010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, D" };
    table[0b0110_0011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, E" };
    table[0b0110_0100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, H" };
    table[0b0110_0101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, L" };
    table[0b0110_0111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD H, A" };
    table[0b0110_1000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, B" };
    table[0b0110_1001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, C" };
    table[0b0110_1010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, D" };
    table[0b0110_1011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, E" };
    table[0b0110_1100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, H" };
    table[0b0110_1101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, L" };
    table[0b0110_1111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD L, A" };
    table[0b0111_1000] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, B" };
    table[0b0111_1001] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, C" };
    table[0b0111_1010] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, D" };
    table[0b0111_1011] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, E" };
    table[0b0111_1100] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, H" };
    table[0b0111_1101] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, L" };
    table[0b0111_1110] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, [HL]" };
    table[0b0111_1111] = Instruction { func: ld_r8_r8, cycles: 1, bytes: 1, asm: "LD A, A" };

    // LD R8, [HL]
    table[0b0100_0110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD B, [HL]" };
    table[0b0100_1110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD C, [HL]" };
    table[0b0101_0110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD D, [HL]" };
    table[0b0101_1110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD E, [HL]" };
    table[0b0110_0110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD H, [HL]" };
    table[0b0110_1110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD L, [HL]" };
    table[0b0111_1110] = Instruction { func: ld_r8_hlmem, cycles: 2, bytes: 1, asm: "LD A, [HL]" };

    // LD [HL], R8
    table[0b0111_0000] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], B" };
    table[0b0111_0001] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], C" };
    table[0b0111_0010] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], D" };
    table[0b0111_0011] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], E" };
    table[0b0111_0100] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], H" };
    table[0b0111_0101] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], L" };
    table[0b0111_0111] = Instruction { func: ld_hlmem_r8, cycles: 2, bytes: 1, asm: "LD [HL], A" };

    // LD [HL], [HL]: instead yields the halt instruction
    // HALT
    table[0b0111_0110] = Instruction { func: halt, cycles: 0, bytes: 1, asm: "HALT" };

    // ---Block 2: 8-bit arithmetic
    // ADD A, R8
    table[0b1000_0000] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, B" };
    table[0b1000_0001] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, C" };
    table[0b1000_0010] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, D" };
    table[0b1000_0011] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, E" };
    table[0b1000_0100] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, H" };
    table[0b1000_0101] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, L" };
    table[0b1000_0111] = Instruction { func: add_a_r8, cycles: 1, bytes: 1, asm: "ADD A, A" };

    // ADD A, [HL]
    table[0b1000_0110] = Instruction { func: add_a_hlmem, cycles: 2, bytes: 1, asm: "ADD A, [HL]" };

    // ADC A, R8
    table[0b1000_1000] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, B" };
    table[0b1000_1001] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, C" };
    table[0b1000_1010] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, D" };
    table[0b1000_1011] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, E" };
    table[0b1000_1100] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, H" };
    table[0b1000_1101] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, L" };
    table[0b1000_1111] = Instruction { func: adc_a_r8, cycles: 1, bytes: 1, asm: "ADC A, A" };

    // ADC A, [HL]
    table[0b1000_1110] = Instruction { func: adc_a_hlmem, cycles: 2, bytes: 1, asm: "ADC A, [HL]" };

    // SUB A, R8
    table[0b1001_0000] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, B" };
    table[0b1001_0001] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, C" };
    table[0b1001_0010] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, D" };
    table[0b1001_0011] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, E" };
    table[0b1001_0100] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, H" };
    table[0b1001_0101] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, L" };
    table[0b1001_0111] = Instruction { func: sub_a_r8, cycles: 1, bytes: 1, asm: "SUB A, A" };

    // SUB A, [HL]
    table[0b1001_0110] = Instruction { func: sub_a_hlmem, cycles: 2, bytes: 1, asm: "SUB A, [HL]" };

    // SBC A, R8
    table[0b1001_1000] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, B" };
    table[0b1001_1001] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, C" };
    table[0b1001_1010] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, D" };
    table[0b1001_1011] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, E" };
    table[0b1001_1100] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, H" };
    table[0b1001_1101] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, L" };
    table[0b1001_1111] = Instruction { func: sbc_a_r8, cycles: 1, bytes: 1, asm: "SBC A, A" };

    // SBC A, [HL]
    table[0b1001_1110] = Instruction { func: sbc_a_hlmem, cycles: 2, bytes: 1, asm: "SBC A, [HL]" };

    // AND A, R8
    table[0b1010_0000] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, B" };
    table[0b1010_0001] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, C" };
    table[0b1010_0010] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, D" };
    table[0b1010_0011] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, E" };
    table[0b1010_0100] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, H" };
    table[0b1010_0101] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, L" };
    table[0b1010_0111] = Instruction { func: and_a_r8, cycles: 1, bytes: 1, asm: "AND A, A" };

    // AND A, [HL]
    table[0b1010_0110] = Instruction { func: and_a_hlmem, cycles: 2, bytes: 1, asm: "AND A, [HL]" };

    // XOR A, R8
    table[0b1010_1000] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, B" };
    table[0b1010_1001] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, C" };
    table[0b1010_1010] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, D" };
    table[0b1010_1011] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, E" };
    table[0b1010_1100] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, H" };
    table[0b1010_1101] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, L" };
    table[0b1010_1111] = Instruction { func: xor_a_r8, cycles: 1, bytes: 1, asm: "XOR A, A" };

    // XOR A, [HL]
    table[0b1010_1110] = Instruction { func: xor_a_hlmem, cycles: 2, bytes: 1, asm: "XOR A, [HL]" };

    // OR A, R8
    table[0b1011_0000] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, B" };
    table[0b1011_0001] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, C" };
    table[0b1011_0010] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, D" };
    table[0b1011_0011] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, E" };
    table[0b1011_0100] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, H" };
    table[0b1011_0101] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, L" };
    table[0b1011_0111] = Instruction { func: or_a_r8, cycles: 1, bytes: 1, asm: "OR A, A" };

    // OR A, [HL]
    table[0b1011_0110] = Instruction { func: or_a_hlmem, cycles: 2, bytes: 1, asm: "OR A, [HL]" };

    // CP A, R8
    table[0b1011_1000] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, B" };
    table[0b1011_1001] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, C" };
    table[0b1011_1010] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, D" };
    table[0b1011_1011] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, E" };
    table[0b1011_1100] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, H" };
    table[0b1011_1101] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, L" };
    table[0b1011_1111] = Instruction { func: cp_a_r8, cycles: 1, bytes: 1, asm: "CP A, A" };

    // CP A, [HL]
    table[0b1011_1110] = Instruction { func: cp_a_hlmem, cycles: 2, bytes: 1, asm: "CP A, [HL]" };

    // ---Block 3
    // ADD A, IMM8
    table[0b1100_0110] = Instruction { func: add_a_imm8, cycles: 2, bytes: 2, asm: "ADD A, IMM8" };

    // ADC A, IMM8
    table[0b1100_1110] = Instruction { func: adc_a_imm8, cycles: 2, bytes: 2, asm: "ADC A, IMM8" };

    // SUB A, IMM8
    table[0b1101_0110] = Instruction { func: sub_a_imm8, cycles: 2, bytes: 2, asm: "SUB A, IMM8" };

    // SBC A, IMM8
    table[0b1101_1110] = Instruction { func: sbc_a_imm8, cycles: 2, bytes: 2, asm: "SBC A, IMM8" };

    // AND A, IMM8
    table[0b1110_0110] = Instruction { func: and_a_imm8, cycles: 2, bytes: 2, asm: "AND A, IMM8" };

    // XOR A, IMM8
    table[0b1110_1110] = Instruction { func: xor_a_imm8, cycles: 2, bytes: 2, asm: "XOR A, IMM8" };

    // OR A, IMM8
    table[0b1111_0110] = Instruction { func: or_a_imm8, cycles: 2, bytes: 2, asm: "OR A, IMM8" };

    // CP A, IMM8
    table[0b1111_1110] = Instruction { func: cp_a_imm8, cycles: 2, bytes: 2, asm: "CP A, IMM8" };

    // RET COND
    table[0b1100_0000] = Instruction { func: ret_cond, cycles: 2, bytes: 1, asm: "RET NZ" };
    table[0b1100_1000] = Instruction { func: ret_cond, cycles: 2, bytes: 1, asm: "RET Z" };
    table[0b1101_0000] = Instruction { func: ret_cond, cycles: 2, bytes: 1, asm: "RET NC" };
    table[0b1101_1000] = Instruction { func: ret_cond, cycles: 2, bytes: 1, asm: "RET C" };

    // RET
    table[0b1100_1001] = Instruction { func: ret, cycles: 4, bytes: 1, asm: "RET" };

    // RETI
    table[0b1101_1001] = Instruction { func: reti, cycles: 4, bytes: 1, asm: "RETI" };

    // JP COND, IMM16
    table[0b1100_0010] = Instruction { func: jp_cond_imm16, cycles: 3, bytes: 3, asm: "JP NZ, IMM16" };
    table[0b1100_1010] = Instruction { func: jp_cond_imm16, cycles: 3, bytes: 3, asm: "JP Z, IMM16" };
    table[0b1101_0010] = Instruction { func: jp_cond_imm16, cycles: 3, bytes: 3, asm: "JP NC, IMM16" };
    table[0b1101_1010] = Instruction { func: jp_cond_imm16, cycles: 3, bytes: 3, asm: "JP C, IMM16" };

    // JP IMM16
    table[0b1100_0011] = Instruction { func: jp_imm16, cycles: 3, bytes: 2, asm: "JP IMM16" };

    // JP HL
    table[0b1110_1001] = Instruction { func: jp_hl, cycles: 1, bytes: 1, asm: "JP HL" };

    // CALL COND, IMM16
    table[0b1100_0100] = Instruction { func: call_cond_imm16, cycles: 3, bytes: 3, asm: "CALL NZ, IMM16" };
    table[0b1100_1100] = Instruction { func: call_cond_imm16, cycles: 3, bytes: 3, asm: "CALL Z, IMM16" };
    table[0b1101_0100] = Instruction { func: call_cond_imm16, cycles: 3, bytes: 3, asm: "CALL NC, IMM16" };
    table[0b1101_1100] = Instruction { func: call_cond_imm16, cycles: 3, bytes: 3, asm: "CALL C, IMM16" };

    // CALL IMM16
    table[0b1100_1101] = Instruction { func: call_imm16, cycles: 6, bytes: 3, asm: "CALL IMM16" };

    // RST TGT3
    table[0b1100_0111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x00" };
    table[0b1100_1111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x08" };
    table[0b1101_0111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x10" };
    table[0b1101_1111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x18" };
    table[0b1110_0111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x20" };
    table[0b1110_1111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x28" };
    table[0b1111_0111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x30" };
    table[0b1111_1111] = Instruction { func: rst_tgt3, cycles: 4, bytes: 1, asm: "RST 0x38" };

    // POP R16STK
    table[0b1100_0001] = Instruction { func: pop_r16stk, cycles: 3, bytes: 1, asm: "POP BC" };
    table[0b1101_0001] = Instruction { func: pop_r16stk, cycles: 3, bytes: 1, asm: "POP DE" };
    table[0b1110_0001] = Instruction { func: pop_r16stk, cycles: 3, bytes: 1, asm: "POP HL" };

    // POP AF
    table[0b1111_0001] = Instruction { func: pop_af, cycles: 3, bytes: 1, asm: "POP AF" };

    // PUSH R16STK
    table[0b1100_0101] = Instruction { func: push_r16stk, cycles: 4, bytes: 1, asm: "PUSH BC" };
    table[0b1101_0101] = Instruction { func: push_r16stk, cycles: 4, bytes: 1, asm: "PUSH DE" };
    table[0b1110_0101] = Instruction { func: push_r16stk, cycles: 4, bytes: 1, asm: "PUSH HL" };

    // PUSH AF
    table[0b1111_0101] = Instruction { func: push_af, cycles: 4, bytes: 1, asm: "PUSH AF" };

    // LDH [C], A
    table[0b1110_0010] = Instruction { func: ldh_c_a, cycles: 2, bytes: 1, asm: "LDH [C], A" };

    // LDH [IMM16], A
    table[0b1110_0000] = Instruction { func: ldh_imm16_a, cycles: 3, bytes: 3, asm: "LDH IMM8, A" };

    // LD [IMM16], A
    table[0b1110_1010] = Instruction { func: ld_imm16_a, cycles: 4, bytes: 3, asm: "LD IMM16, A" };

    // LDH A, [C]
    table[0b1111_0010] = Instruction { func: ldh_a_c, cycles: 2, bytes: 1, asm: "LDH A, [C]" };

    // LDH A, [IMM16]
    table[0b1111_0000] = Instruction { func: ldh_a_imm16, cycles: 3, bytes: 2, asm: "LDH A, IMM8" };

    // LD A, [IMM16]
    table[0b1111_1010] = Instruction { func: ld_a_imm16, cycles: 4, bytes: 3, asm: "LD A, IMM16" };

    // ADD SP, IMM8
    table[0b1110_1000] = Instruction { func: add_sp_imm8, cycles: 4, bytes: 2, asm: "ADD SP, IMM8" };

    // LD HL, SP+IMM8
    table[0b1111_1000] = Instruction { func: ld_hli_imm8, cycles: 3, bytes: 2, asm: "LD HL, SP+IMM8" };

    // LD SP, HL
    table[0b1111_1001] = Instruction { func: ld_sp_hl, cycles: 2, bytes: 1, asm: "LD SP, HL" };

    // Interruptions related
    // DI
    table[0b1111_0011] = Instruction { func: di, cycles: 1, bytes: 1, asm: "DI" };

    // EI
    table[0b1111_1011] = Instruction { func: ei, cycles: 1, bytes: 1, asm: "EI" };

    table
}

const fn build_cb_opcodes_table() -> [Instruction; 256] {
    // Initialize the table with default non-valid instructions
    let mut table: [Instruction; 256] = [Instruction {
        func: unknown,
        cycles: 0,
        bytes: 0,
        asm: "UNKNOWN"
    }; 256];

    // ---$CB prefix instructions
    // RLC R8
    table[0b0000_0000] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC B" };
    table[0b0000_0001] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC C" };
    table[0b0000_0010] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC D" };
    table[0b0000_0011] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC E" };
    table[0b0000_0100] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC H" };
    table[0b0000_0101] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC L" };
    table[0b0000_0111] = Instruction { func: rlc_r8, cycles: 2, bytes: 2, asm: "RLC A" };

    // RLC [HL]
    table[0b0000_0110] = Instruction { func: rlc_hlmem, cycles: 4, bytes: 2, asm: "RLC [HL]" };

    // RRC R8
    table[0b0000_1000] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC B" };
    table[0b0000_1001] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC C" };
    table[0b0000_1010] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC D" };
    table[0b0000_1011] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC E" };
    table[0b0000_1100] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC H" };
    table[0b0000_1101] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC L" };
    table[0b0000_1111] = Instruction { func: rrc_r8, cycles: 2, bytes: 2, asm: "RRC A" };

    // RRC [HL]
    table[0b0000_1110] = Instruction { func: rrc_hlmem, cycles: 4, bytes: 2, asm: "RRC [HL]" };

    // RL R8
    table[0b0001_0000] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL B" };
    table[0b0001_0001] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL C" };
    table[0b0001_0010] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL D" };
    table[0b0001_0011] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL E" };
    table[0b0001_0100] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL H" };
    table[0b0001_0101] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL L" };
    table[0b0001_0111] = Instruction { func: rl_r8, cycles: 2, bytes: 2, asm: "RL A" };

    // RL [HL]
    table[0b0001_0110] = Instruction { func: rl_hlmem, cycles: 4, bytes: 2, asm: "RL [HL]" };

    // RR R8
    table[0b0001_1000] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR B" };
    table[0b0001_1001] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR C" };
    table[0b0001_1010] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR D" };
    table[0b0001_1011] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR E" };
    table[0b0001_1100] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR H" };
    table[0b0001_1101] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR L" };
    table[0b0001_1111] = Instruction { func: rr_r8, cycles: 2, bytes: 2, asm: "RR A" };

    // RR [HL]
    table[0b0001_1110] = Instruction { func: rr_hlmem, cycles: 4, bytes: 2, asm: "RR [HL]" };

    // SLA R8
    table[0b0010_0000] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA B" };
    table[0b0010_0001] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA C" };
    table[0b0010_0010] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA D" };
    table[0b0010_0011] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA E" };
    table[0b0010_0100] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA H" };
    table[0b0010_0101] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA L" };
    table[0b0010_0111] = Instruction { func: sla_r8, cycles: 2, bytes: 2, asm: "SLA A" };

    // SLA [HL]
    table[0b0010_0110] = Instruction { func: sla_hlmem, cycles: 4, bytes: 2, asm: "SLA [HL]" };

    // SRA R8
    table[0b0010_1000] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA B" };
    table[0b0010_1001] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA C" };
    table[0b0010_1010] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA D" };
    table[0b0010_1011] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA E" };
    table[0b0010_1100] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA H" };
    table[0b0010_1101] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA L" };
    table[0b0010_1111] = Instruction { func: sra_r8, cycles: 2, bytes: 2, asm: "SRA A" };

    // SRA [HL]
    table[0b0010_1110] = Instruction { func: sra_hlmem, cycles: 4, bytes: 2, asm: "SRA [HL]" };

    // SWAP R8
    table[0b0011_0000] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP B" };
    table[0b0011_0001] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP C" };
    table[0b0011_0010] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP D" };
    table[0b0011_0011] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP E" };
    table[0b0011_0100] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP H" };
    table[0b0011_0101] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP L" };
    table[0b0011_0111] = Instruction { func: swap_r8, cycles: 2, bytes: 2, asm: "SWAP A" };

    // SWAP [HL]
    table[0b0011_0110] = Instruction { func: swap_hlmem, cycles: 4, bytes: 2, asm: "SWAP [HL]" };

    // SRL R8
    table[0b0011_1000] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL B" };
    table[0b0011_1001] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL C" };
    table[0b0011_1010] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL D" };
    table[0b0011_1011] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL E" };
    table[0b0011_1100] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL H" };
    table[0b0011_1101] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL L" };
    table[0b0011_1111] = Instruction { func: srl_r8, cycles: 2, bytes: 2, asm: "SRL A" };

    // SRL [HL]
    table[0b0011_1110] = Instruction { func: srl_hlmem, cycles: 4, bytes: 2, asm: "SRL [HL]" };

    // BIT B3, R8
    table[0b0100_0000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, B" };
    table[0b0100_0001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, C" };
    table[0b0100_0010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, D" };
    table[0b0100_0011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, E" };
    table[0b0100_0100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, H" };
    table[0b0100_0101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, L" };
    table[0b0100_0111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 0, A" };

    table[0b0100_1000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, B" };
    table[0b0100_1001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, C" };
    table[0b0100_1010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, D" };
    table[0b0100_1011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, E" };
    table[0b0100_1100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, H" };
    table[0b0100_1101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, L" };
    table[0b0100_1111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 1, A" };

    table[0b0101_0000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, B" };
    table[0b0101_0001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, C" };
    table[0b0101_0010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, D" };
    table[0b0101_0011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, E" };
    table[0b0101_0100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, H" };
    table[0b0101_0101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, L" };
    table[0b0101_0111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 2, A" };

    table[0b0101_1000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, B" };
    table[0b0101_1001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, C" };
    table[0b0101_1010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, D" };
    table[0b0101_1011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, E" };
    table[0b0101_1100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, H" };
    table[0b0101_1101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, L" };
    table[0b0101_1111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 3, A" };

    table[0b0110_0000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, B" };
    table[0b0110_0001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, C" };
    table[0b0110_0010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, D" };
    table[0b0110_0011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, E" };
    table[0b0110_0100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, H" };
    table[0b0110_0101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, L" };
    table[0b0110_0111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 4, A" };

    table[0b0110_1000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, B" };
    table[0b0110_1001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, C" };
    table[0b0110_1010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, D" };
    table[0b0110_1011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, E" };
    table[0b0110_1100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, H" };
    table[0b0110_1101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, L" };
    table[0b0110_1111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 5, A" };

    table[0b0111_0000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, B" };
    table[0b0111_0001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, C" };
    table[0b0111_0010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, D" };
    table[0b0111_0011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, E" };
    table[0b0111_0100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, H" };
    table[0b0111_0101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, L" };
    table[0b0111_0111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 6, A" };

    table[0b0111_1000] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, B" };
    table[0b0111_1001] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, C" };
    table[0b0111_1010] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, D" };
    table[0b0111_1011] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, E" };
    table[0b0111_1100] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, H" };
    table[0b0111_1101] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, L" };
    table[0b0111_1111] = Instruction { func: bit_b3_r8, cycles: 2, bytes: 2, asm: "BIT 7, A" };

    // BIT B3, [HL]
    table[0b0100_0110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 0, [HL]" };
    table[0b0100_1110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 1, [HL]" };
    table[0b0101_0110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 2, [HL]" };
    table[0b0101_1110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 3, [HL]" };
    table[0b0110_0110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 4, [HL]" };
    table[0b0110_1110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 5, [HL]" };
    table[0b0111_0110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 6, [HL]" };
    table[0b0111_1110] = Instruction { func: bit_b3_hlmem, cycles: 3, bytes: 2, asm: "BIT 7, [HL]" };

    // RES B3, R8
    table[0b1000_0000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, B" };
    table[0b1000_0001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, C" };
    table[0b1000_0010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, D" };
    table[0b1000_0011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, E" };
    table[0b1000_0100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, H" };
    table[0b1000_0101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, L" };
    table[0b1000_0111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 0, A" };

    table[0b1000_1000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, B" };
    table[0b1000_1001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, C" };
    table[0b1000_1010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, D" };
    table[0b1000_1011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, E" };
    table[0b1000_1100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, H" };
    table[0b1000_1101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, L" };
    table[0b1000_1111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 1, A" };

    table[0b1001_0000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, B" };
    table[0b1001_0001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, C" };
    table[0b1001_0010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, D" };
    table[0b1001_0011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, E" };
    table[0b1001_0100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, H" };
    table[0b1001_0101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, L" };
    table[0b1001_0111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 2, A" };

    table[0b1001_1000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, B" };
    table[0b1001_1001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, C" };
    table[0b1001_1010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, D" };
    table[0b1001_1011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, E" };
    table[0b1001_1100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, H" };
    table[0b1001_1101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, L" };
    table[0b1001_1111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 3, A" };

    table[0b1010_0000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, B" };
    table[0b1010_0001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, C" };
    table[0b1010_0010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, D" };
    table[0b1010_0011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, E" };
    table[0b1010_0100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, H" };
    table[0b1010_0101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, L" };
    table[0b1010_0111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 4, A" };

    table[0b1010_1000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, B" };
    table[0b1010_1001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, C" };
    table[0b1010_1010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, D" };
    table[0b1010_1011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, E" };
    table[0b1010_1100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, H" };
    table[0b1010_1101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, L" };
    table[0b1010_1111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 5, A" };

    table[0b1011_0000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, B" };
    table[0b1011_0001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, C" };
    table[0b1011_0010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, D" };
    table[0b1011_0011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, E" };
    table[0b1011_0100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, H" };
    table[0b1011_0101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, L" };
    table[0b1011_0111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 6, A" };

    table[0b1011_1000] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, B" };
    table[0b1011_1001] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, C" };
    table[0b1011_1010] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, D" };
    table[0b1011_1011] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, E" };
    table[0b1011_1100] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, H" };
    table[0b1011_1101] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, L" };
    table[0b1011_1111] = Instruction { func: res_b3_r8, cycles: 2, bytes: 2, asm: "RES 7, A" };

    // RES B3, [HL]
    table[0b1000_0110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 0, [HL]" };
    table[0b1000_1110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 1, [HL]" };
    table[0b1001_0110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 2, [HL]" };
    table[0b1001_1110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 3, [HL]" };
    table[0b1010_0110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 4, [HL]" };
    table[0b1010_1110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 5, [HL]" };
    table[0b1011_0110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 6, [HL]" };
    table[0b1011_1110] = Instruction { func: res_b3_hlmem, cycles: 4, bytes: 2, asm: "RES 7, [HL]" };

    // SET B3, R8
    table[0b1100_0000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, B" };
    table[0b1100_0001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, C" };
    table[0b1100_0010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, D" };
    table[0b1100_0011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, E" };
    table[0b1100_0100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, H" };
    table[0b1100_0101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, L" };
    table[0b1100_0111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 0, A" };

    table[0b1100_1000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, B" };
    table[0b1100_1001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, C" };
    table[0b1100_1010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, D" };
    table[0b1100_1011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, E" };
    table[0b1100_1100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, H" };
    table[0b1100_1101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, L" };
    table[0b1100_1111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 1, A" };

    table[0b1101_0000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, B" };
    table[0b1101_0001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, C" };
    table[0b1101_0010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, D" };
    table[0b1101_0100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, H" };
    table[0b1101_0011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, E" };
    table[0b1101_0101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, L" };
    table[0b1101_0111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 2, A" };

    table[0b1101_1000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, B" };
    table[0b1101_1001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, C" };
    table[0b1101_1010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, D" };
    table[0b1101_1011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, E" };
    table[0b1101_1100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, H" };
    table[0b1101_1101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, L" };
    table[0b1101_1111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 3, A" };

    table[0b1110_0000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, B" };
    table[0b1110_0001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, C" };
    table[0b1110_0010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, D" };
    table[0b1110_0011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, E" };
    table[0b1110_0100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, H" };
    table[0b1110_0101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, L" };
    table[0b1110_0111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 4, A" };

    table[0b1110_1000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, B" };
    table[0b1110_1001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, C" };
    table[0b1110_1010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, D" };
    table[0b1110_1011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, E" };
    table[0b1110_1100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, H" };
    table[0b1110_1101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, L" };
    table[0b1110_1111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 5, A" };

    table[0b1111_0000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, B" };
    table[0b1111_0001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, C" };
    table[0b1111_0010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, D" };
    table[0b1111_0011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, E" };
    table[0b1111_0100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, H" };
    table[0b1111_0101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, L" };
    table[0b1111_0111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 6, A" };

    table[0b1111_1000] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, B" };
    table[0b1111_1001] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, C" };
    table[0b1111_1010] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, D" };
    table[0b1111_1011] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, E" };
    table[0b1111_1100] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, H" };
    table[0b1111_1101] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, L" };
    table[0b1111_1111] = Instruction { func: set_b3_r8, cycles: 2, bytes: 2, asm: "SET 7, A" };

    // SET B3, [HL]
    table[0b1100_0110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 0, [HL]" };
    table[0b1100_1110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 1, [HL]" };
    table[0b1101_0110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 2, [HL]" };
    table[0b1101_1110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 3, [HL]" };
    table[0b1110_0110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 4, [HL]" };
    table[0b1110_1110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 5, [HL]" };
    table[0b1111_0110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 6, [HL]" };
    table[0b1111_1110] = Instruction { func: set_b3_hlmem, cycles: 4, bytes: 2, asm: "SET 7, [HL]" };

    table
}

/// UNKNOWN
///
/// Handles an invalid or unimplemented opcode.
pub fn unknown(_: &mut Cpu) -> u8 {
    0
}

/// NOP
///
/// No operation.
pub fn nop(_: &mut Cpu) -> u8 {
    // println!("Executing nop");
    0
}

// Block 0
/// LD R16, IMM16
///
/// Copies the 16-bit immediate value into register `R16`.
pub fn ld_r16_imm16(cpu: &mut Cpu) -> u8 {
    let byte1: u8 = cpu.read_pc();
    let byte2: u8 = cpu.read_pc();
    cpu.set_register16(get_register16_from_opcode(cpu.opcode, 4), u16::from_le_bytes([byte1, byte2]));
    0
}

/// LD [R16MEM], A
///
/// Copies register `A` into the byte pointed to by `BC`, `DE`, `[HL+]`, or `[HL-]`.
pub fn ld_r16mem_a(cpu: &mut Cpu) -> u8 {
    let addr: u16 = match cpu.opcode {
        0b0010_0010 => {   // LD [HL+], A
            let hl = cpu.get_register16(Reg16::HL);
            cpu.set_register16(Reg16::HL, hl.wrapping_add(1));
            hl
        }
        0b0011_0010 => {   // LD [HL-], A
            let hl = cpu.get_register16(Reg16::HL);
            cpu.set_register16(Reg16::HL, hl.wrapping_sub(1));
            hl
        }
        opcode => cpu.get_register16(get_register16_from_opcode(opcode, 4)),
    };

    cpu.write_memory(addr, cpu.get_register8(Reg8::A));
    0
}

/// LD A, [R16MEM]
///
/// Copies the byte pointed to by `BC`, `DE`, `[HL+]`, or `[HL-]` into register `A`.
pub fn ld_a_r16mem(cpu: &mut Cpu) -> u8 {
    let addr: u16 = match cpu.opcode {
        0b0010_1010 => {   // Case HL+
            let hl = cpu.get_register16(Reg16::HL);
            cpu.set_register16(Reg16::HL, hl.wrapping_add(1));
            hl
        },
        0b0011_1010 => {   // Case HL-
            let hl = cpu.get_register16(Reg16::HL);
            cpu.set_register16(Reg16::HL, hl.wrapping_sub(1));
            hl
        },
        opcode => cpu.get_register16(get_register16_from_opcode(opcode, 4)),
    };

    let value: u8 = cpu.read_memory(addr);
    cpu.set_register8(Reg8::A, value);
    0
}

/// LD [IMM16], SP
///
/// Copies register `SP` into the two bytes starting at address `IMM16`, low byte first.
pub fn ld_imm16_sp(cpu: &mut Cpu) -> u8 {
    let byte1: u8 = cpu.read_pc();
    let byte2: u8 = cpu.read_pc();
    let addr: u16 = u16::from_le_bytes([byte1, byte2]);

    // Copy SP & OxFF at address imm16
    cpu.write_memory(addr, (cpu.get_register16(Reg16::SP) & 0xFF) as u8);

    // Copy SP >> 8 at address (imm16 + 1)
    cpu.write_memory(addr.wrapping_add(1), (cpu.get_register16(Reg16::SP) >> 8) as u8);
    0
}

/// INC R16
///
/// Increments register `R16` by 1.
pub fn inc_r16(cpu: &mut Cpu) -> u8 {
    let reg: Reg16 = get_register16_from_opcode(cpu.opcode, 4);
    cpu.set_register16(reg, cpu.get_register16(reg).wrapping_add(1));
    0
}

/// DEC R16
///
/// Decrements register `R16` by 1.
pub fn dec_r16(cpu: &mut Cpu) -> u8 {
    let reg: Reg16 = get_register16_from_opcode(cpu.opcode, 4);
    cpu.set_register16(reg, cpu.get_register16(reg).wrapping_sub(1));
    0
}

/// ADD HL, R16
///
/// Adds register `R16` to `HL`.
pub fn add_hl_r16(cpu: &mut Cpu) -> u8 {
    let reg: Reg16 = get_register16_from_opcode(cpu.opcode, 4);
    let value: u16 = cpu.get_register16(reg);
    let hl: u16 = cpu.get_register16(Reg16::HL);
    cpu.set_register16(Reg16::HL, hl.wrapping_add(value));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let c = (overflow_in_bit(hl, value, 15) as u8) << FLAG_C_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(hl, value, 11) as u8) << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_C | FLAG_N | FLAG_H)) | c | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// INC R8
///
/// Increments register `R8` by 1.
pub fn inc_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 3);

    let old: u8 = cpu.get_register8(reg);
    cpu.set_register8(reg, old.wrapping_add(1));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((old.wrapping_add(1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(old as u16, 1, 3) as u8) << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H)) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// INC [HL]
///
/// Increments the byte pointed to by `HL` by 1.
pub fn inc_hlmem(cpu: &mut Cpu) -> u8 {
    let old: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.write_memory(cpu.get_register16(Reg16::HL), old.wrapping_add(1));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((old.wrapping_add(1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(old as u16, 1, 3) as u8) << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H)) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// DEC R8
///
/// Decrements register `R8` by 1.
pub fn dec_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 3);
    let old: u8 = cpu.get_register8(reg) as u8;
    cpu.set_register8(reg, old.wrapping_sub(1));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((old.wrapping_sub(1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(old as u16, 1, 3) as u8) << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H)) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// DEC [HL]
///
/// Decrements the byte pointed to by `HL` by 1.
pub fn dec_hlmem(cpu: &mut Cpu) -> u8 {
    let old: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.write_memory(cpu.get_register16(Reg16::HL), old.wrapping_sub(1));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((old.wrapping_sub(1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(old as u16, 1, 3) as u8) << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H)) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// LD R8, IMM8
///
/// Copies the 8-bit immediate value into register `R8`.
pub fn ld_r8_imm8(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_pc();
    cpu.set_register8(get_register8_from_opcode(cpu.opcode, 3), value);
    0
}

/// LD [HL], IMM8
///
/// Copies the 8-bit immediate value into the byte pointed to by `HL`.
pub fn ld_hlmem_imm8(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_pc();
    cpu.write_memory(cpu.get_register16(Reg16::HL), value);
    0
}

// TODO: not tested
/// RLCA
///
/// Rotates register `A` left, copying bit 7 to both bit 0 and the carry flag.
pub fn rlca(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let bit7: u8 = a >> 7;
    cpu.set_register8(Reg8::A, (a << 1) | bit7);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = 0;
    let n = 0;
    let h = 0;
    let c = ((bit7 == 1) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RRCA
///
/// Rotates register `A` right, copying bit 0 to both bit 7 and the carry flag.
pub fn rrca(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let bit0: u8 = a & 1;
    cpu.set_register8(Reg8::A, (a >> 1) | (bit0 << 7));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = 0;
    let n = 0;
    let h = 0;
    let c = ((bit0 == 1) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RLA
///
/// Rotates register `A` left through the carry flag.
pub fn rla(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;

    let bit7: u8 = a >> 7;
    cpu.set_register8(Reg8::A, (a << 1) | carry);

    // Setting flags
    let c = ((bit7 == 1) as u8) << FLAG_C_SHIFT;
    cpu.set_register8(Reg8::F, c);
    0
}

/// RRA
///
/// Rotates register `A` right through the carry flag.
pub fn rra(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let c: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;

    let bit0: u8 = a & 1;
    cpu.set_register8(Reg8::A, (a >> 1) | (c << 7));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = 0;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// DAA
///
/// Decimal-adjusts register `A` after a BCD `ADD`, `ADC`, `SUB`, or `SBC` operation.
pub fn daa(cpu: &mut Cpu) -> u8 {
    let mut flags: u8 = cpu.get_register8(Reg8::F);
    let mut adjustment: u8 = 0;
    let a = cpu.get_register8(Reg8::A);

    if flags & FLAG_N != 0 {    // Subtract case
        adjustment += ((flags & FLAG_H != 0) as u8) * 0x6;
        adjustment += ((flags & FLAG_C != 0) as u8) * 0x60;

        let result = a.wrapping_sub(adjustment);
        cpu.set_register8(Reg8::A, result);

        // Update flags
        let z = ((result == 0) as u8) << FLAG_Z_SHIFT;
        let n = 1 << FLAG_N_SHIFT;
        let h = 0;
        let c = flags & FLAG_C;

        flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H)) | z | n | h | c;
        cpu.set_register8(Reg8::F, flags);
    } else {    // Add case
        adjustment += (((flags & FLAG_H != 0) || (a & 0xF > 9)) as u8) * 0x6;
        adjustment += (((flags & FLAG_C != 0) || (a > 0x99)) as u8) * 0x60;

        let result = a.wrapping_add(adjustment);
        cpu.set_register8(Reg8::A, result);

        // Update flags
        let z = ((result == 0) as u8) << FLAG_Z_SHIFT;
        let n = 0;
        let h = 0;
        let c = (((flags & FLAG_C != 0) || (a > 0x99)) as u8) << FLAG_C_SHIFT;

        flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;
        cpu.set_register8(Reg8::F, flags);
    }
    0
}

/// CPL
///
/// Complements register `A` bitwise.
pub fn cpl(cpu: &mut Cpu) -> u8 {
    cpu.set_register8(Reg8::A, !cpu.get_register8(Reg8::A));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let n = 1 << FLAG_N_SHIFT;
    let h = 1 << FLAG_H_SHIFT;

    flags = (flags & !(FLAG_N | FLAG_H)) | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SCF
///
/// Sets the carry flag.
pub fn scf(cpu: &mut Cpu) -> u8 {
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let n = 0;
    let h = 0;
    let c = 1 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_N | FLAG_H | FLAG_C)) | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// CCF
///
/// Complements the carry flag.
pub fn ccf(cpu: &mut Cpu) -> u8 {
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let n = 0;
    let h = 0;
    let c = ((!((flags & FLAG_C) > 0)) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_N | FLAG_H | FLAG_C)) | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// JR IMM8
///
/// Jumps relative to `PC` by the signed 8-bit immediate offset.
pub fn jr_imm8(cpu: &mut Cpu) -> u8 {
    let offset = cpu.read_pc() as i8;
    let pc = cpu.get_register16(Reg16::PC);

    let new_pc = ((pc as i32) + (offset as i32)) as u16;
    cpu.set_register16(Reg16::PC, new_pc);

    0
}

/// JR COND, IMM8
///
/// Jumps relative to `PC` by the signed 8-bit immediate offset if condition `COND` is met.
pub fn jr_cond_imm8(cpu: &mut Cpu) -> u8 {
    let offset = cpu.read_pc() as i8;

    let c = cpu.get_register8(Reg8::F) & FLAG_C;
    let z = cpu.get_register8(Reg8::F) & FLAG_Z;

    let condition = match cpu.opcode {
        0b0010_0000 => z == 0,   // JR NZ
        0b0010_1000 => z != 0,   // JR Z
        0b0011_0000 => c == 0,   // JR NC
        0b0011_1000 => c != 0,   // JR C
        _ => unreachable!(),
    };

    if condition {
        let pc = cpu.get_register16(Reg16::PC);
        let new_pc = ((pc as i32) + (offset as i32)) as u16;
        cpu.set_register16(Reg16::PC, new_pc);
        return 1;
    }

    0
}

/// STOP
///
/// Enters CPU very low-power mode. This is encoded as a 2-byte instruction.
pub fn stop(cpu: &mut Cpu) -> u8 {
    cpu.read_pc(); // The second byte is always read (normally 0x00)
    cpu.set_stopped(true);
    0
}

// Block 1: 8-bit register to register loads
/// HALT
///
/// Enters CPU low-power mode until an interrupt occurs.
pub fn halt(cpu: &mut Cpu) -> u8 {
    let pending_interrupts: bool = (cpu.get_ie() & cpu.get_if()) != 0;

    if !cpu.get_ime() && pending_interrupts {
        cpu.set_halt_bug(true);
    } else {
        cpu.set_halted(true);
    }

    0
}

/// LD R8, R8
///
/// Copies the value in the source 8-bit register into the destination 8-bit register.
/// Storing a register into itself is a no-op.
pub fn ld_r8_r8(cpu: &mut Cpu) -> u8 {
    let src: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let dst: Reg8 = get_register8_from_opcode(cpu.opcode, 3);

    cpu.set_register8(dst, cpu.get_register8(src));
    0
}

/// LD R8, [HL]
///
/// Copies the byte pointed to by `HL` into register `R8`.
pub fn ld_r8_hlmem(cpu: &mut Cpu) -> u8 {
    let dst: Reg8 = get_register8_from_opcode(cpu.opcode, 3);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));

    cpu.set_register8(dst, value);
    0
}

/// LD [HL], R8
///
/// Copies register `R8` into the byte pointed to by `HL`.
pub fn ld_hlmem_r8(cpu: &mut Cpu) -> u8 {
    let src: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(src);

    cpu.write_memory(cpu.get_register16(Reg16::HL), value);
    0
}

// Block 2: 8-bit arithmetic
/// ADD A, R8
///
/// Adds register `R8` to register `A`.
pub fn add_a_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_add(value));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((a.wrapping_add(value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = (overflow_in_bit(a as u16, value as u16, 7) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// ADD A, [HL]
///
/// Adds the byte pointed to by `HL` to register `A`.
pub fn add_a_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_add(value));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a.wrapping_add(value)) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = (overflow_in_bit(a as u16, value as u16, 7) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// ADC A, R8
///
/// Adds register `R8` plus the carry flag to register `A`.
pub fn adc_a_r8(cpu: &mut Cpu) -> u8 {
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let sum = a as u16 + value as u16 + carry as u16;
    let res: u8 = sum as u8;
    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = ((((a & 0x0F) as u16 + (value & 0x0F) as u16 + carry as u16) > 0x0F) as u8)
        << FLAG_H_SHIFT;
    let c = ((sum > 0xFF) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// ADC A, [HL]
///
/// Adds the byte pointed to by `HL` plus the carry flag to register `A`.
pub fn adc_a_hlmem(cpu: &mut Cpu) -> u8 {
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));

    let sum = a as u16 + value as u16 + carry as u16;
    let res: u8 = sum as u8;
    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = ((((a & 0x0F) as u16 + (value & 0x0F) as u16 + carry as u16) > 0x0F) as u8)
        << FLAG_H_SHIFT;
    let c = ((sum > 0xFF) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SUB A, R8
///
/// Subtracts register `R8` from register `A`.
pub fn sub_a_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_sub(value));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((a.wrapping_sub(value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((value > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SUB A, [HL]
///
/// Subtracts the byte pointed to by `HL` from register `A`.
pub fn sub_a_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_sub(value));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a.wrapping_sub(value)) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((value > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SBC A, R8
///
/// Subtracts register `R8` and the carry flag from register `A`.
pub fn sbc_a_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.get_register8(reg);
    let subtrahend = value as u16 + carry as u16;
    let res = a.wrapping_sub(value).wrapping_sub(carry);

    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = ((((a & 0x0F) as u16) < ((value & 0x0F) as u16 + carry as u16)) as u8)
        << FLAG_H_SHIFT;
    let c = (((a as u16) < subtrahend) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SBC A, [HL]
///
/// Subtracts the byte pointed to by `HL` and the carry flag from register `A`.
pub fn sbc_a_hlmem(cpu: &mut Cpu) -> u8 {
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let subtrahend = value as u16 + carry as u16;
    let res: u8 = a.wrapping_sub(value).wrapping_sub(carry);
    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = ((((a & 0x0F) as u16) < ((value & 0x0F) as u16 + carry as u16)) as u8)
        << FLAG_H_SHIFT;
    let c = (((a as u16) < subtrahend) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// AND A, R8
///
/// Sets register `A` to the bitwise AND between `A` and register `R8`.
pub fn and_a_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.get_register8(reg);

    cpu.set_register8(Reg8::A, a & value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a & value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 1 << FLAG_H_SHIFT;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// AND A, [HL]
///
/// Sets register `A` to the bitwise AND between `A` and the byte pointed to by `HL`.
pub fn and_a_hlmem(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.set_register8(Reg8::A, a & value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a & value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 1 << FLAG_H_SHIFT;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// XOR A, R8
///
/// Sets register `A` to the bitwise XOR between `A` and register `R8`.
pub fn xor_a_r8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.get_register8(get_register8_from_opcode(cpu.opcode, 0));
    cpu.set_register8(Reg8::A, a ^ value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a ^ value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// XOR A, [HL]
///
/// Sets register `A` to the bitwise XOR between `A` and the byte pointed to by `HL`.
pub fn xor_a_hlmem(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.set_register8(Reg8::A, a ^ value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a ^ value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// OR A, R8
///
/// Sets register `A` to the bitwise OR between `A` and register `R8`.
pub fn or_a_r8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.get_register8(get_register8_from_opcode(cpu.opcode, 0));
    cpu.set_register8(Reg8::A, a | value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a | value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// OR A, [HL]
///
/// Sets register `A` to the bitwise OR between `A` and the byte pointed to by `HL`.
pub fn or_a_hlmem(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.set_register8(Reg8::A, a | value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a | value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// CP A, R8
///
/// Compares register `A` with register `R8` by subtracting `R8` and discarding the result.
pub fn cp_a_r8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.get_register8(get_register8_from_opcode(cpu.opcode, 0));
    let res: u8 = a.wrapping_sub(value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((value > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// CP A, [HL]
///
/// Compares register `A` with the byte pointed to by `HL` and discards the result.
pub fn cp_a_hlmem(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let res: u8 = a.wrapping_sub(value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((value > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// Block 3: 8-bit arithmetic with immediate
/// ADD A, IMM8
///
/// Adds the 8-bit immediate value to register `A`.
pub fn add_a_imm8(cpu: &mut Cpu) -> u8 {
    let byte: u8 = cpu.read_pc();
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_add(byte));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((a.wrapping_add(byte) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = (overflow_in_bit(a as u16, byte as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = (overflow_in_bit(a as u16, byte as u16, 7) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// ADC A, IMM8
///
/// Adds the 8-bit immediate value plus the carry flag to register `A`.
pub fn adc_a_imm8(cpu: &mut Cpu) -> u8 {
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();
    let sum = a as u16 + value as u16 + carry as u16;
    let res = sum as u8;
    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let h = ((((a & 0x0F) as u16 + (value & 0x0F) as u16 + carry as u16) > 0x0F) as u8)
        << FLAG_H_SHIFT;
    let c = ((sum > 0xFF) as u8) << FLAG_C_SHIFT;
    let n = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SUB A, IMM8
///
/// Subtracts the 8-bit immediate value from register `A`.
pub fn sub_a_imm8(cpu: &mut Cpu) -> u8 {
    let byte: u8 = cpu.read_pc();
    let a: u8 = cpu.get_register8(Reg8::A);
    cpu.set_register8(Reg8::A, a.wrapping_sub(byte));

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((a.wrapping_sub(byte) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, byte as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((byte > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SBC A, IMM8
///
/// Subtracts the 8-bit immediate value and the carry flag from register `A`.
pub fn sbc_a_imm8(cpu: &mut Cpu) -> u8 {
    let carry: u8 = (cpu.get_register8(Reg8::F) & FLAG_C) >> FLAG_C_SHIFT;
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();
    let subtrahend = value as u16 + carry as u16;
    let res: u8 = a.wrapping_sub(value).wrapping_sub(carry);
    cpu.set_register8(Reg8::A, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = ((((a & 0x0F) as u16) < ((value & 0x0F) as u16 + carry as u16)) as u8)
        << FLAG_H_SHIFT;
    let c = (((a as u16) < subtrahend) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// AND A, IMM8
///
/// Sets register `A` to the bitwise AND between `A` and the 8-bit immediate value.
pub fn and_a_imm8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();
    cpu.set_register8(Reg8::A, a & value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a & value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 1 << FLAG_H_SHIFT;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// XOR A, IMM8
///
/// Sets register `A` to the bitwise XOR between `A` and the 8-bit immediate value.
pub fn xor_a_imm8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();
    cpu.set_register8(Reg8::A, a ^ value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a ^ value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// OR A, IMM8
///
/// Sets register `A` to the bitwise OR between `A` and the 8-bit immediate value.
pub fn or_a_imm8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();
    cpu.set_register8(Reg8::A, a | value);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((a | value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// CP A, IMM8
///
/// Compares register `A` with the 8-bit immediate value and discards the result.
pub fn cp_a_imm8(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let value: u8 = cpu.read_pc();

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((a.wrapping_sub(value) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 1 << FLAG_N_SHIFT;
    let h = (borrow_in_bit(a as u16, value as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = ((value > a) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// RET COND
///
/// Returns from a subroutine if condition `COND` is met.
pub fn ret_cond(cpu: &mut Cpu) -> u8 {
    let f: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = f & FLAG_C;
    let z: u8 = f & FLAG_Z;

    let condition: bool = match cpu.opcode {
        0b1100_0000 => z == 0,  // Case flag Z is not set
        0b1100_1000 => z != 0,  // Case flag Z is set
        0b1101_0000 => c == 0,  // Case flag C is not set
        0b1101_1000 => c != 0,  // Case flag C is set
        _ => unreachable!("The condition for returning is unreachable."),
    };

    if condition {
        let sp: u16 = cpu.get_register16(Reg16::SP);

        let byte1: u8 = cpu.read_memory(sp);
        let byte2: u8 = cpu.read_memory(sp.wrapping_add(1));
        cpu.set_register16(Reg16::PC, u16::from_le_bytes([byte1, byte2]));

        cpu.set_register16(Reg16::SP, sp.wrapping_add(2));
        return 3;
    }
    0
}

/// RET
///
/// Returns from a subroutine by popping the return address into `PC`.
pub fn ret(cpu: &mut Cpu) -> u8 {
    let sp: u16 = cpu.get_register16(Reg16::SP);

    let byte1: u8 = cpu.read_memory(sp);
    let byte2: u8 = cpu.read_memory(sp.wrapping_add(1));
    cpu.set_register16(Reg16::PC, u16::from_le_bytes([byte1, byte2]));

    cpu.set_register16(Reg16::SP, sp.wrapping_add(2));
    0
}

// TODO: not tested
/// RETI
///
/// Returns from a subroutine and enables interrupts.
pub fn reti(cpu: &mut Cpu) -> u8 {
    let sp: u16 = cpu.get_register16(Reg16::SP);

    let byte1: u8 = cpu.read_memory(sp);
    let byte2: u8 = cpu.read_memory(sp.wrapping_add(1));
    cpu.set_register16(Reg16::PC, u16::from_le_bytes([byte1, byte2]));

    cpu.set_register16(Reg16::SP, sp.wrapping_add(2));

    cpu.set_ime_delay(0);
    cpu.set_ime(true);

    0
}

/// JP COND, IMM16
///
/// Jumps to address `IMM16` if condition `COND` is met.
pub fn jp_cond_imm16(cpu: &mut Cpu) -> u8 {
    let f: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = f & FLAG_C;
    let z: u8 = f & FLAG_Z;

    let condition: bool = match cpu.opcode {
        0b1100_0010 => z == 0,  // Case flag Z not set
        0b1100_1010 => z != 0,  // Case flag Z set
        0b1101_0010 => c == 0,  // Case flag C not set
        0b1101_1010 => c != 0,  // Case flag C set
        _ => unreachable!("The condition for returning is unreachable."),
    };

    let lo = cpu.read_pc();
    let hi = cpu.read_pc();
    let target = u16::from_le_bytes([lo, hi]);

    if condition {
        cpu.set_register16(Reg16::PC, target);
        return 1;
    }

    0
}

/// JP IMM16
///
/// Jumps to address `IMM16`.
pub fn jp_imm16(cpu: &mut Cpu) -> u8 {
    let byte1: u8 = cpu.read_pc();
    let byte2: u8 = cpu.read_pc();
    let res: u16 = u16::from_le_bytes([byte1, byte2]);

    cpu.set_register16(Reg16::PC, res);
    0
}

/// JP HL
///
/// Jumps to the address stored in register `HL`.
pub fn jp_hl(cpu: &mut Cpu) -> u8 {
    cpu.set_register16(Reg16::PC, cpu.get_register16(Reg16::HL));
    0
}

/// CALL COND, IMM16
///
/// Calls address `IMM16` if condition `COND` is met.
pub fn call_cond_imm16(cpu: &mut Cpu) -> u8 {
    let f: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = f & FLAG_C;
    let z: u8 = f & FLAG_Z;

    let condition: bool = match cpu.opcode {
        0b1100_0100 => z == 0,    // Case flag Z is not set
        0b1100_1100 => z != 0,    // Case flag Z is set
        0b1101_0100 => c == 0,    // Case flag C is not set
        0b1101_1100 => c != 0,    // Case flag C is set
        _ => unreachable!("The condition for returning is unreachable."),
    };

    let lo = cpu.read_pc();
    let hi = cpu.read_pc();
    let target = u16::from_le_bytes([lo, hi]);

    if condition {
        let return_addr = cpu.get_register16(Reg16::PC);

        let mut sp = cpu.get_register16(Reg16::SP);

        let ret_lo = (return_addr & 0x00FF) as u8;
        let ret_hi = (return_addr >> 8) as u8;

        // Push return address
        sp -= 1;
        cpu.write_memory(sp, ret_hi);
        sp -= 1;
        cpu.write_memory(sp, ret_lo);

        cpu.set_register16(Reg16::SP, sp);

        // Jump to target
        cpu.set_register16(Reg16::PC, target);
        return 3;
    }
    0
}

/// CALL IMM16
///
/// Calls address `IMM16` by pushing the return address and jumping to `IMM16`.
pub fn call_imm16(cpu: &mut Cpu) -> u8 {
    let lo = cpu.read_pc();
    let hi = cpu.read_pc();
    let target = u16::from_le_bytes([lo, hi]);

    let return_addr = cpu.get_register16(Reg16::PC);

    let mut sp = cpu.get_register16(Reg16::SP);

    let ret_lo = (return_addr & 0x00FF) as u8;
    let ret_hi = (return_addr >> 8) as u8;

    // Push return address
    sp -= 1;
    cpu.write_memory(sp, ret_hi);
    sp -= 1;
    cpu.write_memory(sp, ret_lo);

    cpu.set_register16(Reg16::SP, sp);

    // Jump to target
    cpu.set_register16(Reg16::PC, target);
    0
}

/// RST VEC
///
/// Calls fixed restart vector `VEC`.
pub fn rst_tgt3(cpu: &mut Cpu) -> u8 {
    let addr: u16 = match cpu.opcode {
        0b1100_0111 => 0x00,
        0b1100_1111 => 0x08,
        0b1101_0111 => 0x10,
        0b1101_1111 => 0x18,
        0b1110_0111 => 0x20,
        0b1110_1111 => 0x28,
        0b1111_0111 => 0x30,
        0b1111_1111 => 0x38,
        _ => unreachable!(),
    };

    let pc = cpu.get_register16(Reg16::PC);
    let sp = cpu.get_register16(Reg16::SP);

    let pc_lo = (pc & 0x00FF) as u8;
    let pc_hi = (pc >> 8) as u8;

    cpu.write_memory(sp.wrapping_sub(1), pc_hi);
    cpu.write_memory(sp.wrapping_sub(2), pc_lo);

    cpu.set_register16(Reg16::SP, sp.wrapping_sub(2));
    cpu.set_register16(Reg16::PC, addr);

    0
}

/// POP R16
///
/// Pops a 16-bit value from the stack into register `R16`.
pub fn pop_r16stk(cpu: &mut Cpu) -> u8 {
    let reg: Reg16 = match cpu.opcode {
        0b1111_0001 => Reg16::AF,
        _ => get_register16_from_opcode(cpu.opcode, 4),
    };

    let sp: u16 = cpu.get_register16(Reg16::SP);

    let byte1: u8 = cpu.read_memory(sp);
    let byte2: u8 = cpu.read_memory(sp.wrapping_add(1));

    cpu.set_register16(reg, u16::from_le_bytes([byte1, byte2]));
    cpu.set_register16(Reg16::SP, sp.wrapping_add(2));
    0
}

/// POP AF
///
/// Pops register pair `AF` from the stack, preserving only valid flag bits in `F`.
pub fn pop_af(cpu: &mut Cpu) -> u8 {
    let sp: u16 = cpu.get_register16(Reg16::SP);

    let byte1: u8 = cpu.read_memory(sp);
    let byte2: u8 = cpu.read_memory(sp.wrapping_add(1));

    cpu.set_register8(Reg8::F, byte1 & 0xF0);
    cpu.set_register8(Reg8::A, byte2);
    cpu.set_register16(Reg16::SP, sp.wrapping_add(2));
    0
}

/// PUSH R16
///
/// Pushes register `R16` onto the stack.
pub fn push_r16stk(cpu: &mut Cpu) -> u8 {
    let reg: Reg16 = match cpu.opcode {
        0b1111_0001 => Reg16::AF,
        _ => get_register16_from_opcode(cpu.opcode, 4),
    };

    let mut sp: u16 = cpu.get_register16(Reg16::SP);
    let value: u16 = cpu.get_register16(reg);

    let high = ((value & 0xFF00) >> 8) as u8;
    let low = (value & 0x00FF) as u8;

    sp -= 1;
    cpu.write_memory(sp, high);
    sp -= 1;
    cpu.write_memory(sp, low);

    cpu.set_register16(Reg16::SP, sp);
    0
}

/// PUSH AF
///
/// Pushes register pair `AF` onto the stack.
pub fn push_af(cpu: &mut Cpu) -> u8 {
    let mut sp: u16 = cpu.get_register16(Reg16::SP);
    let value: u16 = cpu.get_register16(Reg16::AF);

    let high = ((value & 0xFF00) >> 8) as u8;
    let low = (value & 0x00FF) as u8;

    sp -= 1;
    cpu.write_memory(sp, high);
    sp -= 1;
    cpu.write_memory(sp, low);

    cpu.set_register16(Reg16::SP, sp);
    0
}

/// LDH [C], A
///
/// Copies register `A` into the byte at address `$FF00 + C`.
pub fn ldh_c_a(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let c: u8 = cpu.get_register8(Reg8::C);
    cpu.write_memory(0xFF00u16 + (c as u16), a);
    0
}

/// LDH [IMM8], A
///
/// Copies register `A` into the byte at address `$FF00 + IMM8`.
pub fn ldh_imm16_a(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let offset: u8 = cpu.read_pc();
    cpu.write_memory(0xFF00u16 + (offset as u16), a);
    0
}

/// LD [IMM16], A
///
/// Copies register `A` into the byte at address `IMM16`.
pub fn ld_imm16_a(cpu: &mut Cpu) -> u8 {
    let a: u8 = cpu.get_register8(Reg8::A);
    let byte1: u8 = cpu.read_pc();
    let byte2: u8 = cpu.read_pc();
    let addr: u16 = u16::from_le_bytes([byte1, byte2]);
    cpu.write_memory(addr, a);
    0
}

/// LDH A, [C]
///
/// Copies the byte at address `$FF00 + C` into register `A`.
pub fn ldh_a_c(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory((cpu.get_register8(Reg8::C) as u16) + 0xFF00u16);
    cpu.set_register8(Reg8::A, value);
    0
}

/// LDH A, [IMM8]
///
/// Copies the byte at address `$FF00 + IMM8` into register `A`.
pub fn ldh_a_imm16(cpu: &mut Cpu) -> u8 {
    let offset: u8 = cpu.read_pc();
    let value: u8 = cpu.read_memory(0xFF00u16 + (offset as u16));
    cpu.set_register8(Reg8::A, value);
    0
}

/// LD A, [IMM16]
///
/// Copies the byte at address `IMM16` into register `A`.
pub fn ld_a_imm16(cpu: &mut Cpu) -> u8 {
    let byte1: u8 = cpu.read_pc();
    let byte2: u8 = cpu.read_pc();
    let addr: u16 = u16::from_le_bytes([byte1, byte2]);
    let value: u8 = cpu.read_memory(addr);
    cpu.set_register8(Reg8::A, value);
    0
}

// TODO: not tested
/// ADD SP, IMM8
///
/// Adds the signed 8-bit immediate value to `SP`.
pub fn add_sp_imm8(cpu: &mut Cpu) -> u8 {
    let sp: u16 = cpu.get_register16(Reg16::SP);
    let byte: i16 = cpu.read_pc() as i8 as i16;
    let res: u16 = sp.wrapping_add(byte as u16);

    cpu.set_register16(Reg16::SP, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = 0;
    let n = 0;
    let h = (overflow_in_bit(sp, byte as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = (overflow_in_bit(sp, byte as u16, 7) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// LD HL, SP+IMM8
///
/// Adds the signed 8-bit immediate value to `SP` and stores the result in `HL`.
pub fn ld_hli_imm8(cpu: &mut Cpu) -> u8 {
    let sp: u16 = cpu.get_register16(Reg16::SP);
    let byte: i16 = cpu.read_pc() as i8 as i16;
    let res: u16 = sp.wrapping_add(byte as u16);

    cpu.set_register16(Reg16::HL, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = 0;
    let n = 0;
    let h = (overflow_in_bit(sp, byte as u16, 3) as u8) << FLAG_H_SHIFT;
    let c = (overflow_in_bit(sp, byte as u16, 7) as u8) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// LD SP, HL
///
/// Copies register `HL` into register `SP`.
pub fn ld_sp_hl(cpu: &mut Cpu) -> u8 {
    cpu.set_register16(Reg16::SP, cpu.get_register16(Reg16::HL));
    0
}

/// DI
///
/// Disables interrupts by clearing `IME`.
pub fn di(cpu: &mut Cpu) -> u8 {
    cpu.set_ime(false);

    // We should cancel the pending IME enabling request
    cpu.set_ime_delay(0);
    0
}

/// EI
///
/// Enables interrupts after the following instruction.
pub fn ei(cpu: &mut Cpu) -> u8 {
    cpu.set_ime_delay(2);
    0
}

// Bloque $CB: prefijo de instrucciones
// TODO: not tested
/// RLC R8
///
/// Rotates register `R8` left, copying bit 7 to both bit 0 and the carry flag.
pub fn rlc_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit7: u8 = value >> 7;
    let res: u8 = (value << 1) | bit7;
    cpu.set_register8(reg, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit7 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RLC [HL]
///
/// Rotates the byte pointed to by `HL` left, copying bit 7 to both bit 0 and the carry flag.
pub fn rlc_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let bit7: u8 = value >> 7;
    let res: u8 = (value << 1) | bit7;
    cpu.write_memory(cpu.get_register16(Reg16::HL), res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit7 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RRC R8
///
/// Rotates register `R8` right, copying bit 0 to both bit 7 and the carry flag.
pub fn rrc_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit0: u8 = value & 1;
    let res: u8 = (value >> 1) | (bit0 << 7);
    cpu.set_register8(reg, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RRC [HL]
///
/// Rotates the byte pointed to by `HL` right, copying bit 0 to both bit 7 and the carry flag.
pub fn rrc_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let bit0: u8 = value & 1;
    let res: u8 = (value >> 1) | (bit0 << 7);
    cpu.write_memory(cpu.get_register16(Reg16::HL), res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RL R8
///
/// Rotates register `R8` left through the carry flag.
pub fn rl_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let mut flags: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = (flags & FLAG_C) >> FLAG_C_SHIFT;
    let value: u8 = cpu.get_register8(reg);
    let res: u8 = (value << 1) | c;
    cpu.set_register8(reg, res);

    // Setting flags
    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c_flag = (value >> 7) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c_flag;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RL [HL]
///
/// Rotates the byte pointed to by `HL` left through the carry flag.
pub fn rl_hlmem(cpu: &mut Cpu) -> u8 {
    let mut flags: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = (flags & FLAG_C) >> FLAG_C_SHIFT;
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let res: u8 = (value << 1) | c;
    cpu.write_memory(cpu.get_register16(Reg16::HL), res);

    // Setting flags
    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c_flag = (value >> 7) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c_flag;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// RR R8
///
/// Rotates register `R8` right through the carry flag.
pub fn rr_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let mut flags: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = (flags & FLAG_C) >> FLAG_C_SHIFT;
    let value: u8 = cpu.get_register8(reg);
    let result: u8 = (value >> 1) | (c << 7);
    cpu.set_register8(reg, result);

    // Setting flags
    let z = ((result == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c_flag = (value & 1) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c_flag;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RR [HL]
///
/// Rotates the byte pointed to by `HL` right through the carry flag.
pub fn rr_hlmem(cpu: &mut Cpu) -> u8 {
    let mut flags: u8 = cpu.get_register8(Reg8::F);
    let c: u8 = (flags & FLAG_C) >> FLAG_C_SHIFT;
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let res: u8 = (value >> 1) | (c << 7);
    cpu.write_memory(cpu.get_register16(Reg16::HL), res);

    // Setting flags
    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c_flag = (value & 1) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c_flag;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SLA R8
///
/// Shifts register `R8` left arithmetically, moving bit 7 into the carry flag.
pub fn sla_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    cpu.set_register8(reg, value << 1);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value << 1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c_flag = (value >> 7) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c_flag;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SLA [HL]
///
/// Shifts the byte pointed to by `HL` left arithmetically, moving bit 7 into the carry flag.
pub fn sla_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    cpu.write_memory(cpu.get_register16(Reg16::HL), value << 1);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value << 1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = (value >> 7) << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SRA R8
///
/// Shifts register `R8` right arithmetically, preserving bit 7 and moving bit 0 into the carry flag.
pub fn sra_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit7: u8 = value & 0x80;
    let bit0: u8 = value & 1;
    let res: u8 = (value >> 1) | bit7;
    cpu.set_register8(reg, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SRA [HL]
///
/// Shifts the byte pointed to by `HL` right arithmetically, preserving bit 7 and moving bit 0 into the carry flag.
pub fn sra_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let bit7: u8 = value & 0x80;
    let bit0: u8 = value & 1;
    let res: u8 = (value >> 1) | bit7;
    cpu.write_memory(cpu.get_register16(Reg16::HL), res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SWAP R8
///
/// Swaps the upper and lower nibbles of register `R8`.
pub fn swap_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let res: u8 = (value >> 4) | (value << 4);
    cpu.set_register8(reg, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SWAP [HL]
///
/// Swaps the upper and lower nibbles of the byte pointed to by `HL`.
pub fn swap_hlmem(cpu: &mut Cpu) -> u8 {
    let addr = cpu.get_register16(Reg16::HL);
    let value: u8 = cpu.read_memory(addr);
    let res: u8 = (value >> 4) | (value << 4);
    cpu.write_memory(addr, res);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = ((res == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = 0;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// SRL R8
///
/// Shifts register `R8` right logically, moving bit 0 into the carry flag.
pub fn srl_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit0: u8 = value & 1;
    cpu.set_register8(reg, value >> 1);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value >> 1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// SRL [HL]
///
/// Shifts the byte pointed to by `HL` right logically, moving bit 0 into the carry flag.
pub fn srl_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let bit0: u8 = value & 1;
    cpu.write_memory(cpu.get_register16(Reg16::HL), value >> 1);

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value >> 1) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = 0;
    let c = bit0 << FLAG_C_SHIFT;

    flags = (flags & !(FLAG_Z | FLAG_N | FLAG_H | FLAG_C)) | z | n | h | c;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// BIT u3, R8
///
/// Tests bit `u3` in register `R8` and sets the zero flag if the bit is 0.
pub fn bit_b3_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit: u8 = (cpu.opcode >> 3) & 0b0000_0111;

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value & (1 << bit)) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = FLAG_H;

    flags = (flags & FLAG_C) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

/// BIT u3, [HL]
///
/// Tests bit `u3` in the byte pointed to by `HL` and sets the zero flag if the bit is 0.
pub fn bit_b3_hlmem(cpu: &mut Cpu) -> u8 {
    let value: u8 = cpu.read_memory(cpu.get_register16(Reg16::HL));
    let bit: u8 = (cpu.opcode >> 3) & 0b0000_0111;

    // Setting flags
    let mut flags: u8 = cpu.get_register8(Reg8::F);

    let z = (((value & (1 << bit)) == 0) as u8) << FLAG_Z_SHIFT;
    let n = 0;
    let h = FLAG_H;

    flags = (flags & FLAG_C) | z | n | h;

    cpu.set_register8(Reg8::F, flags);
    0
}

// TODO: not tested
/// RES u3, R8
///
/// Resets bit `u3` in register `R8` to 0.
pub fn res_b3_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit: u8 = (cpu.opcode >> 3) & 0b0000_0111;
    cpu.set_register8(reg, value & !(1 << bit));
    0
}

// TODO: not tested
/// RES u3, [HL]
///
/// Resets bit `u3` in the byte pointed to by `HL` to 0.
pub fn res_b3_hlmem(cpu: &mut Cpu) -> u8 {
    let addr = cpu.get_register16(Reg16::HL);
    let value = cpu.read_memory(addr);
    let bit = (cpu.opcode >> 3) & 0b0000_0111;

    cpu.write_memory(addr, value & !(1 << bit));
    0
}

// TODO: not tested
/// SET u3, R8
///
/// Sets bit `u3` in register `R8` to 1.
pub fn set_b3_r8(cpu: &mut Cpu) -> u8 {
    let reg: Reg8 = get_register8_from_opcode(cpu.opcode, 0);
    let value: u8 = cpu.get_register8(reg);
    let bit: u8 = (cpu.opcode >> 3) & 0b0000_0111;
    cpu.set_register8(reg, value | (1 << bit));
    0
}

// TODO: not tested
/// SET u3, [HL]
///
/// Sets bit `u3` in the byte pointed to by `HL` to 1.
pub fn set_b3_hlmem(cpu: &mut Cpu) -> u8 {
    let addr = cpu.get_register16(Reg16::HL);
    let value = cpu.read_memory(addr);
    let bit = (cpu.opcode >> 3) & 0b0000_0111;

    cpu.write_memory(addr, value | (1 << bit));
    0
}

// -------------------------------- Unit tests ----------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::MemoryBus;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn test_cpu() -> Cpu {
        let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
        Cpu::new(memory)
    }

    #[test]
    fn rst_opcodes_are_registered() {
        let expected = [
            (0xC7, "RST 0x00"),
            (0xCF, "RST 0x08"),
            (0xD7, "RST 0x10"),
            (0xDF, "RST 0x18"),
            (0xE7, "RST 0x20"),
            (0xEF, "RST 0x28"),
            (0xF7, "RST 0x30"),
            (0xFF, "RST 0x38"),
        ];

        for (opcode, asm) in expected {
            let instruction = OPCODES_TABLE[opcode];
            assert_eq!(instruction.asm, asm);
            assert_eq!(instruction.cycles, 4);
            assert_eq!(instruction.bytes, 1);
        }
    }

    #[test]
    fn rla_rotates_a_through_carry() {
        let mut cpu = test_cpu();

        cpu.set_register8(Reg8::A, 0x80);
        cpu.set_register8(Reg8::F, FLAG_C);

        rla(&mut cpu);

        assert_eq!(cpu.get_register8(Reg8::A), 0x01);
        assert_eq!(cpu.get_register8(Reg8::F), FLAG_C);
    }

    #[test]
    fn adc_uses_full_carry_input_for_flags() {
        let mut cpu = test_cpu();

        cpu.opcode = 0x88;
        cpu.set_register8(Reg8::A, 0x00);
        cpu.set_register8(Reg8::B, 0xFF);
        cpu.set_register8(Reg8::F, FLAG_C);

        adc_a_r8(&mut cpu);

        assert_eq!(cpu.get_register8(Reg8::A), 0x00);
        assert_eq!(cpu.get_register8(Reg8::F), FLAG_Z | FLAG_H | FLAG_C);
    }

    #[test]
    fn sbc_sets_subtract_half_carry_and_carry_flags() {
        let mut cpu = test_cpu();

        cpu.opcode = 0x98;
        cpu.set_register8(Reg8::A, 0x00);
        cpu.set_register8(Reg8::B, 0x00);
        cpu.set_register8(Reg8::F, FLAG_C);

        sbc_a_r8(&mut cpu);

        assert_eq!(cpu.get_register8(Reg8::A), 0xFF);
        assert_eq!(cpu.get_register8(Reg8::F), FLAG_N | FLAG_H | FLAG_C);
    }

    #[test]
    fn swap_exchanges_nibbles() {
        let mut cpu = test_cpu();

        cpu.opcode = 0x37;
        cpu.set_register8(Reg8::A, 0xF0);
        cpu.set_register8(Reg8::F, FLAG_Z | FLAG_N | FLAG_H | FLAG_C);

        swap_r8(&mut cpu);

        assert_eq!(cpu.get_register8(Reg8::A), 0x0F);
        assert_eq!(cpu.get_register8(Reg8::F), 0);
    }

    #[test]
    fn bit_preserves_carry_flag() {
        let mut cpu = test_cpu();

        cpu.opcode = 0x40;
        cpu.set_register8(Reg8::B, 0x00);
        cpu.set_register8(Reg8::F, FLAG_C);

        bit_b3_r8(&mut cpu);

        assert_eq!(cpu.get_register8(Reg8::F), FLAG_Z | FLAG_H | FLAG_C);
    }

    #[test]
    fn reti_enables_ime_immediately() {
        let mut cpu = test_cpu();

        cpu.set_register16(Reg16::SP, 0xC000);
        cpu.write_memory(0xC000, 0x34);
        cpu.write_memory(0xC001, 0x12);

        reti(&mut cpu);

        assert_eq!(cpu.get_register16(Reg16::PC), 0x1234);
        assert_eq!(cpu.get_register16(Reg16::SP), 0xC002);
        assert!(cpu.interrupt_master_enabled());
    }

    mod get_register_tests {
        use super::*;
        #[test]
        pub fn reg8_pos3() {
            assert_eq!(get_register8_from_opcode(0b1000_0100, 3), Reg8::B);
            assert_eq!(get_register8_from_opcode(0b0000_1100, 3), Reg8::C);
            assert_eq!(get_register8_from_opcode(0b0101_0101, 3), Reg8::D);
            assert_eq!(get_register8_from_opcode(0b0001_1100, 3), Reg8::E);
            assert_eq!(get_register8_from_opcode(0b0010_0100, 3), Reg8::H);
            assert_eq!(get_register8_from_opcode(0b0010_1101, 3), Reg8::L);
            assert_eq!(get_register8_from_opcode(0b1011_1110, 3), Reg8::A);
        }

        #[test]
        pub fn reg8_pos0() {
            assert_eq!(get_register8_from_opcode(0b0001_0010, 0), Reg8::D);
            assert_eq!(get_register8_from_opcode(0b0000_0000, 0), Reg8::B);
            assert_eq!(get_register8_from_opcode(0b0000_1001, 0), Reg8::C);
            assert_eq!(get_register8_from_opcode(0b0001_1011, 0), Reg8::E);
            assert_eq!(get_register8_from_opcode(0b0010_0100, 0), Reg8::H);
            assert_eq!(get_register8_from_opcode(0b0010_1101, 0), Reg8::L);
            assert_eq!(get_register8_from_opcode(0b0011_1111, 0), Reg8::A);
        }

        #[test]
        pub fn reg16_pos4() {
            assert_eq!(get_register16_from_opcode(0b1100_0100, 4), Reg16::BC);
            assert_eq!(get_register16_from_opcode(0b0101_0101, 4), Reg16::DE);
            assert_eq!(get_register16_from_opcode(0b0010_0110, 4), Reg16::HL);
            assert_eq!(get_register16_from_opcode(0b1011_1001, 4), Reg16::SP);
        }
    }

    mod overflow_in_bit_tests {
        use super::*;
        #[test]
        pub fn overflow_in_bit0() {
            assert!(overflow_in_bit(1, 1, 0));
            assert!(!overflow_in_bit(0, 0, 0));
            assert!(!overflow_in_bit(1, 0, 0));
            assert!(!overflow_in_bit(0, 1, 0));
        }

        #[test]
        pub fn overflow_in_bit3() {
            assert!(overflow_in_bit(0b0010_1111, 1, 3));
            assert!(!overflow_in_bit(0b1110_1001, 1, 3));
            assert!(!overflow_in_bit(0b0010_0000, 1, 3));
            assert!(!overflow_in_bit(0b01010_1011, 1, 3));
            assert!(!overflow_in_bit(0b0010_0111, 0b1111_0001, 3));
        }

        #[test]
        pub fn overflow_in_bit7() {
            assert!(overflow_in_bit(0b1111_1111, 0b1111_1111, 7));
            assert!(!overflow_in_bit(0b0111_1111, 0b0111_1111, 7));
        }

        #[test]
        pub fn overflow_in_bit15() {
            assert!(overflow_in_bit(
                0b1111_1111_1111_1111,
                0b1111_1111_1111_1111,
                15
            ));
            assert!(!overflow_in_bit(
                0b0111_1111_1111_1111,
                0b0111_1111_1111_1111,
                15
            ));
        }

        #[test]
        pub fn overflow_in_bits() {
            assert!(overflow_in_bit(0xFFFF, 1, 14));
            assert!(!overflow_in_bit(0, 0xFF0F, 13));
            assert!(overflow_in_bit(0xFFFF, 1, 12));
            assert!(!overflow_in_bit(0b1111_1111_1111_1111, 0b0, 11));
            assert!(overflow_in_bit(0b0000_1111_1111_1111, 1, 10));
            assert!(overflow_in_bit(0b0000_1111_1111_1111, 1, 9));
            assert!(!overflow_in_bit(0b0000_1111_0000_1111, 0, 8));
            assert!(overflow_in_bit(0b0110_1111, 0b0011_1111, 6));
            assert!(overflow_in_bit(0b1110_0000, 0b1111_1111, 5));
            assert!(overflow_in_bit(0b0001_1111, 0b0101_1000, 4));
            assert!(!overflow_in_bit(0b1010_1001, 0b1110_0000, 2));
            assert!(overflow_in_bit(0b0000_0010, 0b1111_0110, 1));
        }
    }

    mod borrow_in_bit_tests {
        use super::*;

        #[test]
        pub fn borrow_in_bit0() {
            assert!(borrow_in_bit(0, 1, 0));
        }

        #[test]
        pub fn borrow_in_bit7() {
            assert!(borrow_in_bit(0, 1, 7));
        }

        #[test]
        pub fn borrow_in_bit15() {
            assert!(borrow_in_bit(0, 1, 15));
        }

        #[test]
        pub fn borrow_in_bits() {
            assert!(borrow_in_bit(0b0000_0101, 0b0000_0010, 1));
            assert!(borrow_in_bit(0b1111_0011, 0b0000_1100, 2));
            assert!(!borrow_in_bit(0b1111_0111, 0b1111_0000, 3));
            assert!(!borrow_in_bit(0b1111_0111, 0b1111_0111, 4));
            assert!(borrow_in_bit(0, 1, 8));
            assert!(borrow_in_bit(0b0000_1111_0000_1111, 0xFFFF, 9));
            assert!(!borrow_in_bit(0xFFFF, 0, 10));
            assert!(borrow_in_bit(0xF0F0, 0xFFFF, 11));
            assert!(borrow_in_bit(0x0FFF, 0xF000, 13));
            assert!(!borrow_in_bit(0xF000, 0xF000, 14));
        }
    }
}
