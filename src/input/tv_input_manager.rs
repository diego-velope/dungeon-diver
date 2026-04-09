//! TV Input Manager for WASM builds
//!
//! This module provides a platform-agnostic input layer for TV platforms.
//! It receives input events from JavaScript and exposes
//! a simple API for the game to query input state.

use std::cell::UnsafeCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvAction {
    Up,
    Down,
    Left,
    Right,
    Action,
    Back,
}

impl TvAction {
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            TvAction::Up => 0,
            TvAction::Down => 1,
            TvAction::Left => 2,
            TvAction::Right => 3,
            TvAction::Action => 4,
            TvAction::Back => 5,
        }
    }
}

const ACTION_COUNT: usize = 6;

pub struct TvInputManager {
    current_state: [bool; ACTION_COUNT],
    previous_state: [bool; ACTION_COUNT],
    pressed_latch: [bool; ACTION_COUNT],
}

impl TvInputManager {
    pub fn new() -> Self {
        Self {
            current_state: [false; ACTION_COUNT],
            previous_state: [false; ACTION_COUNT],
            pressed_latch: [false; ACTION_COUNT],
        }
    }

    pub fn set_action(&mut self, action: TvAction, pressed: bool) {
        let i = action.index();
        self.current_state[i] = pressed;
        if pressed {
            self.pressed_latch[i] = true;
        }
    }

    pub fn update(&mut self) {
        self.previous_state.copy_from_slice(&self.current_state);
        for v in &mut self.pressed_latch {
            *v = false;
        }
    }

    pub fn is_action_pressed(&self, action: TvAction) -> bool {
        self.current_state[action.index()]
    }

    pub fn was_action_pressed(&self, action: TvAction) -> bool {
        self.pressed_latch[action.index()]
    }

    pub fn is_action_just_pressed(&self, action: TvAction) -> bool {
        let i = action.index();
        self.current_state[i] && !self.previous_state[i]
    }
}

impl Default for TvInputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
struct TvInputGlobal(UnsafeCell<Option<TvInputManager>>);

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TvInputGlobal {}

#[cfg(target_arch = "wasm32")]
static TV_INPUT_GLOBAL: TvInputGlobal = TvInputGlobal(UnsafeCell::new(None));

#[cfg(target_arch = "wasm32")]
pub fn init_tv_input_manager() {
    unsafe {
        *TV_INPUT_GLOBAL.0.get() = Some(TvInputManager::new());
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager() -> Option<&'static TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_ref() }
}

#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager_mut() -> Option<&'static mut TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_mut() }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_up"]
pub extern "C" fn mq_handle_up(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Up, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_down"]
pub extern "C" fn mq_handle_down(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Down, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_left"]
pub extern "C" fn mq_handle_left(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Left, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_right"]
pub extern "C" fn mq_handle_right(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Right, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_action"]
pub extern "C" fn mq_handle_action(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Action, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_back"]
pub extern "C" fn mq_handle_back(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Back, pressed != 0);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_tv_input_manager() {}
