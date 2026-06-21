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
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use crate::apu::{SharedApu, new_shared_default_apu};
use crate::bus::MemoryBus;
use crate::cpu::{Cpu, ExecutedInstruction};
use crate::ppu::Ppu;
use crate::tas::{TasEvent, TasKey, TasLoadError, load_tas_file};

const APU_T_CYCLES_PER_M_CYCLE: u32 = 4;
const AUDIO_FRAME_M_CYCLES: u32 = 17_556;

/// Core Game Boy emulator state.
///
/// This type owns the CPU and PPU and keeps them connected to the same memory
/// bus. Frontends should drive this type instead of coordinating subsystems
/// directly.
pub struct GameBoy {
    memory: Rc<RefCell<MemoryBus>>,
    apu: SharedApu,
    cpu: Cpu,
    ppu: Ppu,
    cycle: usize,
    audio_frame_cycles: u32,
    emulation_frame_ready: bool,
    audio_samples: Vec<i16>,
    tas_events: Vec<TasEvent>,
    next_tas_event: usize,
}

impl GameBoy {
    /// Creates a Game Boy from an existing memory bus.
    pub fn new(memory: Rc<RefCell<MemoryBus>>) -> Self {
        let apu = new_shared_default_apu();
        {
            let mut memory = memory.borrow_mut();
            memory.set_apu(Rc::clone(&apu));
            memory.init_post_boot_apu_dmg();
        }

        let cpu = Cpu::new(Rc::clone(&memory));
        let ppu = Ppu::new(Rc::clone(&memory));

        Self {
            memory,
            apu,
            cpu,
            ppu,
            cycle: 0,
            audio_frame_cycles: 0,
            emulation_frame_ready: false,
            audio_samples: Vec::new(),
            tas_events: Vec::new(),
            next_tas_event: 0,
        }
    }

    /// Loads a ROM file and creates a Game Boy around it.
    pub fn load_rom(file: &mut File, rom: &str) -> Self {
        let memory = Rc::new(RefCell::new(MemoryBus::load_rom(file, rom)));
        let mut gameboy = Self::new(memory);
        gameboy.cpu.init_post_boot_dmg();
        gameboy
    }

    /// Advances the CPU by one M-cycle and the PPU by four dots.
    pub fn tick(&mut self) -> Option<ExecutedInstruction> {
        self.apply_due_tas_events();

        let executed_instruction = self.cpu.tick();

        // gbapu is clocked in T-cycles (4_194_304 Hz); this core ticks CPU M-cycles.
        self.apu.borrow_mut().step(APU_T_CYCLES_PER_M_CYCLE);

        for _ in 0..4 {
            self.ppu.tick();
        }

        self.tick_audio_frame();

        self.cycle = self.cycle.wrapping_add(1);

        executed_instruction
    }

    /// Advances until the current CPU instruction finishes.
    pub fn step_instruction(&mut self) -> Option<ExecutedInstruction> {
        let executed_instruction = self.tick();

        while self.cpu.is_blocked() {
            let _ = self.tick();
        }

        executed_instruction
    }

    /// Returns and clears the PPU frame-ready flag.
    pub fn take_frame_ready(&mut self) -> bool {
        self.ppu.take_frame_ready()
    }

    pub fn drain_audio_samples(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.audio_samples)
    }

    pub fn take_audio_samples(&mut self) -> Option<Vec<i16>> {
        if self.audio_samples.is_empty() {
            None
        } else {
            Some(self.drain_audio_samples())
        }
    }

    pub fn take_emulation_frame_ready(&mut self) -> bool {
        if self.emulation_frame_ready {
            self.emulation_frame_ready = false;
            true
        } else {
            false
        }
    }

    /// Requests a joypad interrupt and wakes STOP mode if needed.
    pub fn request_joypad_interrupt(&mut self) {
        self.cpu.request_joypad_interrupt();
    }

    pub fn cycle(&self) -> usize {
        self.cycle
    }

    pub fn load_tas_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), TasLoadError> {
        let events = load_tas_file(path)?;
        self.set_tas_events(events);
        Ok(())
    }

    pub fn maybe_load_tas_file<P: AsRef<Path>>(
        &mut self,
        path: Option<P>,
    ) -> Result<(), TasLoadError> {
        if let Some(path) = path {
            self.load_tas_file(path)?;
        }

        Ok(())
    }

    pub fn set_tas_events(&mut self, events: Vec<TasEvent>) {
        let mut indexed_events: Vec<(usize, TasEvent)> = events.into_iter().enumerate().collect();
        indexed_events.sort_by_key(|(index, event)| (event.cycle, *index));

        self.tas_events = indexed_events.into_iter().map(|(_, event)| event).collect();
        self.next_tas_event = 0;
    }

    pub fn clear_tas_events(&mut self) {
        self.tas_events.clear();
        self.next_tas_event = 0;
    }

    pub fn set_tas_key_state(&mut self, key: TasKey, pressed: bool) {
        let press_transition = self.memory.borrow_mut().set_tas_key_state(key, pressed);

        if press_transition {
            self.cpu.request_joypad_interrupt();
        }
    }

    fn apply_due_tas_events(&mut self) {
        while self.next_tas_event < self.tas_events.len()
            && self.tas_events[self.next_tas_event].cycle <= self.cycle
        {
            let event = self.tas_events[self.next_tas_event];
            self.set_tas_key_state(event.key, event.pressed);
            self.next_tas_event += 1;
        }
    }

    fn tick_audio_frame(&mut self) {
        self.audio_frame_cycles += 1;

        if self.audio_frame_cycles < AUDIO_FRAME_M_CYCLES {
            return;
        }

        self.audio_frame_cycles -= AUDIO_FRAME_M_CYCLES;
        self.emulation_frame_ready = true;

        let mut apu = self.apu.borrow_mut();
        apu.end_frame();
        apu.drain_samples(&mut self.audio_samples);
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    pub fn memory(&self) -> Rc<RefCell<MemoryBus>> {
        Rc::clone(&self.memory)
    }
}
