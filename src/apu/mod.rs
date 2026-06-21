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

pub mod dummy;
#[cfg(feature = "native-apu")]
pub mod gbapu;
#[cfg(feature = "native-apu")]
mod gbapu_ffi;

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

pub use dummy::DummyApu;
#[cfg(feature = "native-apu")]
pub use gbapu::GbApu;

pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
pub const DEFAULT_BUFFER_FRAMES: usize = DEFAULT_SAMPLE_RATE as usize / 10;

pub type SharedApu = Rc<RefCell<Box<dyn Apu>>>;

pub trait Apu: Debug {
    fn reset(&mut self);
    fn step(&mut self, cycles: u32);
    fn end_frame(&mut self);
    fn read_register(&mut self, address: u16) -> u8;
    fn write_register(&mut self, address: u16, value: u8);
    fn drain_samples(&mut self, output: &mut Vec<i16>);
}

pub fn new_shared_dummy_apu() -> SharedApu {
    Rc::new(RefCell::new(Box::new(DummyApu::default())))
}

pub fn new_shared_default_apu() -> SharedApu {
    #[cfg(feature = "native-apu")]
    {
        if let Some(apu) = GbApu::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_FRAMES) {
            return Rc::new(RefCell::new(Box::new(apu)));
        }
    }

    new_shared_dummy_apu()
}
