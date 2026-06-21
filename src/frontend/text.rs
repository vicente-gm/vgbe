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

use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::ttf::Font;

pub fn draw_text(canvas: &mut Canvas<Window>, texture_creator: &TextureCreator<WindowContext>, font: &Font, text: &str, x: i32, y: i32) {
    let surface = font
        .render(text)
        .blended(Color::RGB(220, 220, 220))
        .unwrap();

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

    let TextureQuery { width, height, .. } = texture.query();

    let target = Rect::new(x, y, width, height);

    canvas.copy(&texture, None, Some(target)).unwrap();
}