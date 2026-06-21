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
pub enum Interrupt {
    VBLANK, LCD, TIMER, SERIAL, JOYPAD
}

/// Indicates the memory direction of the Interrupt Enable register.
pub const INTERRUPT_ENABLE_DIR: u16 = 0xFFFF;

/// Indicates the memory direction of the Interrupt Flag register.
pub const INTERRUPT_FLAG_DIR: u16 = 0xFF0F;

/// Indicates the bit position of every interrupt type inside the registers.
pub const INTERRUPT_VBLANK_BIT: u8 = 0;
pub const INTERRUPT_LCD_BIT: u8 = 1;
pub const INTERRUPT_TIMER_BIT: u8 = 2;
pub const INTERRUPT_SERIAL_BIT: u8 = 3;
pub const INTERRUPT_JOYPAD_BIT: u8 = 4;

/// Interrupt priority table.
///
/// Each entry contains:
/// - Interrupt identifier
/// - Bit position inside IE/IF registers
/// - Interrupt vector address
pub const INTERRUPTS_TABLE: &[(Interrupt, u8, u16)] = &[
    (Interrupt::VBLANK, INTERRUPT_VBLANK_BIT, 0x40),
    (Interrupt::LCD,    INTERRUPT_LCD_BIT,    0x48),
    (Interrupt::TIMER,  INTERRUPT_TIMER_BIT,  0x50),
    (Interrupt::SERIAL, INTERRUPT_SERIAL_BIT, 0x58),
    (Interrupt::JOYPAD, INTERRUPT_JOYPAD_BIT, 0x60),
];

impl Interrupt {
    #[inline]
    pub const fn bit(self) -> u8 {
        match self {
            Interrupt::VBLANK => INTERRUPT_VBLANK_BIT,
            Interrupt::LCD => INTERRUPT_LCD_BIT,
            Interrupt::TIMER => INTERRUPT_TIMER_BIT,
            Interrupt::SERIAL => INTERRUPT_SERIAL_BIT,
            Interrupt::JOYPAD => INTERRUPT_JOYPAD_BIT,
        }
    }
}