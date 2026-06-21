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

pub const OAM_BASE: u16 = 0xFE00;
pub const OAM_ENTRY_SIZE: u16 = 4;
pub const NUM_OBJECTS: u8 = (OAM_DMA_SIZE / OAM_ENTRY_SIZE) as u8;
pub const MAX_OBJECTS_PER_SCANLINE: usize = 10;

#[derive(Clone, Copy)]
pub struct Object {
    pub oam_index: u8,
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub attrs: u8,
}
