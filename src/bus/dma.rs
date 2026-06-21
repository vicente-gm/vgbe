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

use crate::ppu::registers::OAM_DMA_SIZE;

use super::*;

impl MemoryBus {
    #[inline]
    pub(super) fn do_oam_dma(&mut self, value: u8) {
        let source = (value as u16) << 8;

        for offset in 0..OAM_DMA_SIZE {
            let byte = self.read_byte(source + offset);
            self.oam[offset as usize] = byte;
        }
    }
}
