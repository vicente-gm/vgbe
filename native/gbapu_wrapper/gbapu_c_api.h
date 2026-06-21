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

#ifndef VGBE_GBAPU_C_API_H
#define VGBE_GBAPU_C_API_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
#define VGBE_NOEXCEPT noexcept
extern "C" {
#else
#define VGBE_NOEXCEPT
#endif

typedef struct VgbeGbApu VgbeGbApu;

VgbeGbApu *vgbe_gbapu_create(uint32_t sample_rate, size_t buffer_frames) VGBE_NOEXCEPT;
void vgbe_gbapu_destroy(VgbeGbApu *apu) VGBE_NOEXCEPT;

void vgbe_gbapu_reset(VgbeGbApu *apu) VGBE_NOEXCEPT;
void vgbe_gbapu_step(VgbeGbApu *apu, uint32_t cycles) VGBE_NOEXCEPT;
void vgbe_gbapu_end_frame(VgbeGbApu *apu) VGBE_NOEXCEPT;

uint8_t vgbe_gbapu_read_register(VgbeGbApu *apu, uint16_t address) VGBE_NOEXCEPT;
void vgbe_gbapu_write_register(VgbeGbApu *apu, uint16_t address, uint8_t value) VGBE_NOEXCEPT;

size_t vgbe_gbapu_available_sample_frames(VgbeGbApu *apu) VGBE_NOEXCEPT;

// Writes up to dest_frames stereo frames to dest as interleaved signed i16 samples.
// The destination must have room for dest_frames * 2 i16 values.
size_t vgbe_gbapu_read_samples_i16(VgbeGbApu *apu, int16_t *dest, size_t dest_frames) VGBE_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#undef VGBE_NOEXCEPT

#endif
