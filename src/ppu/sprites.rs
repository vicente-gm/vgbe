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

use super::oam::*;
use super::tile::TILE_SIZE;
use super::Ppu;

impl Ppu {
    pub(super) fn render_sprites(&mut self, lcdc: u8) {
        if lcdc & LCDC_OBJ_ENABLE == 0 {
            return;
        }

        let ly = self.ly;
        let sprite_height = if lcdc & LCDC_OBJ_SIZE != 0 { 16 } else { 8 };
        let mut objects = self.select_objects_for_scanline(ly, sprite_height);
        objects.sort_by_key(|object| (object.x, object.oam_index));

        let mut resolved_object_pixel = [false; SCREEN_WIDTH];

        for object in objects {
            self.render_object_pixels(lcdc, object, sprite_height, &mut resolved_object_pixel);
        }
    }

    pub(super) fn select_objects_for_scanline(&self, ly: u8, sprite_height: u8) -> Vec<Object> {
        let mut objects = Vec::with_capacity(MAX_OBJECTS_PER_SCANLINE);
        let ly = ly as i16;

        for oam_index in 0..NUM_OBJECTS {
            let oam_addr = OAM_BASE + oam_index as u16 * OAM_ENTRY_SIZE;
            let y = self.memory.borrow().read_byte(oam_addr);
            let sprite_y = y as i16 - 16;

            if ly < sprite_y || ly >= sprite_y + sprite_height as i16 {
                continue;
            }

            objects.push(Object {
                oam_index,
                y,
                x: self.memory.borrow().read_byte(oam_addr + 1),
                tile_index: self.memory.borrow().read_byte(oam_addr + 2),
                attrs: self.memory.borrow().read_byte(oam_addr + 3),
            });

            if objects.len() == MAX_OBJECTS_PER_SCANLINE {
                break;
            }
        }

        objects
    }

    fn render_object_pixels(
        &mut self,
        lcdc: u8,
        object: Object,
        sprite_height: u8,
        resolved_object_pixel: &mut [bool; SCREEN_WIDTH],
    ) {
        let sprite_y = object.y as i16 - 16;
        let sprite_x = object.x as i16 - 8;
        let y_flip = object.attrs & 0x40 != 0;
        let x_flip = object.attrs & 0x20 != 0;
        let behind_bg = object.attrs & 0x80 != 0;
        let palette_addr = if object.attrs & 0x10 != 0 { OBP1_ADDR } else { OBP0_ADDR };
        let palette = self.memory.borrow().read_byte(palette_addr);

        let mut tile_row = self.ly as i16 - sprite_y;
        if y_flip {
            tile_row = sprite_height as i16 - 1 - tile_row;
        }

        let tile_index = if sprite_height == 16 {
            (object.tile_index & 0xFE).wrapping_add((tile_row as u8) / 8)
        } else {
            object.tile_index
        };
        let row_in_tile = (tile_row as u8) % 8;
        let tile_addr = TILE_DATA_AREA_0 + tile_index as u16 * TILE_SIZE + row_in_tile as u16 * 2;
        let byte1 = self.memory.borrow().read_byte(tile_addr);
        let byte2 = self.memory.borrow().read_byte(tile_addr + 1);

        for sprite_col in 0..8u8 {
            let screen_x = sprite_x + sprite_col as i16;
            if screen_x < 0 || screen_x as usize >= SCREEN_WIDTH {
                continue;
            }

            let x = screen_x as usize;
            if resolved_object_pixel[x] {
                continue;
            }

            let tile_col = if x_flip { sprite_col } else { 7 - sprite_col };
            let color_id = ((byte2 >> tile_col) & 1) << 1 | ((byte1 >> tile_col) & 1);

            if color_id == 0 {
                continue;
            }

            resolved_object_pixel[x] = true;

            if behind_bg
                && lcdc & LCDC_BG_WINDOW_ENABLE != 0
                && self.bg_color_ids[self.ly as usize][x] != 0
            {
                continue;
            }

            self.framebuffer[self.ly as usize][x] = Self::apply_palette(palette, color_id);
        }
    }
}
