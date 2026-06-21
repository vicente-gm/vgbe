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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbcType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub enum Mbc {
    RomOnly,

    Mbc1 {
        rom_bank: u8,
        ram_bank: u8,
        ram_enabled: bool,
        banking_mode: u8,
    },

    Mbc2 {
        rom_bank: u8,
        ram_enabled: bool,
    },

    Mbc3 {
        rom_bank: u8,
        ram_bank: u8,
        ram_enabled: bool,
    },

    Mbc5 {
        rom_bank: u16,
        ram_bank: u8,
        ram_enabled: bool,
    },
}
