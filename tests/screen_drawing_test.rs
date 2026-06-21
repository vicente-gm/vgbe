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

use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "sdl")]
use std::time::Duration;

#[cfg(feature = "sdl")]
use vgbe::cpu;
#[cfg(feature = "sdl")]
use vgbe::cpu::registers::{FLAG_C, FLAG_H, FLAG_Z, Reg8, Reg16};
#[cfg(feature = "sdl")]
use vgbe::frontend::window::Window;
use vgbe::bus::MemoryBus;
use vgbe::ppu;
#[cfg(feature = "sdl")]
use vgbe::ppu::timing::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[cfg(feature = "sdl")]
fn create_cpu(mem: Rc<RefCell<MemoryBus>>) -> cpu::Cpu {
    let mut my_cpu = cpu::Cpu::new(Rc::clone(&mem));

    let flags: u8 = FLAG_C | FLAG_H | FLAG_Z;
    my_cpu.set_register8(Reg8::A, 0x01);
    my_cpu.set_register8(Reg8::F, flags);
    my_cpu.set_register8(Reg8::B, 0x00);
    my_cpu.set_register8(Reg8::C, 0x13);
    my_cpu.set_register8(Reg8::D, 0x00);
    my_cpu.set_register8(Reg8::E, 0xD8);
    my_cpu.set_register8(Reg8::H, 0x01);
    my_cpu.set_register8(Reg8::L, 0x4D);
    my_cpu.set_register16(Reg16::SP, 0xFFFE);
    my_cpu.set_register16(Reg16::PC, 0x0100);

    my_cpu
}

fn encode_tile(rows: [[u8; 8]; 8]) -> [u8; 16] {
    let mut bytes = [0u8; 16];

    for row in 0..8 {
        let mut lo = 0u8;
        let mut hi = 0u8;

        for col in 0..8 {
            let bit = 7 - col;
            let color = rows[row][col] & 0b11;

            lo |= (color & 1) << bit;
            hi |= ((color >> 1) & 1) << bit;
        }

        bytes[row * 2] = lo;
        bytes[row * 2 + 1] = hi;
    }

    bytes
}

fn write_tile(mem: &mut MemoryBus, tile_index: u16, rows: [[u8; 8]; 8]) {
    let tile = encode_tile(rows);
    let base = 0x8000 + tile_index * 16;

    for (offset, byte) in tile.iter().enumerate() {
        mem.write_byte(base + offset as u16, *byte);
    }
}

fn fill_test_screen(mem: &mut MemoryBus) {
    mem.init_post_boot_dmg();

    // Identity palette: framebuffer values match tile color IDs.
    mem.write_byte(0xFF47, 0b1110_0100);

    write_tile(mem, 0, [[0; 8]; 8]);
    write_tile(mem, 1, [[1; 8]; 8]);
    write_tile(mem, 2, [[2; 8]; 8]);
    write_tile(mem, 3, [[3; 8]; 8]);

    let checker = std::array::from_fn(|row| {
        std::array::from_fn(|col| if (row + col) % 2 == 0 { 0 } else { 3 })
    });
    write_tile(mem, 4, checker);

    let gradient = std::array::from_fn(|_| std::array::from_fn(|col| (col / 2) as u8));
    write_tile(mem, 5, gradient);

    for tile_y in 0..32u16 {
        for tile_x in 0..32u16 {
            let tile_index = match (tile_x + tile_y) % 6 {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => 5,
            };

            mem.write_byte(0x9800 + tile_y * 32 + tile_x, tile_index);
        }
    }
}

fn fill_bg_tilemap(mem: &mut MemoryBus, tile_index: u8) {
    for tile_y in 0..32u16 {
        for tile_x in 0..32u16 {
            mem.write_byte(0x9800 + tile_y * 32 + tile_x, tile_index);
        }
    }
}

#[cfg(feature = "sdl")]
fn fill_debug_tilemaps(mem: &mut MemoryBus) {
    mem.init_post_boot_dmg();
    mem.write_byte(0xFF47, 0b1110_0100);

    write_tile(mem, 0, [[0; 8]; 8]);
    write_tile(mem, 1, [[1; 8]; 8]);
    write_tile(mem, 2, [[2; 8]; 8]);
    write_tile(mem, 3, [[3; 8]; 8]);

    let diagonal = std::array::from_fn(|row| {
        std::array::from_fn(|col| if row == col || row + col == 7 { 3 } else { 0 })
    });
    let vertical_stripes = std::array::from_fn(|_| std::array::from_fn(|col| (col % 4) as u8));
    let horizontal_stripes = std::array::from_fn(|row| std::array::from_fn(|_| (row % 4) as u8));
    let checker = std::array::from_fn(|row| {
        std::array::from_fn(|col| if (row + col) % 2 == 0 { 1 } else { 3 })
    });

    write_tile(mem, 4, diagonal);
    write_tile(mem, 5, vertical_stripes);
    write_tile(mem, 6, horizontal_stripes);
    write_tile(mem, 7, checker);

    for tile_y in 0..32u16 {
        for tile_x in 0..32u16 {
            let tilemap_0_tile = if tile_x == tile_y || tile_x + tile_y == 31 {
                4
            } else {
                ((tile_x / 4 + tile_y / 4) % 4) as u8
            };

            let tilemap_1_tile = if tile_y % 8 == 0 || tile_x % 8 == 0 {
                7
            } else if tile_x < 16 {
                5
            } else {
                6
            };

            mem.write_byte(0x9800 + tile_y * 32 + tile_x, tilemap_0_tile);
            mem.write_byte(0x9C00 + tile_y * 32 + tile_x, tilemap_1_tile);
        }
    }
}

fn write_oam_object(
    mem: &mut MemoryBus,
    object_index: u16,
    y: u8,
    x: u8,
    tile_index: u8,
    attrs: u8,
) {
    let base = 0xFE00 + object_index * 4;

    mem.write_byte(base, y);
    mem.write_byte(base + 1, x);
    mem.write_byte(base + 2, tile_index);
    mem.write_byte(base + 3, attrs);
}

fn run_until_frame(ppu: &mut ppu::Ppu) {
    for _ in 0..(456 * 144) {
        ppu.tick();
    }
}

#[cfg(feature = "sdl")]
fn build_manual_framebuffer() -> [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT] {
    let mut framebuffer = [[0u8; SCREEN_WIDTH]; SCREEN_HEIGHT];

    fill_ellipse(&mut framebuffer, 80, 78, 36, 30, 1);
    fill_ellipse(&mut framebuffer, 57, 64, 13, 17, 1);
    fill_ellipse(&mut framebuffer, 103, 64, 13, 17, 1);

    draw_line(&mut framebuffer, 52, 46, 32, 12, 1);
    draw_line(&mut framebuffer, 108, 46, 128, 12, 1);
    draw_line(&mut framebuffer, 32, 12, 43, 35, 3);
    draw_line(&mut framebuffer, 128, 12, 117, 35, 3);

    fill_ellipse(&mut framebuffer, 68, 73, 4, 6, 3);
    fill_ellipse(&mut framebuffer, 92, 73, 4, 6, 3);
    set_pixel_block(&mut framebuffer, 66, 70, 2, 2, 0);
    set_pixel_block(&mut framebuffer, 90, 70, 2, 2, 0);

    fill_ellipse(&mut framebuffer, 54, 83, 7, 6, 2);
    fill_ellipse(&mut framebuffer, 106, 83, 7, 6, 2);

    fill_ellipse(&mut framebuffer, 80, 83, 3, 2, 3);
    draw_line(&mut framebuffer, 80, 86, 75, 91, 3);
    draw_line(&mut framebuffer, 80, 86, 85, 91, 3);

    draw_line(&mut framebuffer, 44, 95, 18, 83, 1);
    draw_line(&mut framebuffer, 116, 95, 142, 83, 1);
    draw_line(&mut framebuffer, 63, 105, 53, 124, 1);
    draw_line(&mut framebuffer, 97, 105, 107, 124, 1);

    draw_outline(&mut framebuffer);

    framebuffer
}

#[cfg(feature = "sdl")]
fn set_pixel(framebuffer: &mut [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT], x: i32, y: i32, color: u8) {
    if x >= 0 && y >= 0 && (x as usize) < SCREEN_WIDTH && (y as usize) < SCREEN_HEIGHT {
        framebuffer[y as usize][x as usize] = color;
    }
}

#[cfg(feature = "sdl")]
fn set_pixel_block(
    framebuffer: &mut [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: u8,
) {
    for row in y..(y + height) {
        for col in x..(x + width) {
            set_pixel(framebuffer, col, row, color);
        }
    }
}

#[cfg(feature = "sdl")]
fn fill_ellipse(
    framebuffer: &mut [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    center_x: i32,
    center_y: i32,
    radius_x: i32,
    radius_y: i32,
    color: u8,
) {
    for y in (center_y - radius_y)..=(center_y + radius_y) {
        for x in (center_x - radius_x)..=(center_x + radius_x) {
            let dx = x - center_x;
            let dy = y - center_y;

            if dx * dx * radius_y * radius_y + dy * dy * radius_x * radius_x
                <= radius_x * radius_x * radius_y * radius_y
            {
                set_pixel(framebuffer, x, y, color);
            }
        }
    }
}

#[cfg(feature = "sdl")]
fn draw_line(
    framebuffer: &mut [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u8,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        set_pixel_block(framebuffer, x0 - 1, y0 - 1, 3, 3, color);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[cfg(feature = "sdl")]
fn draw_outline(framebuffer: &mut [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
    let original = *framebuffer;

    for y in 1..(SCREEN_HEIGHT - 1) {
        for x in 1..(SCREEN_WIDTH - 1) {
            if original[y][x] == 0 {
                let touches_body = original[y - 1][x] == 1
                    || original[y + 1][x] == 1
                    || original[y][x - 1] == 1
                    || original[y][x + 1] == 1;

                if touches_body {
                    framebuffer[y][x] = 3;
                }
            }
        }
    }
}

#[test]
fn test_screen_framebuffer_renders_background_tilemap() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        fill_test_screen(&mut mem);
    }

    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    run_until_frame(&mut my_ppu);

    assert!(my_ppu.take_frame_ready());

    let framebuffer = my_ppu.get_framebuffer();

    assert_eq!(framebuffer[0][0], 0);
    assert_eq!(framebuffer[0][8], 1);
    assert_eq!(framebuffer[0][16], 2);
    assert_eq!(framebuffer[0][24], 3);

    assert_eq!(framebuffer[0][32], 0);
    assert_eq!(framebuffer[0][33], 3);
    assert_eq!(framebuffer[0][40], 0);
    assert_eq!(framebuffer[0][42], 1);
    assert_eq!(framebuffer[0][44], 2);
    assert_eq!(framebuffer[0][46], 3);
}

#[test]
fn test_ppu_reads_both_tilemaps_as_pixels() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        mem.write_byte(0xFF47, 0b1110_0100);

        write_tile(&mut mem, 1, [[1; 8]; 8]);
        write_tile(&mut mem, 2, [[2; 8]; 8]);

        mem.write_byte(0x9800, 1);
        mem.write_byte(0x9C00, 2);
    }

    let my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    let tilemap_0 = my_ppu.read_tilemap_0();
    let tilemap_1 = my_ppu.read_tilemap_1();

    assert_eq!(tilemap_0[0][0], 1);
    assert_eq!(tilemap_0[7][7], 1);
    assert_eq!(tilemap_1[0][0], 2);
    assert_eq!(tilemap_1[7][7], 2);
}

#[test]
fn test_oam_scan_selects_only_first_ten_y_matching_objects() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        mem.write_byte(0xFF40, 0x93);
        mem.write_byte(0xFF47, 0b1110_0100);
        mem.write_byte(0xFF48, 0b1110_0100);

        write_tile(&mut mem, 0, [[0; 8]; 8]);
        write_tile(&mut mem, 1, [[3; 8]; 8]);
        fill_bg_tilemap(&mut mem, 0);

        for object_index in 0..10u16 {
            write_oam_object(&mut mem, object_index, 16, 0, 1, 0);
        }
        write_oam_object(&mut mem, 10, 16, 16, 1, 0);
    }

    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    run_until_frame(&mut my_ppu);

    assert_eq!(my_ppu.get_framebuffer()[0][8], 0);
}

#[test]
fn test_oam_priority_masks_lower_priority_object_before_bg_priority() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        mem.write_byte(0xFF40, 0x93);
        mem.write_byte(0xFF47, 0b1110_0100);
        mem.write_byte(0xFF48, 0b1110_0100);

        write_tile(&mut mem, 1, [[3; 8]; 8]);
        write_tile(&mut mem, 2, [[1; 8]; 8]);
        write_tile(&mut mem, 3, [[2; 8]; 8]);
        fill_bg_tilemap(&mut mem, 2);

        write_oam_object(&mut mem, 0, 16, 16, 1, 0x80);
        write_oam_object(&mut mem, 1, 16, 17, 3, 0x00);
    }

    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    run_until_frame(&mut my_ppu);

    let framebuffer = my_ppu.get_framebuffer();

    assert_eq!(framebuffer[0][9], 1);
    assert_eq!(framebuffer[0][16], 2);
}

#[test]
fn test_oam_8x16_objects_ignore_tile_index_lsb() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        mem.init_post_boot_dmg();
        mem.write_byte(0xFF40, 0x97);
        mem.write_byte(0xFF47, 0b1110_0100);
        mem.write_byte(0xFF48, 0b1110_0100);

        write_tile(&mut mem, 0, [[0; 8]; 8]);
        write_tile(&mut mem, 2, [[1; 8]; 8]);
        write_tile(&mut mem, 3, [[2; 8]; 8]);
        fill_bg_tilemap(&mut mem, 0);

        write_oam_object(&mut mem, 0, 16, 8, 3, 0x00);
    }

    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    run_until_frame(&mut my_ppu);

    let framebuffer = my_ppu.get_framebuffer();

    assert_eq!(framebuffer[0][0], 1);
    assert_eq!(framebuffer[8][0], 2);
}

#[test]
#[cfg(feature = "sdl")]
#[ignore = "Opens an SDL window for manual screen drawing inspection."]
fn inspect_screen_drawing_pattern() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let framebuffer = build_manual_framebuffer();

    {
        let mut mem = memory.borrow_mut();
        fill_test_screen(&mut mem);
    }

    let my_cpu = create_cpu(Rc::clone(&memory));
    let my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    let mut window = Window::new(true);

    window.record_frame();
    window.render_framebuffer_with_state(&my_cpu, &my_ppu, &framebuffer);

    while window.handle_events() {
        std::thread::sleep(Duration::from_millis(16));
    }
}

#[test]
#[cfg(feature = "sdl")]
#[ignore = "Opens an SDL window for manual tilemap inspection."]
fn inspect_debug_tilemap_panels_pattern() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let framebuffer = build_manual_framebuffer();

    {
        let mut mem = memory.borrow_mut();
        fill_debug_tilemaps(&mut mem);
    }

    let my_cpu = create_cpu(Rc::clone(&memory));
    let my_ppu = ppu::Ppu::new(Rc::clone(&memory));
    let mut window = Window::new(true);

    window.record_frame();
    window.render_framebuffer_with_state(&my_cpu, &my_ppu, &framebuffer);

    while window.handle_events() {
        std::thread::sleep(Duration::from_millis(16));
    }
}
