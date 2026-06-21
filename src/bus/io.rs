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

use crate::cpu::interrupts::{INTERRUPT_FLAG_DIR, Interrupt};
use crate::ppu::registers::*;

use super::*;

const JOYP_ADDR: u16 = 0xFF00;
const JOYP_SELECT_MASK: u8 = 0b0011_0000;
const APU_START_ADDR: u16 = 0xFF10;
const APU_END_ADDR: u16 = 0xFF3F;

impl MemoryBus {
    #[inline]
    pub(super) fn read_io(&self, address: u16) -> u8 {
        match address {
            JOYP_ADDR => self.read_joypad(),

            APU_START_ADDR..=APU_END_ADDR => self.apu.borrow_mut().read_register(address),

            STAT_ADDR => {
                let value = self.io[(address - 0xFF00) as usize] | STAT_UNUSED_BIT;

                if self.io[(LCDC_ADDR - 0xFF00) as usize] & LCDC_LCD_ENABLE == 0 {
                    value & !STAT_MODE_MASK
                } else {
                    value
                }
            }

            0xFF50 => {
                if self.boot_rom_enabled {
                    0x00
                } else {
                    0x01
                }
            }

            _ => self.io[(address - 0xFF00) as usize],
        }
    }

    #[inline]
    pub(super) fn write_io(&mut self, address: u16, value: u8) {
        match address {
            JOYP_ADDR => {
                self.io[(JOYP_ADDR - 0xFF00) as usize] = 0xC0 | (value & JOYP_SELECT_MASK) | 0x0F;
            }

            DIV_ADDR => {
                let old_signal = self.timer_signal();
                self.div_counter = 0;
                self.io[(DIV_ADDR - 0xFF00) as usize] = 0;
                self.increment_tima_on_falling_edge(old_signal);
            }

            TIMA_ADDR => {
                self.io[(TIMA_ADDR - 0xFF00) as usize] = value;
            }

            TMA_ADDR => {
                self.io[(TMA_ADDR - 0xFF00) as usize] = value;
            }

            TAC_ADDR => {
                let old_signal = self.timer_signal();
                self.io[(TAC_ADDR - 0xFF00) as usize] =
                    value & (TAC_ENABLE | TAC_CLOCK_SELECT_MASK);
                self.increment_tima_on_falling_edge(old_signal);
            }

            APU_START_ADDR..=APU_END_ADDR => {
                self.apu.borrow_mut().write_register(address, value);
            }

            0xFF50 => {
                self.boot_rom_enabled = false;
                self.io[(address - 0xFF00) as usize] = value;
            }

            DMA_ADDR => {
                self.io[(address - 0xFF00) as usize] = value;
                self.do_oam_dma(value);
            }

            STAT_ADDR => {
                let index = (address - 0xFF00) as usize;
                let read_only_bits = self.io[index] & (STAT_MODE_MASK | STAT_LYC_EQ_LY_FLAG);

                self.io[index] = (value & !(STAT_MODE_MASK | STAT_LYC_EQ_LY_FLAG)) | read_only_bits;
            }

            LY_ADDR => {}

            _ => {
                self.io[(address - 0xFF00) as usize] = value;
            }
        }
    }

    #[inline]
    pub fn set_ly(&mut self, value: u8) {
        self.io[(LY_ADDR - 0xFF00) as usize] = value;
    }

    #[inline]
    pub fn set_stat_mode_from_ppu(&mut self, mode: u8) {
        let index = (STAT_ADDR - 0xFF00) as usize;
        self.io[index] = (self.io[index] & !STAT_MODE_MASK) | (mode & STAT_MODE_MASK);
    }

    #[inline]
    pub fn set_stat_lyc_eq_ly_from_ppu(&mut self, value: bool) {
        let index = (STAT_ADDR - 0xFF00) as usize;

        if value {
            self.io[index] |= STAT_LYC_EQ_LY_FLAG;
        } else {
            self.io[index] &= !STAT_LYC_EQ_LY_FLAG;
        }
    }

    #[inline]
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        let iflag = self.read_byte(INTERRUPT_FLAG_DIR);
        self.write_byte(INTERRUPT_FLAG_DIR, iflag | (1 << interrupt.bit()));
    }

    #[inline]
    pub fn clear_interrupt_request(&mut self, interrupt: Interrupt) {
        let iflag = self.read_byte(INTERRUPT_FLAG_DIR);
        self.write_byte(INTERRUPT_FLAG_DIR, iflag & !(1 << interrupt.bit()));
    }

    #[inline]
    pub fn is_interrupt_requested(&self, interrupt: Interrupt) -> bool {
        let iflag = self.read_byte(INTERRUPT_FLAG_DIR);
        iflag & (1 << interrupt.bit()) != 0
    }

    fn read_joypad(&self) -> u8 {
        let select = self.io[(JOYP_ADDR - 0xFF00) as usize] & JOYP_SELECT_MASK;
        let mut low_nibble = 0x0F;

        if select & 0b0001_0000 == 0 {
            low_nibble &= self.tas_input.direction_bits();
        }

        if select & 0b0010_0000 == 0 {
            low_nibble &= self.tas_input.button_bits();
        }

        0xC0 | select | low_nibble
    }
}
