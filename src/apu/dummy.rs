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

use super::Apu;

const APU_START: u16 = 0xFF10;
const APU_END: u16 = 0xFF3F;

#[derive(Debug)]
pub struct DummyApu {
    registers: [u8; (APU_END - APU_START + 1) as usize],
}

impl Default for DummyApu {
    fn default() -> Self {
        Self {
            registers: [0; (APU_END - APU_START + 1) as usize],
        }
    }
}

impl DummyApu {
    fn register_index(address: u16) -> Option<usize> {
        (APU_START..=APU_END)
            .contains(&address)
            .then_some((address - APU_START) as usize)
    }
}

impl Apu for DummyApu {
    fn reset(&mut self) {
        self.registers.fill(0);
    }

    fn step(&mut self, _cycles: u32) {}

    fn end_frame(&mut self) {}

    fn read_register(&mut self, address: u16) -> u8 {
        Self::register_index(address)
            .map(|index| self.registers[index])
            .unwrap_or(0xFF)
    }

    fn write_register(&mut self, address: u16, value: u8) {
        if let Some(index) = Self::register_index(address) {
            self.registers[index] = value;
        }
    }

    fn drain_samples(&mut self, _output: &mut Vec<i16>) {}
}
