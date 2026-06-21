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

use std::fs::File;
use std::time::{Duration, Instant};

use crate::emulator::GameBoy;
use crate::frontend::audio::Audio;
use crate::frontend::window::Window;
use crate::instruction_logger::InstructionLogger;

const GAME_BOY_FRAME_T_CYCLES: u64 = 70_224;
const GAME_BOY_CLOCK_HZ: u64 = 4_194_304;
const NANOS_PER_SECOND: u64 = 1_000_000_000;

pub fn run(
    file: &mut File,
    rom: &str,
    debug: bool,
    tas_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut gameboy = GameBoy::load_rom(file, rom);

    let mut window = Window::new(debug);
    let mut audio = Audio::new().ok();
    let mut instruction_logger = InstructionLogger::new(rom);
    let tas_playback = tas_file.is_some();

    if let Some(tas_file) = tas_file {
        gameboy.load_tas_file(tas_file)?;
    }

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);
    let mut frame_limiter = FrameLimiter::new();

    while window.handle_events() {
        if tas_playback {
            // Drain live button events so keyboard input cannot alter deterministic TAS playback.
            window.take_tas_key_events();
        } else {
            for (key, pressed) in window.take_tas_key_events() {
                gameboy.set_tas_key_state(key, pressed);
            }
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger.log_if_debug_with_timing(
                    window.is_debug_mode(),
                    &executed_instruction,
                    gameboy.cpu(),
                    gameboy.cycle(),
                    gameboy.ppu().dots(),
                )?;
            }

            window.finish_tick();
            did_tick = true;
        }

        if let Some(audio_samples) = gameboy.take_audio_samples() {
            if let Some(audio) = audio.as_mut() {
                audio.push_samples(&audio_samples)?;
            }
        }

        let frame_ready = gameboy.take_frame_ready();
        if frame_ready {
            window.record_frame();
        }

        let window_needs_redraw = window.should_redraw();
        let ui_needs_redraw = did_tick && last_ui_redraw.elapsed() >= ui_redraw_interval;

        if frame_ready || ui_needs_redraw || window_needs_redraw {
            window.render_dirty_with_cycle(
                gameboy.cpu(),
                gameboy.ppu(),
                gameboy.cycle(),
                frame_ready || window_needs_redraw,
            );
            last_ui_redraw = Instant::now();
        } else if !did_tick {
            std::thread::sleep(Duration::from_millis(1));
        }

        if gameboy.take_emulation_frame_ready() {
            frame_limiter.wait();
        }
    }

    Ok(())
}

struct FrameLimiter {
    frame_duration: Duration,
    next_frame_time: Instant,
}

impl FrameLimiter {
    fn new() -> Self {
        let frame_duration = game_boy_frame_duration();

        Self {
            frame_duration,
            next_frame_time: Instant::now() + frame_duration,
        }
    }

    fn wait(&mut self) {
        let now = Instant::now();

        if now < self.next_frame_time {
            std::thread::sleep(self.next_frame_time - now);
            self.next_frame_time += self.frame_duration;
        } else {
            self.next_frame_time = now + self.frame_duration;
        }
    }
}

fn game_boy_frame_duration() -> Duration {
    Duration::from_nanos(
        ((NANOS_PER_SECOND * GAME_BOY_FRAME_T_CYCLES) + (GAME_BOY_CLOCK_HZ / 2))
            / GAME_BOY_CLOCK_HZ,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_boy_frame_duration_is_about_sixty_fps() {
        let duration = game_boy_frame_duration();

        assert!(duration >= Duration::from_millis(16));
        assert!(duration < Duration::from_millis(17));
    }
}
