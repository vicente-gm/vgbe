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

#![cfg(feature = "native-apu")]

use std::cell::RefCell;
use std::rc::Rc;

use vgbe::apu::{Apu, DEFAULT_BUFFER_FRAMES, DEFAULT_SAMPLE_RATE, GbApu};
use vgbe::bus::MemoryBus;
use vgbe::emulator::GameBoy;

const GB_FRAME_M_CYCLES: usize = 17_556;

#[test]
fn gbapu_adapter_reads_writes_and_drains_i16_samples() {
    let mut apu = GbApu::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_FRAMES)
        .expect("gbapu adapter should initialize");

    apu.write_register(0xFF26, 0x80);
    apu.write_register(0xFF24, 0x77);
    apu.write_register(0xFF25, 0x11);

    assert_ne!(apu.read_register(0xFF26) & 0x80, 0);

    apu.step(4);
    apu.end_frame();

    let mut samples = Vec::new();
    apu.drain_samples(&mut samples);

    assert_eq!(samples.len() % 2, 0);
}

#[test]
fn gameboy_extracts_audio_after_one_frame_of_cpu_cycles() {
    let memory = Rc::new(RefCell::new(MemoryBus::init_mem_void()));
    let mut gameboy = GameBoy::new(memory);

    for _ in 0..(GB_FRAME_M_CYCLES - 1) {
        let _ = gameboy.tick();
    }

    assert!(gameboy.take_audio_samples().is_none());

    let _ = gameboy.tick();
    let samples = gameboy
        .take_audio_samples()
        .expect("expected audio samples after one full Game Boy frame");

    assert!(!samples.is_empty());
    assert_eq!(samples.len() % 2, 0);
}
