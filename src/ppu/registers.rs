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

//! LCD/PPU memory-mapped register constants.

pub const LCDC_ADDR: u16 = 0xFF40;
pub const STAT_ADDR: u16 = 0xFF41;
pub const SCY_ADDR: u16 = 0xFF42;
pub const SCX_ADDR: u16 = 0xFF43;
pub const LY_ADDR: u16 = 0xFF44;
pub const LYC_ADDR: u16 = 0xFF45;
pub const DMA_ADDR: u16 = 0xFF46;
pub const BGP_ADDR: u16 = 0xFF47;
pub const OBP0_ADDR: u16 = 0xFF48;
pub const OBP1_ADDR: u16 = 0xFF49;
pub const WY_ADDR: u16 = 0xFF4A;
pub const WX_ADDR: u16 = 0xFF4B;

// STAT mode bits.
pub const STAT_MODE_MASK: u8 = 0b0000_0011;

pub const STAT_MODE_HBLANK: u8 = 0b00;
pub const STAT_MODE_VBLANK: u8 = 0b01;
pub const STAT_MODE_OAM: u8 = 0b10;
pub const STAT_MODE_DRAWING: u8 = 0b11;

// STAT interrupt enable bits.
pub const STAT_LYC_EQ_LY_FLAG: u8 = 0b0000_0100;
pub const STAT_HBLANK_INTERRUPT_ENABLE: u8 = 0b0000_1000;
pub const STAT_VBLANK_INTERRUPT_ENABLE: u8 = 0b0001_0000;
pub const STAT_OAM_INTERRUPT_ENABLE: u8 = 0b0010_0000;
pub const STAT_LYC_INTERRUPT_ENABLE: u8 = 0b0100_0000;

// Bit 7 is always read as 1 on real hardware in many references.
// You can use it later if you want to normalize STAT reads/writes.
pub const STAT_UNUSED_BIT: u8 = 0b1000_0000;

// LCDC bits.
pub const LCDC_BG_WINDOW_ENABLE: u8 = 0b0000_0001;
pub const LCDC_OBJ_ENABLE: u8 = 0b0000_0010;
pub const LCDC_OBJ_SIZE: u8 = 0b0000_0100;
pub const LCDC_BG_TILEMAP_AREA: u8 = 0b0000_1000;
pub const LCDC_BG_WINDOW_TILE_DATA_AREA: u8 = 0b0001_0000;
pub const LCDC_WINDOW_ENABLE: u8 = 0b0010_0000;
pub const LCDC_WINDOW_TILEMAP_AREA: u8 = 0b0100_0000;
pub const LCDC_LCD_ENABLE: u8 = 0b1000_0000;

// Tile data / tile maps.
pub const TILE_DATA_AREA_0: u16 = 0x8000;
pub const TILE_DATA_AREA_1: u16 = 0x8800;
pub const TILE_DATA_SIGNED_BASE: u16 = 0x9000;

pub const BG_TILEMAP_AREA_0: u16 = 0x9800;
pub const BG_TILEMAP_AREA_1: u16 = 0x9C00;

// OAM DMA.
pub const OAM_DMA_SIZE: u16 = 0xA0;