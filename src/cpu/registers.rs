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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg16 {
    AF, BC, DE, HL, SP, PC
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg8 {
    A, F, B, C, D, E, H, L
}

pub const FLAG_Z: u8 = 0b1000_0000; // Zero
pub const FLAG_N: u8 = 0b0100_0000; // Subtract
pub const FLAG_H: u8 = 0b0010_0000; // Half-Carry
pub const FLAG_C: u8 = 0b0001_0000; // Carry

// Bit position of every flag in register F
pub const FLAG_Z_SHIFT: u8 = 7;
pub const FLAG_N_SHIFT: u8 = 6;
pub const FLAG_H_SHIFT: u8 = 5;
pub const FLAG_C_SHIFT: u8 = 4;