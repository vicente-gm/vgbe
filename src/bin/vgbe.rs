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

use std::env;
use std::fs::File;

use vgbe::run;

#[derive(Debug, PartialEq, Eq)]
struct Args {
    rom: String,
    debug: bool,
    tas_file: Option<String>,
}

fn main() {
    env_logger::init();

    let program = env::args().next().unwrap_or_else(|| "vgbe".to_string());
    let args = match parse_args(env::args()) {
        Ok(args) => args,
        Err(message) => {
            log::error!("{}", message);
            log::info!("Usage: {} <rom> [--debug=true|false] [--tas=<file>]", program);
            std::process::exit(1);
        }
    };

    log::info!("Received ROM: {}", args.rom);

    let mut file = File::open(&args.rom).unwrap_or_else(|_| panic!("Could not open ROM file {}", args.rom));

    if let Err(e) = run(&mut file, &args.rom, args.debug, args.tas_file.as_deref()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut rom = None;
    let mut debug = true;
    let mut tas_file = None;
    let args = args.into_iter().skip(1);

    for arg in args {
        if let Some(value) = arg.strip_prefix("--debug=") {
            debug = parse_bool(value)?;
        } else if let Some(value) = arg.strip_prefix("--tas=") {
            set_tas_file(&mut tas_file, value)?;
        } else if arg.starts_with("--") {
            return Err(format!("Unknown argument: {}", arg));
        } else if rom.replace(arg).is_some() {
            return Err("Expected exactly one ROM path".to_string());
        }
    }

    Ok(Args {
        rom: rom.ok_or_else(|| "Missing ROM path".to_string())?,
        debug,
        tas_file,
    })
}

fn set_tas_file(tas_file: &mut Option<String>, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("TAS file path cannot be empty".to_string());
    }

    if tas_file.replace(value.to_string()).is_some() {
        return Err("Expected at most one TAS file".to_string());
    }

    Ok(())
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("Invalid debug value: {}", value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parse_args_defaults_to_debug_mode() {
        assert_eq!(
            parse_args(args(&["vgbe", "roms/Pokemon-Amarillo.gb"])).unwrap(),
            Args {
                rom: "roms/Pokemon-Amarillo.gb".to_string(),
                debug: true,
                tas_file: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_play_mode() {
        assert_eq!(
            parse_args(args(&["vgbe", "roms/Pokemon-Amarillo.gb", "--debug=false"])).unwrap(),
            Args {
                rom: "roms/Pokemon-Amarillo.gb".to_string(),
                debug: false,
                tas_file: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_tas_file_with_equals() {
        assert_eq!(
            parse_args(args(&[
                "vgbe",
                "roms/Pokemon-Amarillo.gb",
                "--tas=inputs/demo.tas",
            ]))
            .unwrap(),
            Args {
                rom: "roms/Pokemon-Amarillo.gb".to_string(),
                debug: true,
                tas_file: Some("inputs/demo.tas".to_string()),
            }
        );
    }

    #[test]
    fn parse_args_rejects_duplicate_tas_files() {
        let result = parse_args(args(&[
            "vgbe",
            "roms/Pokemon-Amarillo.gb",
            "--tas=a.tas",
            "--tas=b.tas",
        ]));

        assert_eq!(result, Err("Expected at most one TAS file".to_string()));
    }

    #[test]
    fn parse_args_rejects_tas_without_equals() {
        let result = parse_args(args(&[
            "vgbe",
            "roms/Pokemon-Amarillo.gb",
            "--tas",
            "inputs/demo.tas",
        ]));

        assert_eq!(result, Err("Unknown argument: --tas".to_string()));
    }

    #[test]
    fn parse_args_rejects_tas_file_alias() {
        let result = parse_args(args(&[
            "vgbe",
            "roms/Pokemon-Amarillo.gb",
            "--tas-file=inputs/demo.tas",
        ]));

        assert_eq!(
            result,
            Err("Unknown argument: --tas-file=inputs/demo.tas".to_string())
        );
    }

    #[test]
    fn parse_args_rejects_tas_file_alias_without_equals() {
        let result = parse_args(args(&[
            "vgbe",
            "roms/Pokemon-Amarillo.gb",
            "--tas-file",
            "inputs/demo.tas",
        ]));

        assert_eq!(result, Err("Unknown argument: --tas-file".to_string()));
    }
}
