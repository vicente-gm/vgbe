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
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TasKey {
    W,
    A,
    S,
    D,
    K,
    L,
    I,
    O,
}

impl FromStr for TasKey {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "up" => Ok(TasKey::W),
            "left" => Ok(TasKey::A),
            "down" => Ok(TasKey::S),
            "right" => Ok(TasKey::D),
            "a" => Ok(TasKey::K),
            "b" => Ok(TasKey::L),
            "start" => Ok(TasKey::I),
            "select" => Ok(TasKey::O),
            _ => Err(format!("invalid TAS button `{value}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TasEvent {
    pub cycle: usize,
    pub key: TasKey,
    pub pressed: bool,
}

impl TasEvent {
    pub const fn new(cycle: usize, key: TasKey, pressed: bool) -> Self {
        Self {
            cycle,
            key,
            pressed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TasInputState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub k: bool,
    pub l: bool,
    pub i: bool,
    pub o: bool,
}

impl TasInputState {
    pub fn set(&mut self, key: TasKey, pressed: bool) -> bool {
        let old = self.is_pressed(key);

        match key {
            TasKey::W => self.w = pressed,
            TasKey::A => self.a = pressed,
            TasKey::S => self.s = pressed,
            TasKey::D => self.d = pressed,
            TasKey::K => self.k = pressed,
            TasKey::L => self.l = pressed,
            TasKey::I => self.i = pressed,
            TasKey::O => self.o = pressed,
        }

        !old && pressed
    }

    pub fn is_pressed(&self, key: TasKey) -> bool {
        match key {
            TasKey::W => self.w,
            TasKey::A => self.a,
            TasKey::S => self.s,
            TasKey::D => self.d,
            TasKey::K => self.k,
            TasKey::L => self.l,
            TasKey::I => self.i,
            TasKey::O => self.o,
        }
    }

    pub fn direction_bits(&self) -> u8 {
        active_low_nibble([self.d, self.a, self.w, self.s])
    }

    pub fn button_bits(&self) -> u8 {
        active_low_nibble([self.k, self.l, self.i, self.o])
    }
}

#[derive(Debug)]
pub enum TasLoadError {
    Io(io::Error),
    Parse { line: usize, message: String },
}

impl fmt::Display for TasLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TasLoadError::Io(error) => write!(f, "failed to load TAS file: {error}"),
            TasLoadError::Parse { line, message } => {
                write!(f, "failed to parse TAS file at line {line}: {message}")
            }
        }
    }
}

impl std::error::Error for TasLoadError {}

impl From<io::Error> for TasLoadError {
    fn from(error: io::Error) -> Self {
        TasLoadError::Io(error)
    }
}

pub fn load_tas_file<P: AsRef<Path>>(path: P) -> Result<Vec<TasEvent>, TasLoadError> {
    parse_tas_file(&fs::read_to_string(path)?)
}

pub fn parse_tas_file(contents: &str) -> Result<Vec<TasEvent>, TasLoadError> {
    let mut events = Vec::new();

    for (line_index, line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(parse_error(line_number, "expected: <cycle> <button> <0|1>"));
        }

        let cycle = parts[0]
            .parse::<usize>()
            .map_err(|_| parse_error(line_number, "cycle must be a usize"))?;
        let key = parts[1]
            .parse::<TasKey>()
            .map_err(|message| parse_error(line_number, message))?;
        let pressed = match parts[2] {
            "0" => false,
            "1" => true,
            _ => return Err(parse_error(line_number, "value must be 0 or 1")),
        };

        events.push(TasEvent::new(cycle, key, pressed));
    }

    Ok(events)
}

fn parse_error(line: usize, message: impl Into<String>) -> TasLoadError {
    TasLoadError::Parse {
        line,
        message: message.into(),
    }
}

fn active_low_nibble(bits: [bool; 4]) -> u8 {
    let mut value = 0x0F;

    for (bit, pressed) in bits.iter().enumerate() {
        if *pressed {
            value &= !(1 << bit);
        }
    }

    value
}
