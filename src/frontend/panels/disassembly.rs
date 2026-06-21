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
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::cpu::Cpu;
use crate::frontend::text::draw_text;
use crate::frontend::window::Window as UiWindow;

pub fn draw_disassembly_panel(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    bold_font: &Font,
    cpu: &Cpu,
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

    draw_text(
        canvas,
        texture_creator,
        bold_font,
        "Disassembly",
        x + 8,
        y + 8,
    );

    let line_height = UiWindow::DEBUG_DISASSEMBLY_LINE_HEIGHT as i32;
    let mut line_y = y + 34;

    let queue = cpu.get_instruction_queue();
    let last_index = queue.len().saturating_sub(1);

    for (index, disassembled_instr) in queue.iter().enumerate() {
        let is_last_instruction = index == last_index;
        let current_font = if is_last_instruction { bold_font } else { font };

        if is_last_instruction {
            let highlight_rect = Rect::new(
                x + 4,
                line_y - 2,
                width - 8,
                line_height as u32,
            );

            canvas.set_draw_color(Color::RGB(46, 46, 46));
            let _ = canvas.fill_rect(highlight_rect);
        }

        let addr = format!("0x{:04X}:", disassembled_instr.address);

        draw_text(
            canvas,
            texture_creator,
            current_font,
            &addr,
            x + 8,
            line_y,
        );

        draw_text(
            canvas,
            texture_creator,
            current_font,
            disassembled_instr.instr.asm,
            x + 90,
            line_y,
        );

        line_y += line_height;
    }
}
