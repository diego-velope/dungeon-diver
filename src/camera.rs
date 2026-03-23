// Dungeon Diver - Camera System
// Smooth camera following the player

use macroquad::prelude::*;
use crate::constants::*;

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub shake_time: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            shake_time: 0.0,
        }
    }

    /// Set the target position (player position)
    pub fn set_target(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
    }

    /// Update camera position with smooth following
    pub fn update(&mut self, dt: f32) {
        // Smooth follow using lerp
        self.x += (self.target_x - self.x - SCREEN_W / 2.0) * CAMERA_SMOOTH;
        self.y += (self.target_y - self.y - SCREEN_H / 2.0) * CAMERA_SMOOTH;

        // Update screen shake
        if self.shake_time > 0.0 {
            self.shake_time -= dt;
        }
    }

    /// Start screen shake
    pub fn shake(&mut self) {
        self.shake_time = SCREEN_SHAKE_DURATION;
    }

    /// Get the actual render position with shake applied
    pub fn get_render_offset(&self) -> (f32, f32) {
        if self.shake_time > 0.0 {
            let shake_amount = self.shake_time / SCREEN_SHAKE_DURATION * SCREEN_SHAKE_INTENSITY;
            let shake_x = rand::gen_range(-shake_amount, shake_amount);
            let shake_y = rand::gen_range(-shake_amount, shake_amount);
            (self.x + shake_x, self.y + shake_y)
        } else {
            (self.x, self.y)
        }
    }

    /// Clamp camera to level bounds
    /// Handles levels smaller than the screen by centering them
    pub fn clamp_to_level(&mut self, level_w: f32, level_h: f32) {
        // If level is smaller than screen, center it
        if level_w < SCREEN_W {
            self.x = -(SCREEN_W - level_w) / 2.0;
        } else {
            if self.x < 0.0 { self.x = 0.0; }
            if self.x > level_w - SCREEN_W { self.x = level_w - SCREEN_W; }
        }

        if level_h < SCREEN_H {
            self.y = -(SCREEN_H - level_h) / 2.0;
        } else {
            if self.y < 0.0 { self.y = 0.0; }
            if self.y > level_h - SCREEN_H { self.y = level_h - SCREEN_H; }
        }

        // Clamp target to keep player in reasonable bounds
        let min_target_x = if level_w < SCREEN_W { level_w / 2.0 } else { SCREEN_W / 2.0 };
        let max_target_x = if level_w < SCREEN_W { level_w / 2.0 } else { level_w - SCREEN_W / 2.0 };
        let min_target_y = if level_h < SCREEN_H { level_h / 2.0 } else { SCREEN_H / 2.0 };
        let max_target_y = if level_h < SCREEN_H { level_h / 2.0 } else { level_h - SCREEN_H };

        if self.target_x < min_target_x { self.target_x = min_target_x; }
        if self.target_x > max_target_x { self.target_x = max_target_x; }
        if self.target_y < min_target_y { self.target_y = min_target_y; }
        if self.target_y > max_target_y { self.target_y = max_target_y; }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
