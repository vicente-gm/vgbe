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

#[repr(C)]
pub struct RawGbApu {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn vgbe_gbapu_create(sample_rate: u32, buffer_frames: usize) -> *mut RawGbApu;
    pub fn vgbe_gbapu_destroy(apu: *mut RawGbApu);

    pub fn vgbe_gbapu_reset(apu: *mut RawGbApu);
    pub fn vgbe_gbapu_step(apu: *mut RawGbApu, cycles: u32);
    pub fn vgbe_gbapu_end_frame(apu: *mut RawGbApu);

    pub fn vgbe_gbapu_read_register(apu: *mut RawGbApu, address: u16) -> u8;
    pub fn vgbe_gbapu_write_register(apu: *mut RawGbApu, address: u16, value: u8);

    pub fn vgbe_gbapu_available_sample_frames(apu: *mut RawGbApu) -> usize;
    pub fn vgbe_gbapu_read_samples_i16(
        apu: *mut RawGbApu,
        dest: *mut i16,
        dest_frames: usize,
    ) -> usize;
}
