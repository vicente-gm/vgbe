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

use crate::ppu::Ppu;

pub struct Tile {
    pub data: [[u8; 8]; 8],
}

pub const TILE_DATA_START: u16 = 0x8000;
pub const TILE_DATA_END: u16 = 0x97FF;
pub const TILE_SIZE: u16 = 16;  // Bytes

pub const NUM_TILES: u16 = 384;

pub const TILEMAP_TILES_PER_ROW: usize = 32;
pub const TILEMAP_PIXEL_WIDTH: usize = TILEMAP_TILES_PER_ROW * 8;
pub const TILEMAP_PIXEL_HEIGHT: usize = TILEMAP_PIXEL_WIDTH;

pub type TileMapPixels = [[u8; TILEMAP_PIXEL_WIDTH]; TILEMAP_PIXEL_HEIGHT];

impl Ppu {
    pub fn read_tile(&self, tile_index: u16) -> Tile {
        let base: u16 = TILE_DATA_START + tile_index * TILE_SIZE;
        let mut tile = Tile {
            data: [[0u8; 8]; 8],
        };

        for row in 0..8usize {
            let byte1 = self.memory.borrow().read_byte(base + (row as u16) * 2);
            let byte2 = self.memory.borrow().read_byte(base + (row as u16) * 2 + 1);

            for col in 0..8usize {
                let bit = 7 - col;
                let lo = (byte1 >> bit) & 1;
                let hi = (byte2 >> bit) & 1;

                tile.data[row][col] = (hi << 1) | lo;
            }
        }

        tile
    }

    pub fn read_all_tiles(&self) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(NUM_TILES as usize);

        for i in 0..NUM_TILES {
            tiles.push(self.read_tile(i));
        }

        tiles
    }
}
