use crate::input::InputAction;

#[derive(Debug, Clone)]
pub struct GameSettings {
    pub master_volume: u8,
    pub music_volume: u8,
    pub effects_volume: u8,
    pub game_speed: u8,
    pub focused_row: usize,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 5,
            music_volume: 10,
            effects_volume: 10,
            game_speed: 5,
            focused_row: 0,
        }
    }
}

impl GameSettings {
    /// Master × music (0–1 each): master gates everything; music slider only affects music tracks.
    pub fn effective_music_volume(&self) -> f32 {
        (self.master_volume as f32 / 10.0) * (self.music_volume as f32 / 10.0)
    }

    /// Master × effects: FX slider only affects SFX (coins, potions, etc.).
    pub fn effective_effects_volume(&self) -> f32 {
        (self.master_volume as f32 / 10.0) * (self.effects_volume as f32 / 10.0)
    }

    pub fn speed_multiplier(&self) -> f32 {
        self.game_speed as f32 / 5.0
    }

    pub fn handle_options_input(&mut self, actions: &[InputAction]) -> bool {
        for &action in actions {
            match action {
                InputAction::Cancel | InputAction::Pause => return true,
                InputAction::MoveUp => {
                    if self.focused_row > 0 {
                        self.focused_row -= 1;
                    }
                }
                InputAction::MoveDown => {
                    if self.focused_row < 3 {
                        self.focused_row += 1;
                    }
                }
                InputAction::MoveLeft => match self.focused_row {
                    0 => self.master_volume = self.master_volume.saturating_sub(1),
                    1 => self.music_volume = self.music_volume.saturating_sub(1),
                    2 => self.effects_volume = self.effects_volume.saturating_sub(1),
                    3 => {
                        if self.game_speed > 1 {
                            self.game_speed -= 1;
                        }
                    }
                    _ => {}
                },
                InputAction::MoveRight => match self.focused_row {
                    0 => self.master_volume = (self.master_volume + 1).min(10),
                    1 => self.music_volume = (self.music_volume + 1).min(10),
                    2 => self.effects_volume = (self.effects_volume + 1).min(10),
                    3 => self.game_speed = (self.game_speed + 1).min(10),
                    _ => {}
                },
                _ => {}
            }
        }
        false
    }
}
