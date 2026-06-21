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

use std::mem::size_of;

use sdl2::audio::{AudioQueue, AudioSpecDesired};

use crate::apu::DEFAULT_SAMPLE_RATE;

const CHANNELS: u8 = 2;
const QUEUE_SAMPLES: u16 = 1024;
const MAX_BUFFERED_MS: u32 = 200;

pub struct Audio {
    _sdl_context: sdl2::Sdl,
    queue: AudioQueue<i16>,
    max_buffered_bytes: u32,
}

impl Audio {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let audio = sdl_context.audio()?;
        let desired = AudioSpecDesired {
            freq: Some(DEFAULT_SAMPLE_RATE as i32),
            channels: Some(CHANNELS),
            samples: Some(QUEUE_SAMPLES),
        };

        let queue = audio.open_queue::<i16, _>(None, &desired)?;
        queue.resume();

        let bytes_per_second = DEFAULT_SAMPLE_RATE * CHANNELS as u32 * size_of::<i16>() as u32;
        let max_buffered_bytes = bytes_per_second * MAX_BUFFERED_MS / 1000;

        Ok(Self {
            _sdl_context: sdl_context,
            queue,
            max_buffered_bytes,
        })
    }

    pub fn push_samples(&mut self, samples: &[i16]) -> Result<(), String> {
        if samples.is_empty() {
            return Ok(());
        }

        if self.queue.size() > self.max_buffered_bytes {
            self.queue.clear();
        }

        self.queue.queue_audio(samples)
    }
}
