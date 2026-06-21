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

//! PPU timing constants.

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const DOTS_PER_SCANLINE: u16 = 456;
pub const VISIBLE_SCANLINES: u8 = 144;
pub const TOTAL_SCANLINES: u8 = 154;
pub const LAST_SCANLINE: u8 = TOTAL_SCANLINES - 1;

// Mode transition dots inside a visible scanline.
pub const OAM_SCAN_DOTS: u16 = 80;
pub const DRAWING_END_DOT: u16 = 252;

// More explicit aliases for match arms.
pub const DOT_ENTER_DRAWING: u16 = OAM_SCAN_DOTS;
pub const DOT_ENTER_HBLANK: u16 = DRAWING_END_DOT;
pub const DOT_END_SCANLINE: u16 = DOTS_PER_SCANLINE;

// Useful derived value.
pub const DOTS_PER_FRAME: usize = DOTS_PER_SCANLINE as usize * TOTAL_SCANLINES as usize;