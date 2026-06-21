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

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NATIVE_APU");

    if std::env::var_os("CARGO_FEATURE_NATIVE_APU").is_none() {
        return;
    }

    println!("cargo:rerun-if-changed=native/gbapu_wrapper/gbapu_c_api.h");
    println!("cargo:rerun-if-changed=native/gbapu_wrapper/gbapu_c_api.cpp");
    println!("cargo:rerun-if-changed=external/gbapu/include/gbapu.hpp");
    println!("cargo:rerun-if-changed=external/gbapu/src/Apu.cpp");
    println!("cargo:rerun-if-changed=external/gbapu/src/_internal.cpp");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .include("native/gbapu_wrapper")
        .include("external/gbapu/include")
        .file("native/gbapu_wrapper/gbapu_c_api.cpp")
        .file("external/gbapu/src/Apu.cpp")
        .file("external/gbapu/src/_internal.cpp")
        .warnings(false);

    build.compile("vgbe_gbapu");
}
