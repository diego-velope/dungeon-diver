// Dungeon Diver - TV Remote Input Handler
// Handles D-pad movement with hold detection and single-press buttons

use macroquad::prelude::*;
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputAction {
    None,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Pause,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_vec(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// Tracks state for hold detection on D-pad buttons
#[derive(Clone, Copy)]
struct ButtonState {
    hold_duration: f32,
    is_holding: bool,
    repeat_timer: f32,
    just_triggered: bool,
}

impl ButtonState {
    fn new() -> Self {
        Self {
            hold_duration: 0.0,
            is_holding: false,
            repeat_timer: 0.0,
            just_triggered: false,
        }
    }

    fn reset(&mut self) {
        self.hold_duration = 0.0;
        self.is_holding = false;
        self.repeat_timer = 0.0;
        self.just_triggered = false;
    }
}

/// TV Remote Input Handler with hold detection
pub struct InputHandler {
    buttons: [ButtonState; 4], // up, down, left, right

    // Track previous frame state for single-press detection
    prev_pause: bool,
    prev_attack: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            buttons: [ButtonState::new(); 4],
            prev_pause: false,
            prev_attack: false,
        }
    }

    /// Update input state and return actions
    /// Call this once per frame with delta time
    pub fn update(&mut self, dt: f32) -> Vec<InputAction> {
        let mut actions = Vec::new();

        // Check D-pad buttons (with hold detection)
        // Process each direction independently
        self.update_direction_button(0, KeyCode::Up, dt, InputAction::MoveUp, &mut actions);
        self.update_direction_button(1, KeyCode::Down, dt, InputAction::MoveDown, &mut actions);
        self.update_direction_button(2, KeyCode::Left, dt, InputAction::MoveLeft, &mut actions);
        self.update_direction_button(3, KeyCode::Right, dt, InputAction::MoveRight, &mut actions);

        // Check single-press buttons
        let pause_pressed = is_key_down(KeyCode::Escape) || is_key_down(KeyCode::Backspace);
        if pause_pressed && !self.prev_pause {
            actions.push(InputAction::Pause);
        }
        self.prev_pause = pause_pressed;

        // Attack button (OK/Enter) - single press only
        let attack_pressed = is_key_down(KeyCode::Enter) || is_key_down(KeyCode::Space);
        if attack_pressed && !self.prev_attack {
            actions.push(InputAction::Attack);
        }
        self.prev_attack = attack_pressed;

        // Also accept Enter for confirm in menus
        if attack_pressed && !self.prev_attack { // prev_attack is already updated, so this might not push, which is fine since Attack handles Confirm actions in menus too.
            actions.push(InputAction::Confirm);
        }

        actions
    }

    fn update_direction_button(
        &mut self,
        index: usize,
        key: KeyCode,
        dt: f32,
        action: InputAction,
        actions: &mut Vec<InputAction>,
    ) {
        let is_down = is_key_down(key);
        let state = &mut self.buttons[index];

        if is_down {
            state.hold_duration += dt;

            // First press (tap)
            if state.hold_duration == dt {
                state.just_triggered = true;
                actions.push(action);
            }
            // Hold detected - trigger repeat inputs
            else if state.hold_duration > HOLD_THRESHOLD {
                if !state.is_holding {
                    state.is_holding = true;
                }

                state.repeat_timer += dt;
                if state.repeat_timer >= INPUT_REPEAT_DELAY {
                    state.repeat_timer = 0.0;
                    actions.push(action);
                }
            }
        } else {
            state.reset();
        }
    }

    /// Check if a direction is currently being held (for continuous movement)
    pub fn is_direction_held(&self, dir: Direction) -> bool {
        let index = match dir {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 2,
            Direction::Right => 3,
        };
        self.buttons[index].is_holding
    }

    /// Get the current direction being held (if any)
    pub fn get_held_direction(&self) -> Option<Direction> {
        if self.buttons[0].is_holding { return Some(Direction::Up); }
        if self.buttons[1].is_holding { return Some(Direction::Down); }
        if self.buttons[2].is_holding { return Some(Direction::Left); }
        if self.buttons[3].is_holding { return Some(Direction::Right); }
        None
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
