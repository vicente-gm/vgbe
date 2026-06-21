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

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cpu::interrupts::{
    INTERRUPT_JOYPAD_BIT, INTERRUPT_LCD_BIT, INTERRUPT_SERIAL_BIT, INTERRUPT_TIMER_BIT,
    INTERRUPT_VBLANK_BIT,
};
use crate::cpu::registers::{FLAG_C, FLAG_H, FLAG_N, FLAG_Z, Reg8, Reg16};
use crate::cpu::{Cpu, ExecutedInstruction};

pub struct InstructionLogger {
    log_dir: PathBuf,
    rom_name: String,
    writer: Option<BufWriter<File>>,
    path: Option<PathBuf>,
}

impl InstructionLogger {
    pub fn new<P: AsRef<Path>>(rom_path: P) -> Self {
        Self::with_log_dir("logs", rom_path)
    }

    pub fn with_log_dir<P: AsRef<Path>, R: AsRef<Path>>(log_dir: P, rom_path: R) -> Self {
        Self {
            log_dir: log_dir.as_ref().to_path_buf(),
            rom_name: sanitize_rom_name(rom_path.as_ref()),
            writer: None,
            path: None,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn log_if_debug(&mut self, debug_mode: bool, executed_instruction: &ExecutedInstruction, cpu: &Cpu) -> io::Result<()> {
        if !debug_mode {
            return Ok(());
        }

        self.ensure_writer()?;

        let entry = Self::format_entry(executed_instruction, cpu);
        if let Some(writer) = self.writer.as_mut() {
            writer.write_all(entry.as_bytes())?;
        }

        Ok(())
    }

    pub fn log_if_debug_with_timing(
        &mut self,
        debug_mode: bool,
        executed_instruction: &ExecutedInstruction,
        cpu: &Cpu,
        cycle: usize,
        dots: u16,
    ) -> io::Result<()> {
        if !debug_mode {
            return Ok(());
        }

        self.ensure_writer()?;

        let entry = Self::format_entry_with_timing(executed_instruction, cpu, cycle, dots);
        if let Some(writer) = self.writer.as_mut() {
            writer.write_all(entry.as_bytes())?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
        }

        Ok(())
    }

    pub fn format_entry(executed_instruction: &ExecutedInstruction, cpu: &Cpu) -> String {
        let flags = cpu.get_register8(Reg8::F);
        let ie = cpu.interrupt_enable();
        let iflag = cpu.interrupt_flag();
        let pending = ie & iflag;

        format!(
            "0x{:04X}: {}\n\tA:{:02X}  F:{:02X}  B:{:02X}  C:{:02X}  D:{:02X}  E:{:02X}  H:{:02X}  L:{:02X}  PC:0x{:04X}  SP:0x{:04X}  FLAGS:Z{} N{} H{} C{}\n\tIME:{}  IE:{:02X}  IF:{:02X}  PENDING:{:02X}  VBLANK:IE{} IF{}  LCD:IE{} IF{}  TIMER:IE{} IF{}  SERIAL:IE{} IF{}  JOYPAD:IE{} IF{}\n\n",
            executed_instruction.address,
            executed_instruction.instr.asm,
            cpu.get_register8(Reg8::A),
            flags,
            cpu.get_register8(Reg8::B),
            cpu.get_register8(Reg8::C),
            cpu.get_register8(Reg8::D),
            cpu.get_register8(Reg8::E),
            cpu.get_register8(Reg8::H),
            cpu.get_register8(Reg8::L),
            cpu.get_register16(Reg16::PC),
            cpu.get_register16(Reg16::SP),
            bit_value(flags, FLAG_Z),
            bit_value(flags, FLAG_N),
            bit_value(flags, FLAG_H),
            bit_value(flags, FLAG_C),
            bool_as_bit(cpu.interrupt_master_enabled()),
            ie,
            iflag,
            pending,
            bit_at(ie, INTERRUPT_VBLANK_BIT),
            bit_at(iflag, INTERRUPT_VBLANK_BIT),
            bit_at(ie, INTERRUPT_LCD_BIT),
            bit_at(iflag, INTERRUPT_LCD_BIT),
            bit_at(ie, INTERRUPT_TIMER_BIT),
            bit_at(iflag, INTERRUPT_TIMER_BIT),
            bit_at(ie, INTERRUPT_SERIAL_BIT),
            bit_at(iflag, INTERRUPT_SERIAL_BIT),
            bit_at(ie, INTERRUPT_JOYPAD_BIT),
            bit_at(iflag, INTERRUPT_JOYPAD_BIT),
        )
    }

    pub fn format_entry_with_timing(
        executed_instruction: &ExecutedInstruction,
        cpu: &Cpu,
        cycle: usize,
        dots: u16,
    ) -> String {
        let mut entry = Self::format_entry(executed_instruction, cpu);

        if entry.ends_with("\n\n") {
            entry.truncate(entry.len() - 1);
        }

        entry.push_str(&format!("\tTIMING:M-CYCLE:{} DOTS:{}\n\n", cycle, dots));
        entry
    }

    fn ensure_writer(&mut self) -> io::Result<()> {
        if self.writer.is_some() {
            return Ok(());
        }

        create_dir_all(&self.log_dir)?;

        let timestamp = current_timestamp();
        for attempt in 0..1000 {
            let file_name = if attempt == 0 {
                format!("{}-{}.log", timestamp, self.rom_name)
            } else {
                format!("{}-{}-{}.log", timestamp, attempt, self.rom_name)
            };
            let path = self.log_dir.join(file_name);

            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    self.writer = Some(BufWriter::new(file));
                    self.path = Some(path);
                    return Ok(());
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a unique instruction log file",
        ))
    }
}

fn sanitize_rom_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("rom");

    let mut sanitized = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() {
        String::from("rom")
    } else {
        sanitized
    }
}

fn current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format_utc_timestamp(duration.as_secs())
}

fn format_utc_timestamp(seconds_since_epoch: u64) -> String {
    let days = (seconds_since_epoch / 86_400) as i64;
    let seconds_of_day = seconds_since_epoch % 86_400;

    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        year, month, day, hour, minute, second
    )
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
}

fn bit_value(value: u8, mask: u8) -> u8 {
    bool_as_bit(value & mask != 0)
}

fn bit_at(value: u8, bit: u8) -> u8 {
    bit_value(value, 1 << bit)
}

fn bool_as_bit(value: bool) -> u8 {
    if value { 1 } else { 0 }
}
