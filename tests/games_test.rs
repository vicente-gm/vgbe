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

#![cfg(feature = "sdl")]

use std::fs::File;
use std::time::{Duration, Instant};
use vgbe::emulator::GameBoy;
use vgbe::frontend::window::Window;
use vgbe::instruction_logger::InstructionLogger;


#[test]
#[ignore = "Opens an SDL window for manual Tetris inspection."]
fn test_tetris() {
    let rom = String::from("roms/Tetris.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let mut gameboy = GameBoy::load_rom(&mut file, &rom);

    if let Ok(tas_file) = std::env::var("VGBE_TAS_FILE") {
        if !tas_file.trim().is_empty() {
            gameboy.load_tas_file(&tas_file).unwrap();
        }
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);

    while window.handle_events() {
        for (key, pressed) in window.take_tas_key_events() {
            gameboy.set_tas_key_state(key, pressed);
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger
                    .log_if_debug_with_timing(
                        window.is_debug_mode(),
                        &executed_instruction,
                        gameboy.cpu(),
                        gameboy.cycle(),
                        gameboy.ppu().dots(),
                    )
                    .unwrap();
            }

            window.finish_tick();
            did_tick = true;
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
    }
}


#[test]
#[ignore = "Opens an SDL window for manual Pokemon Yellow inspection."]
fn test_pokemon_amarillo() {
    let rom = String::from("roms/Pokemon-Amarillo.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let mut gameboy = GameBoy::load_rom(&mut file, &rom);

    if let Ok(tas_file) = std::env::var("VGBE_TAS_FILE") {
        if !tas_file.trim().is_empty() {
            gameboy.load_tas_file(&tas_file).unwrap();
        }
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);

    while window.handle_events() {
        for (key, pressed) in window.take_tas_key_events() {
            gameboy.set_tas_key_state(key, pressed);
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger
                    .log_if_debug_with_timing(
                        window.is_debug_mode(),
                        &executed_instruction,
                        gameboy.cpu(),
                        gameboy.cycle(),
                        gameboy.ppu().dots(),
                    )
                    .unwrap();
            }

            window.finish_tick();
            did_tick = true;
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
    }
}

#[test]
#[ignore = "Opens an SDL window for visual Blargg tests."]
fn test_blargg_visual() {
    let rom = String::from("roms/test/blargg/cpu_instrs/cpu_instrs.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let mut gameboy = GameBoy::load_rom(&mut file, &rom);

    if let Ok(tas_file) = std::env::var("VGBE_TAS_FILE") {
        if !tas_file.trim().is_empty() {
            gameboy.load_tas_file(&tas_file).unwrap();
        }
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);

    while window.handle_events() {
        for (key, pressed) in window.take_tas_key_events() {
            gameboy.set_tas_key_state(key, pressed);
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger
                    .log_if_debug_with_timing(
                        window.is_debug_mode(),
                        &executed_instruction,
                        gameboy.cpu(),
                        gameboy.cycle(),
                        gameboy.ppu().dots(),
                    )
                    .unwrap();
            }

            window.finish_tick();
            did_tick = true;
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
    }
}

#[test]
#[ignore = "Opens an SDL window for acid2 tests."]
fn test_acid2() {
    let rom = String::from("roms/dmg-acid2.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let mut gameboy = GameBoy::load_rom(&mut file, &rom);

    if let Ok(tas_file) = std::env::var("VGBE_TAS_FILE") {
        if !tas_file.trim().is_empty() {
            gameboy.load_tas_file(&tas_file).unwrap();
        }
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);

    while window.handle_events() {
        for (key, pressed) in window.take_tas_key_events() {
            gameboy.set_tas_key_state(key, pressed);
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger
                    .log_if_debug_with_timing(
                        window.is_debug_mode(),
                        &executed_instruction,
                        gameboy.cpu(),
                        gameboy.cycle(),
                        gameboy.ppu().dots(),
                    )
                    .unwrap();
            }

            window.finish_tick();
            did_tick = true;
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
    }
}

#[test]
#[ignore = "Opens an SDL window for manual Pokemon Red inspection."]
fn test_pokemon_rojo() {
    let rom = String::from("roms/Pokemon-Rojo.gb");
    let mut file = File::open(&rom).expect("Could not open ROM file");

    let mut gameboy = GameBoy::load_rom(&mut file, &rom);

    if let Ok(tas_file) = std::env::var("VGBE_TAS_FILE") {
        if !tas_file.trim().is_empty() {
            gameboy.load_tas_file(&tas_file).unwrap();
        }
    }

    let mut window = Window::new(true);
    let mut instruction_logger = InstructionLogger::new(&rom);

    let mut last_ui_redraw = Instant::now();
    let ui_redraw_interval = Duration::from_millis(16);

    while window.handle_events() {
        for (key, pressed) in window.take_tas_key_events() {
            gameboy.set_tas_key_state(key, pressed);
        }

        let mut did_tick = false;

        if window.should_tick() {
            let executed_instruction = if window.is_step_requested() {
                gameboy.step_instruction()
            } else {
                gameboy.tick()
            };

            if let Some(executed_instruction) = executed_instruction {
                instruction_logger
                    .log_if_debug_with_timing(
                        window.is_debug_mode(),
                        &executed_instruction,
                        gameboy.cpu(),
                        gameboy.cycle(),
                        gameboy.ppu().dots(),
                    )
                    .unwrap();
            }

            window.finish_tick();
            did_tick = true;
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
    }
}