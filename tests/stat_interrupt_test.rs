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

use vgbe::cpu::interrupts::{INTERRUPT_FLAG_DIR, INTERRUPT_LCD_BIT};
use vgbe::bus::MemoryBus;
use vgbe::ppu;
use vgbe::ppu::registers::{LYC_ADDR, STAT_ADDR, STAT_HBLANK_INTERRUPT_ENABLE, STAT_LYC_EQ_LY_FLAG, STAT_LYC_INTERRUPT_ENABLE, STAT_MODE_MASK, STAT_OAM_INTERRUPT_ENABLE};

fn lcd_interrupt_requested(mem: &MemoryBus) -> bool {
    mem.read_byte(INTERRUPT_FLAG_DIR) & (1 << INTERRUPT_LCD_BIT) != 0
}

fn clear_lcd_interrupt(mem: &mut MemoryBus) {
    let iflag = mem.read_byte(INTERRUPT_FLAG_DIR);
    mem.write_byte(INTERRUPT_FLAG_DIR, iflag & !(1 << INTERRUPT_LCD_BIT));
}

#[test]
fn test_stat_oam_interrupt_is_requested_on_initial_rising_edge() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));

    {
        let mut mem = memory.borrow_mut();
        clear_lcd_interrupt(&mut mem);
        mem.write_byte(STAT_ADDR, STAT_OAM_INTERRUPT_ENABLE);
    }

    let _ppu = ppu::Ppu::new(Rc::clone(&memory));

    assert!(lcd_interrupt_requested(&memory.borrow()));
}

#[test]
fn test_stat_hblank_interrupt_is_requested_on_hblank_entry() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));

    {
        let mut mem = memory.borrow_mut();
        clear_lcd_interrupt(&mut mem);
        mem.write_byte(STAT_ADDR, STAT_HBLANK_INTERRUPT_ENABLE);
    }

    for _ in 0..252 {
        my_ppu.tick();
    }

    assert!(lcd_interrupt_requested(&memory.borrow()));
}

#[test]
fn test_stat_lyc_interrupt_is_requested_when_ly_matches_lyc() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));

    {
        let mut mem = memory.borrow_mut();
        clear_lcd_interrupt(&mut mem);
        mem.write_byte(LYC_ADDR, 1);
        mem.write_byte(STAT_ADDR, STAT_LYC_INTERRUPT_ENABLE);
    }

    for _ in 0..456 {
        my_ppu.tick();
    }

    let mem = memory.borrow();

    assert_eq!(mem.read_byte(STAT_ADDR) & STAT_LYC_EQ_LY_FLAG, STAT_LYC_EQ_LY_FLAG);
    assert!(lcd_interrupt_requested(&mem));
}

#[test]
fn test_stat_interrupt_line_only_requests_on_rising_edge() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut my_ppu = ppu::Ppu::new(Rc::clone(&memory));

    {
        let mut mem = memory.borrow_mut();
        clear_lcd_interrupt(&mut mem);
        mem.write_byte(STAT_ADDR, STAT_HBLANK_INTERRUPT_ENABLE);
    }

    for _ in 0..252 {
        my_ppu.tick();
    }

    assert!(lcd_interrupt_requested(&memory.borrow()));

    {
        let mut mem = memory.borrow_mut();
        clear_lcd_interrupt(&mut mem);
    }

    my_ppu.tick();

    assert!(!lcd_interrupt_requested(&memory.borrow()));
}

#[test]
fn test_stat_write_preserves_read_only_status_bits() {
    let memory: Rc<RefCell<MemoryBus>> = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let _ppu = ppu::Ppu::new(Rc::clone(&memory));

    let before = memory.borrow().read_byte(STAT_ADDR) & (STAT_MODE_MASK | STAT_LYC_EQ_LY_FLAG);

    memory.borrow_mut().write_byte(STAT_ADDR, 0x00);

    let after = memory.borrow().read_byte(STAT_ADDR) & (STAT_MODE_MASK | STAT_LYC_EQ_LY_FLAG);

    assert_eq!(after, before);
}
