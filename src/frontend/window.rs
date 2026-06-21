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

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::MouseButton;
use sdl2::rect::{Point, Rect};

use std::time::{Duration, Instant};

use crate::cpu::Cpu;
use crate::frontend::input::Input;
use crate::frontend::renderer::Renderer;
use crate::ppu::Ppu;
use crate::ppu::timing::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::tas::TasKey;

pub struct Window {
    renderer: Renderer,
    input: Input,
    event_pump: sdl2::EventPump,

    debug_mode: bool,

    last_fps_time: Instant,
    frame_counter: u32,
    fps: u32,

    paused: bool,
    step_requested: bool,
    needs_redraw: bool,
    joypad_interrupt_requested: bool,
    tas_key_events: Vec<(TasKey, bool)>,
}

impl Window {
    pub const SCALE_FACTOR: u32 = 5;
    pub const DEBUG_SCALE_FACTOR: u32 = 2;

    pub const SCREEN_WIDTH: u32 = 160 * Self::SCALE_FACTOR;
    pub const SCREEN_HEIGHT: u32 = 144 * Self::SCALE_FACTOR;

    pub const DEBUG_SCREEN_WIDTH: u32 = 160 * Self::DEBUG_SCALE_FACTOR;
    pub const DEBUG_SCREEN_HEIGHT: u32 = 144 * Self::DEBUG_SCALE_FACTOR;

    pub const STATUS_HEIGHT: u32 = 24;

    pub const DEBUG_SIDE_MARGIN: u32 = 10;
    pub const DEBUG_PANEL_GAP: u32 = 10;

    pub const DEBUG_LEFT_COLUMN_WIDTH: u32 = 340;
    pub const DEBUG_CENTER_COLUMN_WIDTH: u32 = 500;
    pub const DEBUG_RIGHT_COLUMN_WIDTH: u32 = 300;
    pub const DEBUG_CONTENT_HEIGHT: u32 = 734;

    pub const DEBUG_LEFT_COLUMN_X: i32 = Self::DEBUG_SIDE_MARGIN as i32;
    pub const DEBUG_CENTER_COLUMN_X: i32 = Self::DEBUG_LEFT_COLUMN_X
        + Self::DEBUG_LEFT_COLUMN_WIDTH as i32
        + Self::DEBUG_PANEL_GAP as i32;
    pub const DEBUG_RIGHT_COLUMN_X: i32 = Self::DEBUG_CENTER_COLUMN_X
        + Self::DEBUG_CENTER_COLUMN_WIDTH as i32
        + Self::DEBUG_PANEL_GAP as i32;
    pub const DEBUG_TOP_Y: i32 = Self::DEBUG_SIDE_MARGIN as i32;

    pub const DEBUG_SCREEN_PANEL_HEIGHT: u32 = 330;
    pub const DEBUG_TILES_PANEL_HEIGHT: u32 = Self::DEBUG_CONTENT_HEIGHT
        - Self::DEBUG_SIDE_MARGIN * 2
        - Self::DEBUG_PANEL_GAP
        - Self::DEBUG_SCREEN_PANEL_HEIGHT;

    pub const DEBUG_CPU_PANEL_HEIGHT: u32 = 164;

    pub const DEBUG_DISASSEMBLY_HEADER_HEIGHT: u32 = 34;
    pub const DEBUG_DISASSEMBLY_LINE_HEIGHT: u32 = 22;
    pub const DEBUG_DISASSEMBLY_BOTTOM_PADDING: u32 = 10;

    pub const DEBUG_BUTTON_GAP: u32 = 10;
    pub const DEBUG_BUTTON_HEIGHT: u32 = 36;

    pub const DEBUG_DISASSEMBLY_PANEL_HEIGHT: u32 = Self::DEBUG_CONTENT_HEIGHT
        - Self::DEBUG_SIDE_MARGIN * 2
        - Self::DEBUG_CPU_PANEL_HEIGHT
        - Self::DEBUG_PANEL_GAP * 2
        - Self::DEBUG_BUTTON_HEIGHT;

    pub const DEBUG_TILEMAP_PANEL_HEIGHT: u32 = (Self::DEBUG_CONTENT_HEIGHT
        - Self::DEBUG_SIDE_MARGIN * 2
        - Self::DEBUG_PANEL_GAP)
        / 2;

    pub const WINDOW_WIDTH: u32 = Self::DEBUG_SIDE_MARGIN
        + Self::DEBUG_LEFT_COLUMN_WIDTH
        + Self::DEBUG_PANEL_GAP
        + Self::DEBUG_CENTER_COLUMN_WIDTH
        + Self::DEBUG_PANEL_GAP
        + Self::DEBUG_RIGHT_COLUMN_WIDTH
        + Self::DEBUG_SIDE_MARGIN;

    pub const CONTENT_HEIGHT: u32 = Self::DEBUG_CONTENT_HEIGHT;

    pub const fn window_width(debug: bool) -> u32 {
        if debug {
            Self::WINDOW_WIDTH
        } else {
            Self::SCREEN_WIDTH
        }
    }

    pub const fn content_height(debug: bool) -> u32 {
        if debug {
            Self::CONTENT_HEIGHT
        } else {
            Self::SCREEN_HEIGHT
        }
    }

    pub const fn window_height(debug: bool) -> u32 {
        Self::content_height(debug) + Self::STATUS_HEIGHT
    }

    pub fn new(debug: bool) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        let window = video
            .window(
                "VGBE",
                Self::window_width(debug),
                Self::window_height(debug),
            )
            .position_centered()
            .build()
            .unwrap();

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            renderer: Renderer::new(canvas),
            input: Input::new(),
            event_pump,

            debug_mode: debug,

            paused: debug,
            step_requested: false,
            needs_redraw: true,
            joypad_interrupt_requested: false,
            tas_key_events: Vec::new(),

            last_fps_time: Instant::now(),
            frame_counter: 0,
            fps: 0,
        }
    }

    pub fn handle_events(&mut self) -> bool {
        while let Some(event) = self.event_pump.poll_event() {
            if let Some((key, pressed)) = Self::tas_key_event_from_event(&event) {
                self.tas_key_events.push((key, pressed));
                if pressed {
                    self.joypad_interrupt_requested = true;
                }
                continue;
            }

            match event {
                Event::Quit { .. } => return false,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,

                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => {
                    self.debug_mode = !self.debug_mode;
                    self.renderer.set_window_size(
                        Self::window_width(self.debug_mode),
                        Self::window_height(self.debug_mode),
                    );
                    self.needs_redraw = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => self.handle_debug_button_click(DebugButton::Continue),

                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => self.handle_debug_button_click(DebugButton::Next),

                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => self.handle_debug_button_click(DebugButton::Pause),

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    if let Some(button) = self.debug_button_at(x, y) {
                        self.handle_debug_button_click(button);
                    }
                }

                _ => self.input.handle_event(event),
            }
        }

        true
    }

    pub fn record_frame(&mut self) {
        self.frame_counter += 1;
        self.update_fps();
    }

    fn update_fps(&mut self) {
        let elapsed = self.last_fps_time.elapsed();

        if elapsed >= Duration::from_secs(1) {
            self.fps = (self.frame_counter as f32 / elapsed.as_secs_f32()).round() as u32;
            self.frame_counter = 0;
            self.last_fps_time = Instant::now();
        }
    }

    pub fn render(&mut self, cpu: &Cpu, ppu: &Ppu) {
        self.render_dirty_with_cycle(cpu, ppu, 0, true);
    }

    pub fn render_dirty(&mut self, cpu: &Cpu, ppu: &Ppu, redraw_screen: bool) {
        self.render_dirty_with_cycle(cpu, ppu, 0, redraw_screen);
    }

    pub fn render_dirty_with_cycle(
        &mut self,
        cpu: &Cpu,
        ppu: &Ppu,
        cycle: usize,
        redraw_screen: bool,
    ) {
        self.update_fps();

        self.renderer.render(
            cpu,
            ppu,
            self.debug_mode,
            self.fps,
            cycle,
            redraw_screen,
        );

        self.needs_redraw = false;
    }

    pub fn render_framebuffer(&mut self, framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        self.update_fps();

        self.renderer.render_framebuffer(
            framebuffer,
            self.debug_mode,
            self.fps,
        );

        self.needs_redraw = false;
    }

    pub fn render_framebuffer_with_state(
        &mut self,
        cpu: &Cpu,
        ppu: &Ppu,
        framebuffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    ) {
        self.update_fps();

        self.renderer.render_framebuffer_with_state(
            cpu,
            ppu,
            framebuffer,
            self.debug_mode,
            self.fps,
        );

        self.needs_redraw = false;
    }

    pub fn should_tick(&self) -> bool {
        !self.paused || self.step_requested
    }

    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    pub fn finish_tick(&mut self) {
        if self.step_requested {
            self.step_requested = false;
            self.paused = true;
            self.needs_redraw = true;
        }
    }

    pub fn should_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub fn take_joypad_interrupt_requested(&mut self) -> bool {
        if self.joypad_interrupt_requested {
            self.joypad_interrupt_requested = false;
            true
        } else {
            false
        }
    }

    pub fn take_tas_key_events(&mut self) -> Vec<(TasKey, bool)> {
        self.tas_key_events.drain(..).collect()
    }

    fn tas_key_from_keycode(keycode: Keycode) -> Option<TasKey> {
        match keycode {
            Keycode::W | Keycode::Up => Some(TasKey::W),
            Keycode::A | Keycode::Left => Some(TasKey::A),
            Keycode::S | Keycode::Down => Some(TasKey::S),
            Keycode::D | Keycode::Right => Some(TasKey::D),
            Keycode::K => Some(TasKey::K),
            Keycode::L => Some(TasKey::L),
            Keycode::I => Some(TasKey::I),
            Keycode::O => Some(TasKey::O),
            _ => None,
        }
    }

    fn tas_key_from_key_event(
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
    ) -> Option<TasKey> {
        keycode
            .and_then(Self::tas_key_from_keycode)
            .or_else(|| scancode.and_then(Self::tas_key_from_scancode))
    }

    fn tas_key_event_from_event(event: &Event) -> Option<(TasKey, bool)> {
        match event {
            Event::KeyDown {
                keycode,
                scancode,
                repeat: false,
                ..
            } => Self::tas_key_from_key_event(*keycode, *scancode).map(|key| (key, true)),

            Event::KeyUp {
                keycode,
                scancode,
                repeat: false,
                ..
            } => Self::tas_key_from_key_event(*keycode, *scancode).map(|key| (key, false)),

            _ => None,
        }
    }

    fn tas_key_from_scancode(scancode: Scancode) -> Option<TasKey> {
        match scancode {
            Scancode::W | Scancode::Up => Some(TasKey::W),
            Scancode::A | Scancode::Left => Some(TasKey::A),
            Scancode::S | Scancode::Down => Some(TasKey::S),
            Scancode::D | Scancode::Right => Some(TasKey::D),
            Scancode::K => Some(TasKey::K),
            Scancode::L => Some(TasKey::L),
            Scancode::I => Some(TasKey::I),
            Scancode::O => Some(TasKey::O),
            _ => None,
        }
    }

    fn debug_button_at(&self, x: i32, y: i32) -> Option<DebugButton> {
        if !self.debug_mode {
            return None;
        }

        let point = Point::new(x, y);

        let (pause_rect, continue_rect, next_rect) = Self::debug_button_rects();

        if pause_rect.contains_point(point) {
            Some(DebugButton::Pause)
        } else if continue_rect.contains_point(point) {
            Some(DebugButton::Continue)
        } else if next_rect.contains_point(point) {
            Some(DebugButton::Next)
        } else {
            None
        }
    }

    fn handle_debug_button_click(&mut self, button: DebugButton) {
        match button {
            DebugButton::Pause => {
                self.paused = true;
                self.step_requested = false;
                self.needs_redraw = true;
            }

            DebugButton::Continue => {
                self.paused = false;
                self.step_requested = false;
                self.needs_redraw = true;
            }

            DebugButton::Next => {
                self.paused = true;
                self.step_requested = true;
                self.needs_redraw = true;
            }
        }
    }

    fn debug_button_rects() -> (Rect, Rect, Rect) {
        let buttons_y = Self::DEBUG_CONTENT_HEIGHT as i32
            - Self::DEBUG_SIDE_MARGIN as i32
            - Self::DEBUG_BUTTON_HEIGHT as i32;
        let button_width = (Self::DEBUG_CENTER_COLUMN_WIDTH - Self::DEBUG_BUTTON_GAP * 2) / 3;

        let pause_rect = Rect::new(
            Self::DEBUG_CENTER_COLUMN_X,
            buttons_y,
            button_width,
            Self::DEBUG_BUTTON_HEIGHT,
        );

        let continue_rect = Rect::new(
            Self::DEBUG_CENTER_COLUMN_X + (button_width + Self::DEBUG_BUTTON_GAP) as i32,
            buttons_y,
            button_width,
            Self::DEBUG_BUTTON_HEIGHT,
        );

        let next_rect = Rect::new(
            Self::DEBUG_CENTER_COLUMN_X + ((button_width + Self::DEBUG_BUTTON_GAP) * 2) as i32,
            buttons_y,
            button_width,
            Self::DEBUG_BUTTON_HEIGHT,
        );

        (pause_rect, continue_rect, next_rect)
    }

    pub fn is_step_requested(&self) -> bool {
        self.step_requested
    }
}

#[derive(Debug, Copy, Clone)]
enum DebugButton {
    Pause,
    Continue,
    Next,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::keyboard::Mod;

    fn key_down_event(keycode: Option<Keycode>, scancode: Option<Scancode>, repeat: bool) -> Event {
        Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode,
            scancode,
            keymod: Mod::NOMOD,
            repeat,
        }
    }

    fn key_up_event(keycode: Option<Keycode>, scancode: Option<Scancode>) -> Event {
        Event::KeyUp {
            timestamp: 0,
            window_id: 0,
            keycode,
            scancode,
            keymod: Mod::NOMOD,
            repeat: false,
        }
    }

    #[test]
    fn normal_mode_window_excludes_removed_top_menu_bar() {
        assert_eq!(Window::window_width(false), 800);
        assert_eq!(Window::window_height(false), 744);
    }

    #[test]
    fn debug_mode_window_excludes_removed_top_menu_bar() {
        assert_eq!(Window::window_width(true), 1180);
        assert_eq!(Window::window_height(true), 758);
    }

    #[test]
    fn debug_buttons_are_packed_under_disassembly() {
        let (pause, continue_button, next) = Window::debug_button_rects();

        assert_eq!(pause, Rect::new(360, 688, 160, 36));
        assert_eq!(continue_button, Rect::new(530, 688, 160, 36));
        assert_eq!(next, Rect::new(700, 688, 160, 36));
    }

    #[test]
    fn tas_key_event_recognizes_w_key_down_and_up_events() {
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(Some(Keycode::W), None, false)),
            Some((TasKey::W, true))
        );
        assert_eq!(
            Window::tas_key_event_from_event(&key_up_event(Some(Keycode::W), None)),
            Some((TasKey::W, false))
        );
    }

    #[test]
    fn tas_key_event_recognizes_physical_w_key() {
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(None, Some(Scancode::W), false)),
            Some((TasKey::W, true))
        );
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(
                Some(Keycode::Z),
                Some(Scancode::W),
                false,
            )),
            Some((TasKey::W, true))
        );
    }

    #[test]
    fn tas_key_event_recognizes_arrow_key_aliases() {
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(Some(Keycode::Up), None, false)),
            Some((TasKey::W, true))
        );
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(None, Some(Scancode::Up), false)),
            Some((TasKey::W, true))
        );
    }

    #[test]
    fn tas_key_event_ignores_repeated_key_down_events() {
        assert_eq!(
            Window::tas_key_event_from_event(&key_down_event(Some(Keycode::W), None, true)),
            None
        );
    }
}
