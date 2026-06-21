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

#include "gbapu_c_api.h"

#include "gbapu.hpp"

#include <cmath>
#include <cstdint>
#include <limits>
#include <vector>

struct VgbeGbApu {
    explicit VgbeGbApu(uint32_t sample_rate, size_t buffer_frames)
        : apu(sample_rate, buffer_frames) {}

    gbapu::Apu apu;
};

namespace {

uint8_t register_from_address(uint16_t address) noexcept {
    return static_cast<uint8_t>(address - 0xFF00);
}

int16_t float_to_i16(float sample) noexcept {
    if (!std::isfinite(sample)) {
        return 0;
    }

    if (sample >= 1.0f) {
        return std::numeric_limits<int16_t>::max();
    }

    if (sample <= -1.0f) {
        return std::numeric_limits<int16_t>::min();
    }

    return static_cast<int16_t>(sample * 32767.0f);
}

} // namespace

extern "C" {

VgbeGbApu *vgbe_gbapu_create(uint32_t sample_rate, size_t buffer_frames) noexcept {
    try {
        return new VgbeGbApu(sample_rate, buffer_frames);
    } catch (...) {
        return nullptr;
    }
}

void vgbe_gbapu_destroy(VgbeGbApu *apu) noexcept {
    delete apu;
}

void vgbe_gbapu_reset(VgbeGbApu *apu) noexcept {
    try {
        if (apu == nullptr) {
            return;
        }

        apu->apu.reset();
    } catch (...) {
    }
}

void vgbe_gbapu_step(VgbeGbApu *apu, uint32_t cycles) noexcept {
    try {
        if (apu == nullptr) {
            return;
        }

        apu->apu.step(cycles);
    } catch (...) {
    }
}

void vgbe_gbapu_end_frame(VgbeGbApu *apu) noexcept {
    try {
        if (apu == nullptr) {
            return;
        }

        apu->apu.endFrame();
    } catch (...) {
    }
}

uint8_t vgbe_gbapu_read_register(VgbeGbApu *apu, uint16_t address) noexcept {
    try {
        if (apu == nullptr) {
            return 0xFF;
        }

        return apu->apu.readRegister(register_from_address(address), 0);
    } catch (...) {
        return 0xFF;
    }
}

void vgbe_gbapu_write_register(VgbeGbApu *apu, uint16_t address, uint8_t value) noexcept {
    try {
        if (apu == nullptr) {
            return;
        }

        apu->apu.writeRegister(register_from_address(address), value, 0);
    } catch (...) {
    }
}

size_t vgbe_gbapu_available_sample_frames(VgbeGbApu *apu) noexcept {
    try {
        if (apu == nullptr) {
            return 0;
        }

        return apu->apu.availableSamples();
    } catch (...) {
        return 0;
    }
}

size_t vgbe_gbapu_read_samples_i16(VgbeGbApu *apu, int16_t *dest, size_t dest_frames) noexcept {
    if (apu == nullptr || dest == nullptr || dest_frames == 0) {
        return 0;
    }

    try {
        std::vector<float> float_samples(dest_frames * 2);
        size_t frames_read = apu->apu.readSamples(float_samples.data(), dest_frames);
        size_t samples_read = frames_read * 2;

        for (size_t i = 0; i < samples_read; ++i) {
            dest[i] = float_to_i16(float_samples[i]);
        }

        return frames_read;
    } catch (...) {
        return 0;
    }
}

}
