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

use std::fmt;
use std::ptr::NonNull;

use super::Apu;
use super::gbapu_ffi;

pub struct GbApu {
    handle: NonNull<gbapu_ffi::RawGbApu>,
}

impl GbApu {
    pub fn new(sample_rate: u32, buffer_frames: usize) -> Option<Self> {
        let handle = unsafe { gbapu_ffi::vgbe_gbapu_create(sample_rate, buffer_frames) };
        NonNull::new(handle).map(|handle| Self { handle })
    }

    fn as_ptr(&self) -> *mut gbapu_ffi::RawGbApu {
        self.handle.as_ptr()
    }
}

impl fmt::Debug for GbApu {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GbApu").finish_non_exhaustive()
    }
}

impl Drop for GbApu {
    fn drop(&mut self) {
        unsafe { gbapu_ffi::vgbe_gbapu_destroy(self.as_ptr()) }
    }
}

impl Apu for GbApu {
    fn reset(&mut self) {
        unsafe { gbapu_ffi::vgbe_gbapu_reset(self.as_ptr()) }
    }

    fn step(&mut self, cycles: u32) {
        unsafe { gbapu_ffi::vgbe_gbapu_step(self.as_ptr(), cycles) }
    }

    fn end_frame(&mut self) {
        unsafe { gbapu_ffi::vgbe_gbapu_end_frame(self.as_ptr()) }
    }

    fn read_register(&mut self, address: u16) -> u8 {
        unsafe { gbapu_ffi::vgbe_gbapu_read_register(self.as_ptr(), address) }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        unsafe { gbapu_ffi::vgbe_gbapu_write_register(self.as_ptr(), address, value) }
    }

    fn drain_samples(&mut self, output: &mut Vec<i16>) {
        let frames = unsafe { gbapu_ffi::vgbe_gbapu_available_sample_frames(self.as_ptr()) };
        if frames == 0 {
            return;
        }

        let start = output.len();
        output.resize(start + frames * 2, 0);

        let frames_read = unsafe {
            gbapu_ffi::vgbe_gbapu_read_samples_i16(
                self.as_ptr(),
                output[start..].as_mut_ptr(),
                frames,
            )
        };

        output.truncate(start + frames_read * 2);
    }
}
