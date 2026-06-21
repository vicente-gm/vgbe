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
use crate::ppu::timing::*;

use super::{BgTileDataMode, Ppu};
use super::tile::*;

impl Ppu {
    pub fn get_framebuffer(&self) -> &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.framebuffer
    }

    pub(super) fn render_scanline(&mut self) {
        let lcdc = self.memory.borrow().read_byte(LCDC_ADDR);

        if lcdc & LCDC_LCD_ENABLE == 0 {
            self.framebuffer[self.ly as usize] = [0; SCREEN_WIDTH];
            self.bg_color_ids[self.ly as usize] = [0; SCREEN_WIDTH];
            return;
        }

        self.render_background(lcdc);
        self.render_sprites(lcdc);
    }

    fn render_background(&mut self, lcdc: u8) {
        let ly = self.ly;
        let bg_enabled = lcdc & LCDC_BG_WINDOW_ENABLE != 0;
        let window_enabled = lcdc & LCDC_WINDOW_ENABLE != 0;
        let scy = self.memory.borrow().read_byte(SCY_ADDR);
        let scx = self.memory.borrow().read_byte(SCX_ADDR);
        let wy = self.memory.borrow().read_byte(WY_ADDR);
        let wx = self.memory.borrow().read_byte(WX_ADDR);
        let bgp = self.memory.borrow().read_byte(BGP_ADDR);
        let mut window_used_this_scanline = false;

        for x in 0..SCREEN_WIDTH {
            if !bg_enabled {
                self.bg_color_ids[ly as usize][x] = 0;
                self.framebuffer[ly as usize][x] = 0;
                continue;
            }

            let screen_x_i = x as i16;
            let window_start_x = wx as i16 - 7;
            let use_window = window_enabled && ly >= wy && screen_x_i >= window_start_x;

            let (tilemap_base, pixel_x, pixel_y) = if use_window {
                window_used_this_scanline = true;

                let window_x = (screen_x_i - window_start_x) as u8;
                let window_y = self.window_line;
                let base = if lcdc & LCDC_WINDOW_TILEMAP_AREA != 0 {
                    BG_TILEMAP_AREA_1
                } else {
                    BG_TILEMAP_AREA_0
                };

                (base, window_x, window_y)
            } else {
                let base = if lcdc & LCDC_BG_TILEMAP_AREA != 0 {
                    BG_TILEMAP_AREA_1
                } else {
                    BG_TILEMAP_AREA_0
                };

                (base, scx.wrapping_add(x as u8), scy.wrapping_add(ly))
            };

            let color_id = self.read_bg_color(lcdc, tilemap_base, pixel_x, pixel_y);

            self.bg_color_ids[ly as usize][x] = color_id;
            self.framebuffer[ly as usize][x] = Self::apply_palette(bgp, color_id);
        }

        if window_used_this_scanline {
            self.window_line = self.window_line.wrapping_add(1);
        }
    }

    fn read_bg_color(&self, lcdc: u8, tilemap_base: u16, pixel_x: u8, pixel_y: u8) -> u8 {
        let tile_x = (pixel_x / 8) as u16;
        let tile_y = (pixel_y / 8) as u16;
        let tile_map_addr = tilemap_base + tile_y * 32 + tile_x;
        let tile_index = self.memory.borrow().read_byte(tile_map_addr);

        self.read_tile_pixel_color(lcdc, tile_index, pixel_x % 8, pixel_y % 8)
    }

    pub(super) fn read_tile_pixel_color(&self, lcdc: u8, tile_index: u8, col: u8, row: u8) -> u8 {
        let mode = Self::bg_tile_data_mode_from_lcdc(lcdc);
        self.read_tile_pixel_color_with_mode(mode, tile_index, col, row)
    }

    pub(super) fn read_tile_pixel_color_with_mode(
        &self,
        mode: BgTileDataMode,
        tile_index: u8,
        col: u8,
        row: u8,
    ) -> u8 {
        let tile_addr = match mode {
            BgTileDataMode::CurrentLcdc => {
                let lcdc = self.memory.borrow().read_byte(LCDC_ADDR);
                if lcdc & LCDC_BG_WINDOW_TILE_DATA_AREA != 0 {
                    TILE_DATA_AREA_0 + tile_index as u16 * TILE_SIZE
                } else {
                    let signed_index = tile_index as i8 as i16;
                    (TILE_DATA_SIGNED_BASE as i32 + signed_index as i32 * TILE_SIZE as i32) as u16
                }
            }

            BgTileDataMode::Unsigned8000 => TILE_DATA_AREA_0 + tile_index as u16 * TILE_SIZE,

            BgTileDataMode::Signed8800 => {
                let signed_index = tile_index as i8 as i16;
                (TILE_DATA_SIGNED_BASE as i32 + signed_index as i32 * TILE_SIZE as i32) as u16
            }
        };

        let byte1 = self.memory.borrow().read_byte(tile_addr + row as u16 * 2);
        let byte2 = self.memory.borrow().read_byte(tile_addr + row as u16 * 2 + 1);
        let bit = 7 - col;

        ((byte2 >> bit) & 1) << 1 | ((byte1 >> bit) & 1)
    }

    #[inline(always)]
    pub(super) fn apply_palette(palette: u8, color_id: u8) -> u8 {
        (palette >> (color_id * 2)) & 0b11
    }

    #[inline(always)]
    pub(super) fn bg_tile_data_mode_from_lcdc(lcdc: u8) -> BgTileDataMode {
        if lcdc & LCDC_BG_WINDOW_TILE_DATA_AREA != 0 {
            BgTileDataMode::Unsigned8000
        } else {
            BgTileDataMode::Signed8800
        }
    }
}
