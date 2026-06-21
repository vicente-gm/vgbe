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

use crate::ppu::registers::*;

use super::tile::*;
use super::{BgTileDataMode, DebugPaletteMode, Ppu};

impl Ppu {
    pub fn read_tilemap_0(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_0,
            BgTileDataMode::CurrentLcdc,
            DebugPaletteMode::ApplyBgp,
        )
    }

    pub fn read_tilemap_1(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_1,
            BgTileDataMode::CurrentLcdc,
            DebugPaletteMode::ApplyBgp,
        )
    }

    pub fn read_tilemap_0_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_0,
            BgTileDataMode::CurrentLcdc,
            DebugPaletteMode::RawColorId,
        )
    }

    pub fn read_tilemap_1_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_1,
            BgTileDataMode::CurrentLcdc,
            DebugPaletteMode::RawColorId,
        )
    }

    pub fn read_tilemap_0_unsigned_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_0,
            BgTileDataMode::Unsigned8000,
            DebugPaletteMode::RawColorId,
        )
    }

    pub fn read_tilemap_1_unsigned_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_1,
            BgTileDataMode::Unsigned8000,
            DebugPaletteMode::RawColorId,
        )
    }

    pub fn read_tilemap_0_signed_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_0,
            BgTileDataMode::Signed8800,
            DebugPaletteMode::RawColorId,
        )
    }

    pub fn read_tilemap_1_signed_raw(&self) -> TileMapPixels {
        self.read_tilemap_debug(
            BG_TILEMAP_AREA_1,
            BgTileDataMode::Signed8800,
            DebugPaletteMode::RawColorId,
        )
    }

    fn read_tilemap_debug(
        &self,
        tilemap_base: u16,
        tile_data_mode: BgTileDataMode,
        palette_mode: DebugPaletteMode,
    ) -> TileMapPixels {
        let lcdc = self.memory.borrow().read_byte(LCDC_ADDR);
        let bgp = self.memory.borrow().read_byte(BGP_ADDR);
        let resolved_tile_data_mode = match tile_data_mode {
            BgTileDataMode::CurrentLcdc => Self::bg_tile_data_mode_from_lcdc(lcdc),
            mode => mode,
        };
        let mut tilemap = [[0u8; TILEMAP_PIXEL_WIDTH]; TILEMAP_PIXEL_HEIGHT];

        for tile_y in 0..TILEMAP_TILES_PER_ROW {
            for tile_x in 0..TILEMAP_TILES_PER_ROW {
                let tile_index_addr =
                    tilemap_base + (tile_y * TILEMAP_TILES_PER_ROW + tile_x) as u16;
                let tile_index = self.memory.borrow().read_byte(tile_index_addr);

                for row in 0..8usize {
                    for col in 0..8usize {
                        let color_id = self.read_tile_pixel_color_with_mode(
                            resolved_tile_data_mode,
                            tile_index,
                            col as u8,
                            row as u8,
                        );

                        tilemap[tile_y * 8 + row][tile_x * 8 + col] = match palette_mode {
                            DebugPaletteMode::RawColorId => color_id,
                            DebugPaletteMode::ApplyBgp => Self::apply_palette(bgp, color_id),
                        };
                    }
                }
            }
        }

        tilemap
    }

    pub fn debug_tilemap_mode_label(&self) -> String {
        let mem = self.memory.borrow();
        let lcdc = mem.read_byte(LCDC_ADDR);
        let bgp = mem.read_byte(BGP_ADDR);
        let bg_map = if lcdc & LCDC_BG_TILEMAP_AREA != 0 { "0x9C00" } else { "0x9800" };
        let win_map = if lcdc & LCDC_WINDOW_TILEMAP_AREA != 0 { "0x9C00" } else { "0x9800" };
        let tile_data = if lcdc & LCDC_BG_WINDOW_TILE_DATA_AREA != 0 {
            "0x8000 unsign"
        } else {
            "0x8800 signed"
        };

        format!(
            "LCDC:{:02X} BGP:{:02X} BG:{} WIN:{} TD:{}",
            lcdc, bgp, bg_map, win_map, tile_data
        )
    }
}
