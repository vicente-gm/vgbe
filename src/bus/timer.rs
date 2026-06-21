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

use crate::cpu::interrupts::Interrupt;

use super::*;

impl MemoryBus {
    pub fn tick_timer(&mut self) {
        let old_signal = self.timer_signal();

        self.div_counter = self.div_counter.wrapping_add(1);
        self.io[(DIV_ADDR - 0xFF00) as usize] = (self.div_counter >> 6) as u8;

        self.increment_tima_on_falling_edge(old_signal);
    }

    pub(super) fn increment_tima_on_falling_edge(&mut self, old_signal: bool) {
        if old_signal && !self.timer_signal() {
            self.increment_tima();
        }
    }

    fn increment_tima(&mut self) {
        let tima_index = (TIMA_ADDR - 0xFF00) as usize;
        let (value, overflow) = self.io[tima_index].overflowing_add(1);

        if overflow {
            self.io[tima_index] = self.io[(TMA_ADDR - 0xFF00) as usize];
            self.request_interrupt(Interrupt::TIMER);
        } else {
            self.io[tima_index] = value;
        }
    }

    pub(super) fn timer_signal(&self) -> bool {
        let tac = self.io[(TAC_ADDR - 0xFF00) as usize];

        if tac & TAC_ENABLE == 0 {
            return false;
        }

        let bit = match tac & TAC_CLOCK_SELECT_MASK {
            0b00 => 7,
            0b01 => 1,
            0b10 => 3,
            0b11 => 5,
            _ => unreachable!(),
        };

        self.div_counter & (1 << bit) != 0
    }
}
