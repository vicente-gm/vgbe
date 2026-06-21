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

mod background;
mod lifecycle;
pub mod oam;
pub mod registers;
mod sprites;
pub mod tile;
mod tilemap;
pub mod timing;

use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::MemoryBus;
use registers::*;
use timing::*;

#[derive(Debug)]
pub struct Ppu {
    memory: Rc<RefCell<MemoryBus>>,
    dots: u16,
    frame_ready: bool,
    mode: PpuMode,
    ly: u8,
    window_line: u8,
    lcd_enabled: bool,
    stat_interrupt_line: bool,
    hblank_start_dot: u16,
    framebuffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    bg_color_ids: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PpuMode {
    HBlank = STAT_MODE_HBLANK,
    VBlank = STAT_MODE_VBLANK,
    OamScan = STAT_MODE_OAM,
    Drawing = STAT_MODE_DRAWING,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgTileDataMode {
    CurrentLcdc,
    Unsigned8000,
    Signed8800,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPaletteMode {
    RawColorId,
    ApplyBgp,
}

impl Ppu {
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Ppu {
        let lcd_enabled = mem.borrow().read_byte(LCDC_ADDR) & LCDC_LCD_ENABLE != 0;
        let initial_mode = if lcd_enabled {
            PpuMode::OamScan
        } else {
            PpuMode::HBlank
        };

        let mut ppu = Ppu {
            memory: Rc::clone(&mem),
            dots: 0,
            frame_ready: false,
            mode: initial_mode,
            ly: 0,
            lcd_enabled,
            window_line: 0,
            stat_interrupt_line: false,
            hblank_start_dot: DOT_ENTER_HBLANK,
            framebuffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            bg_color_ids: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
        };

        // Initialize STAT
        {
            let mut mem = ppu.memory.borrow_mut();
            let lyc_eq_ly = mem.read_byte(LYC_ADDR) == 0;
            mem.set_ly(0);
            mem.set_stat_mode_from_ppu(initial_mode as u8);
            mem.set_stat_lyc_eq_ly_from_ppu(lyc_eq_ly);
        }

        ppu.update_stat_interrupt_line();

        ppu
    }

    pub fn dots(&self) -> u16 {
        self.dots
    }

    pub fn tick(&mut self) {
        let lcd_enabled_now = self.memory.borrow().read_byte(LCDC_ADDR) & LCDC_LCD_ENABLE != 0;

        if !lcd_enabled_now {
            if self.lcd_enabled || self.dots != 0 || self.ly != 0 || self.mode != PpuMode::HBlank {
                self.enter_lcd_disabled_state();
            }

            self.lcd_enabled = false;
            return;
        }

        if !self.lcd_enabled {
            self.lcd_enabled = true;
            self.enter_lcd_enabled_state();
        }

        self.dots += 1;
        self.update_stat_interrupt_line();


        match self.dots {
            DOT_ENTER_DRAWING => {
                if self.ly < VISIBLE_SCANLINES {
                    self.hblank_start_dot = self.calculate_hblank_start_dot();
                    self.set_mode(PpuMode::Drawing);
                }
            }

            dot if dot == self.hblank_start_dot => {
                if self.ly < VISIBLE_SCANLINES {
                    self.render_scanline();
                    self.set_mode(PpuMode::HBlank);
                }
            }

            DOT_END_SCANLINE => {
                self.dots = 0;
                self.increment_ly();

                if self.ly >= VISIBLE_SCANLINES {
                    self.set_mode(PpuMode::VBlank);
                } else {
                    self.set_mode(PpuMode::OamScan);
                }
            }

            _ => {}
        }
    }
}
