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

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{Font, FontStyle, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};

use crate::cpu::Cpu;
use crate::frontend::panels::disassembly::draw_disassembly_panel;
use crate::frontend::text::draw_text;
use crate::frontend::window::Window as UiWindow;
use crate::ppu::tile::TileMapPixels;
use crate::ppu::timing::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ppu::Ppu;

pub struct Renderer {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    ttf_context: Sdl2TtfContext,
}

impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        let texture_creator = canvas.texture_creator();
        let ttf_context = sdl2::ttf::init().unwrap();

        Self {
            canvas,
            texture_creator,
            ttf_context,
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.canvas.window_mut().set_size(width, height).unwrap();
    }

    fn load_font(ttf_context: &Sdl2TtfContext) -> Font<'_, '_> {
        ttf_context
            .load_font("assets/fonts/DejaVuSansMono.ttf", 16)
            .unwrap()
    }

    fn load_bold_font(ttf_context: &Sdl2TtfContext) -> Font<'_, '_> {
        let mut font = ttf_context
            .load_font("assets/fonts/DejaVuSansMono.ttf", 16)
            .unwrap();

        font.set_style(FontStyle::BOLD);
        font
    }

    pub fn render(
        &mut self,
        cpu: &Cpu,
        ppu: &Ppu,
        debug: bool,
        fps: u32,
        cycle: usize,
        redraw_screen: bool,
    ) {
        let font = Self::load_font(&self.ttf_context);
        let bold_font = Self::load_bold_font(&self.ttf_context);

        if redraw_screen {
            self.canvas.set_draw_color(Color::RGB(25, 25, 25));
            self.canvas.clear();
        }

        Self::draw_status_bar(
            &mut self.canvas,
            &self.texture_creator,
            &font,
            fps,
            UiWindow::window_width(debug),
            UiWindow::content_height(debug),
        );

        if debug {
            Self::draw_debug_layout(
                &mut self.canvas,
                &self.texture_creator,
                cpu,
                ppu,
                ppu.get_framebuffer(),
                &font,
                &bold_font,
                cycle,
                redraw_screen,
            );
        } else {
            Self::draw_normal_layout(&mut self.canvas, ppu.get_framebuffer(), redraw_screen);
        }

        self.canvas.present();
    }

    pub fn render_framebuffer_with_state(
        &mut self,
        cpu: &Cpu,
        ppu: &Ppu,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        debug: bool,
        fps: u32,
    ) {
        let font = Self::load_font(&self.ttf_context);
        let bold_font = Self::load_bold_font(&self.ttf_context);

        self.canvas.set_draw_color(Color::RGB(25, 25, 25));
        self.canvas.clear();

        Self::draw_status_bar(
            &mut self.canvas,
            &self.texture_creator,
            &font,
            fps,
            UiWindow::window_width(debug),
            UiWindow::content_height(debug),
        );

        if debug {
            Self::draw_debug_layout(
                &mut self.canvas,
                &self.texture_creator,
                cpu,
                ppu,
                framebuffer,
                &font,
                &bold_font,
                0,
                true,
            );
        } else {
            Self::draw_normal_layout(&mut self.canvas, framebuffer, true);
        }

        self.canvas.present();
    }

    pub fn render_framebuffer(
        &mut self,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        debug: bool,
        fps: u32,
    ) {
        let font = Self::load_font(&self.ttf_context);

        self.canvas.set_draw_color(Color::RGB(25, 25, 25));
        self.canvas.clear();

        Self::draw_status_bar(
            &mut self.canvas,
            &self.texture_creator,
            &font,
            fps,
            UiWindow::window_width(debug),
            UiWindow::content_height(debug),
        );

        if debug {
            let bold_font = Self::load_bold_font(&self.ttf_context);
            Self::draw_screen_panel(
                &mut self.canvas,
                &self.texture_creator,
                &bold_font,
                framebuffer,
                UiWindow::DEBUG_LEFT_COLUMN_X,
                UiWindow::DEBUG_TOP_Y,
                UiWindow::DEBUG_LEFT_COLUMN_WIDTH,
                UiWindow::DEBUG_SCREEN_PANEL_HEIGHT,
            );
        } else {
            Self::draw_screen_framebuffer(
                &mut self.canvas,
                framebuffer,
                0,
                0,
                UiWindow::SCALE_FACTOR,
            );
        }

        self.canvas.present();
    }

    fn draw_status_bar(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        fps: u32,
        window_width: u32,
        content_height: u32,
    ) {
        let y = content_height as i32;
        let rect = Rect::new(0, y, window_width, UiWindow::STATUS_HEIGHT);

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        let _ = canvas.fill_rect(rect);

        let status = format!("FPS: {}", fps);
        draw_text(canvas, texture_creator, font, &status, 10, y + 2);
    }

    fn draw_normal_layout(
        canvas: &mut Canvas<Window>,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        redraw_screen: bool,
    ) {
        if !redraw_screen {
            return;
        }

        Self::draw_screen_framebuffer(
            canvas,
            framebuffer,
            0,
            0,
            UiWindow::SCALE_FACTOR,
        );
    }

    fn draw_debug_layout(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        cpu: &Cpu,
        ppu: &Ppu,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        font: &Font,
        bold_font: &Font,
        cycle: usize,
        redraw_screen: bool,
    ) {
        if redraw_screen {
            Self::draw_screen_panel(
                canvas,
                texture_creator,
                bold_font,
                framebuffer,
                UiWindow::DEBUG_LEFT_COLUMN_X,
                UiWindow::DEBUG_TOP_Y,
                UiWindow::DEBUG_LEFT_COLUMN_WIDTH,
                UiWindow::DEBUG_SCREEN_PANEL_HEIGHT,
            );
        }

        Self::draw_cpu_panel(
            canvas,
            texture_creator,
            font,
            bold_font,
            cpu,
            ppu,
            cycle,
            UiWindow::DEBUG_CENTER_COLUMN_X,
            UiWindow::DEBUG_TOP_Y,
            UiWindow::DEBUG_CENTER_COLUMN_WIDTH,
            UiWindow::DEBUG_CPU_PANEL_HEIGHT,
        );

        let disassembly_y = UiWindow::DEBUG_TOP_Y
            + UiWindow::DEBUG_CPU_PANEL_HEIGHT as i32
            + UiWindow::DEBUG_PANEL_GAP as i32;

        draw_disassembly_panel(
            canvas,
            texture_creator,
            font,
            bold_font,
            cpu,
            UiWindow::DEBUG_CENTER_COLUMN_X,
            disassembly_y,
            UiWindow::DEBUG_CENTER_COLUMN_WIDTH,
            UiWindow::DEBUG_DISASSEMBLY_PANEL_HEIGHT,
        );

        let buttons_y = UiWindow::DEBUG_CONTENT_HEIGHT as i32
            - UiWindow::DEBUG_SIDE_MARGIN as i32
            - UiWindow::DEBUG_BUTTON_HEIGHT as i32;
        let button_width =
            (UiWindow::DEBUG_CENTER_COLUMN_WIDTH - UiWindow::DEBUG_BUTTON_GAP * 2) / 3;

        Self::draw_debug_button(
            canvas,
            texture_creator,
            font,
            "Pause",
            UiWindow::DEBUG_CENTER_COLUMN_X,
            buttons_y,
            button_width,
            UiWindow::DEBUG_BUTTON_HEIGHT,
        );

        Self::draw_debug_button(
            canvas,
            texture_creator,
            font,
            "Continue",
            UiWindow::DEBUG_CENTER_COLUMN_X
                + (button_width + UiWindow::DEBUG_BUTTON_GAP) as i32,
            buttons_y,
            button_width,
            UiWindow::DEBUG_BUTTON_HEIGHT,
        );

        Self::draw_debug_button(
            canvas,
            texture_creator,
            font,
            "Next",
            UiWindow::DEBUG_CENTER_COLUMN_X
                + ((button_width + UiWindow::DEBUG_BUTTON_GAP) * 2) as i32,
            buttons_y,
            button_width,
            UiWindow::DEBUG_BUTTON_HEIGHT,
        );

        let tiles_y = UiWindow::DEBUG_TOP_Y
            + UiWindow::DEBUG_SCREEN_PANEL_HEIGHT as i32
            + UiWindow::DEBUG_PANEL_GAP as i32;

        Self::draw_tiles_panel(
            canvas,
            texture_creator,
            bold_font,
            ppu,
            UiWindow::DEBUG_LEFT_COLUMN_X,
            tiles_y,
            UiWindow::DEBUG_LEFT_COLUMN_WIDTH,
            UiWindow::DEBUG_TILES_PANEL_HEIGHT,
        );

        let tilemap_1_y = UiWindow::DEBUG_TOP_Y;
        let tilemap_2_y = tilemap_1_y
            + UiWindow::DEBUG_TILEMAP_PANEL_HEIGHT as i32
            + UiWindow::DEBUG_PANEL_GAP as i32;

        let tilemap_0_title = format!("Tilemap 0: 0x9800");
        let tilemap_1_title = format!("Tilemap 1: 0x9C00");

        Self::draw_tilemap_panel(
            canvas,
            texture_creator,
            bold_font,
            &tilemap_0_title,
            &ppu.read_tilemap_0_raw(),
            UiWindow::DEBUG_RIGHT_COLUMN_X,
            tilemap_1_y,
            UiWindow::DEBUG_RIGHT_COLUMN_WIDTH,
            UiWindow::DEBUG_TILEMAP_PANEL_HEIGHT,
        );

        Self::draw_tilemap_panel(
            canvas,
            texture_creator,
            bold_font,
            &tilemap_1_title,
            &ppu.read_tilemap_1_raw(),
            UiWindow::DEBUG_RIGHT_COLUMN_X,
            tilemap_2_y,
            UiWindow::DEBUG_RIGHT_COLUMN_WIDTH,
            UiWindow::DEBUG_TILEMAP_PANEL_HEIGHT,
        );
    }

    fn draw_panel(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        title_font: &Font,
        title: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        let panel_rect = Rect::new(x, y, width, height);

        canvas.set_draw_color(Color::RGB(32, 32, 32));
        let _ = canvas.fill_rect(panel_rect);

        canvas.set_draw_color(Color::RGB(70, 70, 70));
        let _ = canvas.draw_rect(panel_rect);

        draw_text(canvas, texture_creator, title_font, title, x + 8, y + 8);
    }

    fn draw_screen_panel(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        title_font: &Font,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        Self::draw_panel(canvas, texture_creator, title_font, "Screen", x, y, width, height);

        let screen_x = x + ((width - UiWindow::DEBUG_SCREEN_WIDTH) / 2) as i32;
        let screen_y = y + 34;

        canvas.set_draw_color(Color::RGB(70, 70, 70));
        let _ = canvas.draw_rect(Rect::new(
            screen_x - 1,
            screen_y - 1,
            UiWindow::DEBUG_SCREEN_WIDTH + 2,
            UiWindow::DEBUG_SCREEN_HEIGHT + 2,
        ));

        Self::draw_screen_framebuffer(
            canvas,
            framebuffer,
            screen_x,
            screen_y,
            UiWindow::DEBUG_SCALE_FACTOR,
        );
    }

    fn draw_cpu_panel(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        bold_font: &Font,
        cpu: &Cpu,
        ppu: &Ppu,
        cycle: usize,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        Self::draw_panel(
            canvas,
            texture_creator,
            bold_font,
            "Registers",
            x,
            y,
            width,
            height,
        );

        let cycle_line = format!("M-cycle: {}      Dots: {}", cycle, ppu.dots());
        draw_text(canvas, texture_creator, font, &cycle_line, x + 8, y + 50);

        let reg_line_1 = format!(
            "A:{:02X}  F:{:02X}    B:{:02X}  C:{:02X}    D:{:02X}  E:{:02X}",
            cpu.get_register8(crate::cpu::registers::Reg8::A),
            cpu.get_register8(crate::cpu::registers::Reg8::F),
            cpu.get_register8(crate::cpu::registers::Reg8::B),
            cpu.get_register8(crate::cpu::registers::Reg8::C),
            cpu.get_register8(crate::cpu::registers::Reg8::D),
            cpu.get_register8(crate::cpu::registers::Reg8::E),
        );

        let reg_line_2 = format!(
            "H:{:02X}  L:{:02X}    PC:0x{:04X}     SP:0x{:04X}",
            cpu.get_register8(crate::cpu::registers::Reg8::H),
            cpu.get_register8(crate::cpu::registers::Reg8::L),
            cpu.get_register16(crate::cpu::registers::Reg16::PC),
            cpu.get_register16(crate::cpu::registers::Reg16::SP),
        );

        draw_text(canvas, texture_creator, font, &reg_line_1, x + 8, y + 78);
        draw_text(canvas, texture_creator, font, &reg_line_2, x + 8, y + 102);
    }

    fn draw_tiles_panel(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        bold_font: &Font,
        ppu: &Ppu,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        Self::draw_panel(
            canvas,
            texture_creator,
            bold_font,
            "Tiles 0x8000-97FF",
            x,
            y,
            width,
            height,
        );
        Self::draw_tiles(canvas, ppu, x + 10, y + 34);
    }

    fn draw_tilemap_panel(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        bold_font: &Font,
        title: &str,
        tilemap: &TileMapPixels,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        Self::draw_panel(canvas, texture_creator, bold_font, title, x, y, width, height);

        let tilemap_x = x + ((width - 256) / 2) as i32;
        let tilemap_y = y + 34 + ((height - 34 - 256) / 2) as i32;
        Self::draw_tilemap(canvas, tilemap, tilemap_x, tilemap_y);
    }

    fn draw_tilemap(
        canvas: &mut Canvas<Window>,
        tilemap: &TileMapPixels,
        start_x: i32,
        start_y: i32,
    ) {
        for (row, pixels) in tilemap.iter().enumerate() {
            for (col, shade) in pixels.iter().enumerate() {
                canvas.set_draw_color(Self::dmg_color(*shade));

                let rect = Rect::new(start_x + col as i32, start_y + row as i32, 1, 1);

                let _ = canvas.fill_rect(rect);
            }
        }
    }

    fn draw_tiles(canvas: &mut Canvas<Window>, ppu: &Ppu, start_x: i32, start_y: i32) {
        let tiles = ppu.read_all_tiles();
        let scale = 2;
        let tiles_per_row = 20;

        for (i, tile) in tiles.iter().enumerate() {
            let tile_x = (i % tiles_per_row) as i32;
            let tile_y = (i / tiles_per_row) as i32;

            for row in 0..8 {
                for col in 0..8 {
                    let value = tile.data[row][col];
                    let color = Self::dmg_color(value);

                    canvas.set_draw_color(color);
                    let x = start_x + (tile_x * 8 + col as i32) * scale;
                    let y = start_y + (tile_y * 8 + row as i32) * scale;

                    let rect = Rect::new(x, y, scale as u32, scale as u32);
                    let _ = canvas.fill_rect(rect);
                }
            }
        }
    }

    fn draw_screen_framebuffer(
        canvas: &mut Canvas<Window>,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        start_x: i32,
        start_y: i32,
        scale: u32,
    ) {
        let scale_i32 = scale as i32;

        for (row, pixels) in framebuffer.iter().enumerate() {
            for (col, shade) in pixels.iter().enumerate() {
                canvas.set_draw_color(Self::dmg_color(*shade));

                let rect = Rect::new(
                    start_x + col as i32 * scale_i32,
                    start_y + row as i32 * scale_i32,
                    scale,
                    scale,
                );

                let _ = canvas.fill_rect(rect);
            }
        }
    }

    fn dmg_color(shade: u8) -> Color {
        match shade {
            0 => Color::RGB(255, 255, 255),
            1 => Color::RGB(170, 170, 170),
            2 => Color::RGB(85, 85, 85),
            _ => Color::RGB(0, 0, 0),
        }
    }

    fn draw_debug_button(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        label: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        let button_rect = Rect::new(x, y, width, height);

        canvas.set_draw_color(Color::RGB(48, 48, 48));
        let _ = canvas.fill_rect(button_rect);

        canvas.set_draw_color(Color::RGB(85, 85, 85));
        let _ = canvas.draw_rect(button_rect);

        let (text_width, text_height) = font.size_of(label).unwrap_or((0, 0));

        let text_x = x + ((width as i32 - text_width as i32) / 2);
        let text_y = y + ((height as i32 - text_height as i32) / 2);

        draw_text(canvas, texture_creator, font, label, text_x, text_y);
    }
}
