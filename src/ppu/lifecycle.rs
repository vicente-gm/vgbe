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

use crate::cpu::interrupts::Interrupt;
use crate::ppu::registers::*;
use crate::ppu::timing::*;

use super::{Ppu, PpuMode};

impl Ppu {
    #[inline]
    pub(super) fn enter_lcd_disabled_state(&mut self) {
        self.dots = 0;
        self.ly = 0;
        self.window_line = 0;
        self.frame_ready = false;
        self.mode = PpuMode::HBlank;
        self.stat_interrupt_line = false;

        let mut mem = self.memory.borrow_mut();

        mem.set_ly(0);
        mem.set_stat_mode_from_ppu(STAT_MODE_HBLANK);

        let lyc_eq_ly = mem.read_byte(LYC_ADDR) == 0;
        mem.set_stat_lyc_eq_ly_from_ppu(lyc_eq_ly);

        self.framebuffer = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.bg_color_ids = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }

    #[inline]
    pub(super) fn enter_lcd_enabled_state(&mut self) {
        self.dots = 0;
        self.ly = 0;
        self.window_line = 0;
        self.frame_ready = false;
        self.mode = PpuMode::OamScan;
        self.stat_interrupt_line = false;

        let mut mem = self.memory.borrow_mut();

        mem.set_ly(0);
        mem.set_stat_mode_from_ppu(STAT_MODE_OAM);

        let lyc_eq_ly = mem.read_byte(LYC_ADDR) == 0;
        mem.set_stat_lyc_eq_ly_from_ppu(lyc_eq_ly);

        drop(mem);
        self.update_stat_interrupt_line();
    }

    #[inline]
    pub(super) fn calculate_hblank_start_dot(&self) -> u16 {
        let lcdc = self.memory.borrow().read_byte(LCDC_ADDR);
        let mut extra = 0u16;

        let scx = self.memory.borrow().read_byte(SCX_ADDR);
        extra += (scx & 0x07) as u16;

        if self.window_visible_on_current_scanline(lcdc) {
            extra += 6;
        }

        if lcdc & LCDC_OBJ_ENABLE != 0 && self.ly < VISIBLE_SCANLINES {
            let sprite_height = if lcdc & LCDC_OBJ_SIZE != 0 { 16 } else { 8 };
            let sprite_count = self.select_objects_for_scanline(self.ly, sprite_height).len() as u16;

            extra += sprite_count * 6;
        }

        (DOT_ENTER_HBLANK + extra).min(DOT_END_SCANLINE - 1)
    }

    #[inline]
    fn window_visible_on_current_scanline(&self, lcdc: u8) -> bool {
        if lcdc & LCDC_BG_WINDOW_ENABLE == 0 || lcdc & LCDC_WINDOW_ENABLE == 0 {
            return false;
        }

        let wy = self.memory.borrow().read_byte(WY_ADDR);
        let wx = self.memory.borrow().read_byte(WX_ADDR);
        let window_start_x = wx as i16 - 7;

        self.ly >= wy && window_start_x < SCREEN_WIDTH as i16
    }

    #[inline]
    pub(super) fn increment_ly(&mut self) {
        self.ly = if self.ly >= LAST_SCANLINE {
            self.window_line = 0;
            0
        } else {
            self.ly + 1
        };

        let mut mem = self.memory.borrow_mut();
        let lyc_eq_ly = mem.read_byte(LYC_ADDR) == self.ly;

        mem.set_ly(self.ly);
        mem.set_stat_lyc_eq_ly_from_ppu(lyc_eq_ly);

        if self.ly == VISIBLE_SCANLINES {
            self.frame_ready = true;
            mem.request_interrupt(Interrupt::VBLANK);
        }

        drop(mem);
        self.update_stat_interrupt_line();
    }

    #[inline]
    pub fn take_frame_ready(&mut self) -> bool {
        if self.frame_ready {
            self.frame_ready = false;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub(super) fn set_mode(&mut self, mode: PpuMode) {
        if self.mode == mode {
            return;
        }

        self.mode = mode;
        self.memory.borrow_mut().set_stat_mode_from_ppu(mode as u8);
        self.update_stat_interrupt_line();
    }

    pub(super) fn update_stat_interrupt_line(&mut self) {
        let mut mem = self.memory.borrow_mut();
        let stat = mem.read_byte(STAT_ADDR);
        let lcdc = mem.read_byte(LCDC_ADDR);
        let ly = mem.read_byte(LY_ADDR);
        let lyc = mem.read_byte(LYC_ADDR);
        let lyc_eq_ly = ly == lyc;

        mem.set_stat_lyc_eq_ly_from_ppu(lyc_eq_ly);

        let mode = if lcdc & LCDC_LCD_ENABLE == 0 {
            STAT_MODE_HBLANK
        } else {
            stat & STAT_MODE_MASK
        };

        let line = lcdc & LCDC_LCD_ENABLE != 0
            && ((lyc_eq_ly && stat & STAT_LYC_INTERRUPT_ENABLE != 0)
                || (mode == STAT_MODE_OAM && stat & STAT_OAM_INTERRUPT_ENABLE != 0)
                || (mode == STAT_MODE_VBLANK && stat & STAT_VBLANK_INTERRUPT_ENABLE != 0)
                || (mode == STAT_MODE_HBLANK && stat & STAT_HBLANK_INTERRUPT_ENABLE != 0));

        if line && !self.stat_interrupt_line {
            mem.request_interrupt(Interrupt::LCD);
        }

        self.stat_interrupt_line = line;
    }
}
