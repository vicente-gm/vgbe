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

use vgbe::cpu::interrupts::{INTERRUPT_FLAG_DIR, INTERRUPT_TIMER_BIT};
use vgbe::bus::MemoryBus;

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

fn timer_interrupt_requested(mem: &MemoryBus) -> bool {
    mem.read_byte(INTERRUPT_FLAG_DIR) & (1 << INTERRUPT_TIMER_BIT) != 0
}

#[test]
fn test_timer_disabled_only_div_counts() {
    let mut mem = MemoryBus::init_mem_void();

    for _ in 0..64 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(DIV_ADDR), 1);
    assert_eq!(mem.read_byte(TIMA_ADDR), 0);
}

#[test]
fn test_tima_increments_at_4_mcycle_frequency() {
    let mut mem = MemoryBus::init_mem_void();

    mem.write_byte(TAC_ADDR, 0b101);

    for _ in 0..4 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(TIMA_ADDR), 1);

    for _ in 0..4 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(TIMA_ADDR), 2);
}

#[test]
fn test_tima_overflow_reloads_tma_and_requests_interrupt() {
    let mut mem = MemoryBus::init_mem_void();

    mem.write_byte(TIMA_ADDR, 0xFF);
    mem.write_byte(TMA_ADDR, 0xAB);
    mem.write_byte(TAC_ADDR, 0b101);

    for _ in 0..4 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(TIMA_ADDR), 0xAB);
    assert!(timer_interrupt_requested(&mem));
}

#[test]
fn test_div_write_resets_div() {
    let mut mem = MemoryBus::init_mem_void();

    for _ in 0..128 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(DIV_ADDR), 2);

    mem.write_byte(DIV_ADDR, 0xFF);

    assert_eq!(mem.read_byte(DIV_ADDR), 0);
}

#[test]
fn test_tac_write_can_increment_tima_once() {
    let mut mem = MemoryBus::init_mem_void();

    mem.write_byte(TAC_ADDR, 0b101);

    for _ in 0..2 {
        mem.tick_timer();
    }

    assert_eq!(mem.read_byte(TIMA_ADDR), 0);

    mem.write_byte(TAC_ADDR, 0b100);

    assert_eq!(mem.read_byte(TIMA_ADDR), 1);
}
