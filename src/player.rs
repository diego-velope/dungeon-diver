// Dungeon Diver - Player Entity
// Handles player movement, animation, and combat

use macroquad::prelude::*;
use crate::constants::*;
use crate::input::*;
use crate::level::Level;

/// Animation states for the player
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimState {
    Idle,
    Run,
    Attack,
    Hurt,
    Death,
}

/// Player entity
pub struct Player {
    // Grid position (tile coordinates)
    pub grid_x: i32,
    pub grid_y: i32,

    // Pixel position (for smooth rendering)
    pub x: f32,
    pub y: f32,

    // Movement
    pub facing: Direction,
    pub move_progress: f32,  // 0.0 to 1.0 between tiles
    pub is_moving: bool,

    // Combat
    pub hp: i32,
    pub max_hp: i32,
    pub lives: i32,
    /// Set to true after opening the Level 1 chest and receiving the key.
    pub has_key: bool,
    pub attack_cooldown: f32,
    pub is_attacking: bool,
    pub invincible_time: f32,

    // Animation
    pub anim_state: AnimState,
    pub anim_frame: u32,
    pub anim_timer: f32,

    // Sprites (loaded from assets)
    idle_sprite: Option<Texture2D>,
    run_sprite: Option<Texture2D>,
    /// Frames in idle strip (derived from texture width ÷ KNIGHT_IDLE_FRAME_W when sprites load)
    idle_frame_count: u32,
    run_frame_count: u32,
}

impl Player {
    /// Create a new player at grid position
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        let x = grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

        Self {
            grid_x,
            grid_y,
            x,
            y,
            facing: Direction::Down,  // start facing camera (idle sheet row 0)
            move_progress: 0.0,
            is_moving: false,
            hp: STARTING_HP,
            max_hp: MAX_HP,
            lives: STARTING_LIVES,
            has_key: false,
            attack_cooldown: 0.0,
            is_attacking: false,
            invincible_time: 0.0,
            anim_state: AnimState::Idle,
            anim_frame: 0,
            anim_timer: 0.0,
            idle_sprite: None,
            run_sprite: None,
            idle_frame_count: 1,
            run_frame_count: 1,
        }
    }

    fn derive_frame_count(tex_w: f32, cell_w: f32) -> u32 {
        let n = (tex_w / cell_w).floor() as i32;
        n.max(1) as u32
    }

    /// Load player sprites from assets
    pub async fn load_sprites(&mut self) {
        // Load Blue Knight sprites
        if let Ok(tex) = load_texture("assets/dg_knight/Blue Knight idle Sprite-sheet 16x16.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.idle_frame_count = Self::derive_frame_count(tex.width(), KNIGHT_IDLE_FRAME_W);
            self.idle_sprite = Some(tex);
        }

        if let Ok(tex) = load_texture("assets/dg_knight/Blue Knight run Sprite-sheet 16x17.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.run_frame_count = Self::derive_frame_count(tex.width(), KNIGHT_RUN_FRAME_W);
            self.run_sprite = Some(tex);
        }
    }

    /// Set preloaded sprites (non-async version)
    pub fn set_sprites(&mut self, idle: Texture2D, run: Texture2D) {
        self.idle_frame_count = Self::derive_frame_count(idle.width(), KNIGHT_IDLE_FRAME_W);
        self.run_frame_count = Self::derive_frame_count(run.width(), KNIGHT_RUN_FRAME_W);
        self.idle_sprite = Some(idle);
        self.run_sprite = Some(run);
    }

    /// Update player state
    pub fn update(&mut self, dt: f32, level: &Level, actions: &[InputAction]) {
        // Update cooldowns
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        if self.invincible_time > 0.0 {
            self.invincible_time -= dt;
        }

        // Process input for movement
        if !self.is_moving && self.attack_cooldown <= 0.0 {
            for &action in actions {
                match action {
                    InputAction::MoveUp => {
                        self.try_move(Direction::Up, level);
                        break;
                    }
                    InputAction::MoveDown => {
                        self.try_move(Direction::Down, level);
                        break;
                    }
                    InputAction::MoveLeft => {
                        self.try_move(Direction::Left, level);
                        break;
                    }
                    InputAction::MoveRight => {
                        self.try_move(Direction::Right, level);
                        break;
                    }
                    InputAction::Attack => {
                        self.start_attack();
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Update movement
        if self.is_moving {
            self.move_progress += dt * (PLAYER_SPEED / TILE_SIZE);

            if self.move_progress >= 1.0 {
                // Movement complete
                self.move_progress = 0.0;
                self.is_moving = false;

                // Snap to grid
                self.x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                self.y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            } else {
                // Interpolate position
                let (dx, dy) = self.facing.to_vec();
                let start_x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let start_y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let target_x = (self.grid_x + dx) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let target_y = (self.grid_y + dy) as f32 * TILE_SIZE + TILE_SIZE / 2.0;

                self.x = start_x + (target_x - start_x) * self.move_progress;
                self.y = start_y + (target_y - start_y) * self.move_progress;
            }
        }

        // Update animation
        self.update_animation(dt);
    }

    /// Try to move in a direction
    fn try_move(&mut self, dir: Direction, level: &Level) {
        let (dx, dy) = dir.to_vec();
        let new_x = self.grid_x + dx;
        let new_y = self.grid_y + dy;

        // Check if the target tile is walkable
        if level.is_valid(new_x, new_y) {
            self.facing = dir;
            self.grid_x = new_x;
            self.grid_y = new_y;
            self.is_moving = true;
            self.move_progress = 0.0;
            self.anim_state = AnimState::Run;
        } else {
            // Face the direction even if we can't move
            self.facing = dir;
        }
    }

    /// Start an attack
    fn start_attack(&mut self) {
        self.is_attacking = true;
        self.attack_cooldown = PLAYER_ATTACK_COOLDOWN;
        self.anim_state = AnimState::Attack;
        self.anim_frame = 0;
    }

    /// Get the attack hitbox position
    pub fn get_attack_position(&self) -> (i32, i32) {
        let (dx, dy) = self.facing.to_vec();
        (self.grid_x + dx, self.grid_y + dy)
    }

    /// Take damage
    pub fn take_damage(&mut self, amount: i32) {
        if self.invincible_time > 0.0 {
            return;
        }

        self.hp -= amount;
        self.invincible_time = PLAYER_INVINCIBLE_TIME;
        self.anim_state = AnimState::Hurt;

        if self.hp <= 0 {
            self.hp = 0;
            self.anim_state = AnimState::Death;
        }
    }

    /// Heal the player
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Respawn at level start
    pub fn respawn(&mut self, spawn_x: i32, spawn_y: i32) {
        self.grid_x = spawn_x;
        self.grid_y = spawn_y;
        self.x = spawn_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        self.y = spawn_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        self.hp = self.max_hp;
        self.is_moving = false;
        self.move_progress = 0.0;
        self.invincible_time = PLAYER_INVINCIBLE_TIME;
        self.anim_state = AnimState::Idle;
        self.has_key = false;
    }

    /// Update animation state
    fn update_animation(&mut self, dt: f32) {
        self.anim_timer += dt;

        match self.anim_state {
            AnimState::Idle => {
                let frame_time = PLAYER_IDLE_FRAME_TIME;
                if self.anim_timer >= frame_time {
                    self.anim_timer = 0.0;
                    let n = self.idle_frame_count.max(1);
                    self.anim_frame = (self.anim_frame + 1) % n;
                }
            }
            AnimState::Run => {
                if !self.is_moving {
                    self.anim_state = AnimState::Idle;
                    self.anim_frame = 0;
                } else {
                    let frame_time = PLAYER_RUN_FRAME_TIME;
                    if self.anim_timer >= frame_time {
                        self.anim_timer = 0.0;
                        let n = self.run_frame_count.max(1);
                        self.anim_frame = (self.anim_frame + 1) % n;
                    }
                }
            }
            AnimState::Attack => {
                // Attack animation is quick
                if self.anim_timer >= 0.1 {
                    self.is_attacking = false;
                    self.anim_state = if self.is_moving { AnimState::Run } else { AnimState::Idle };
                    self.anim_frame = 0;
                    self.anim_timer = 0.0;
                }
            }
            AnimState::Hurt => {
                // Brief hurt flash
                if self.anim_timer >= 0.2 {
                    self.anim_state = if self.is_moving { AnimState::Run } else { AnimState::Idle };
                    self.anim_frame = 0;
                    self.anim_timer = 0.0;
                }
            }
            AnimState::Death => {
                // Death animation - handled separately
            }
        }
    }

    /// Draw the player
    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        // Flash when invincible
        if self.invincible_time > 0.0 {
            let flash_rate = 0.1;
            if ((self.invincible_time % flash_rate) / flash_rate) > 0.5 {
                return; // Skip drawing for flash effect
            }
        }

        // Draw sprite if loaded, otherwise draw placeholder
        // Player is rendered at PLAYER_DISPLAY_SIZE (2x tile size) for TV visibility
        let display_size = PLAYER_DISPLAY_SIZE;
        let half_size = display_size / 2.0;

        if let (Some(ref idle), Some(ref run)) = (&self.idle_sprite, &self.run_sprite) {
            // ── Direction → sprite row mapping ────────────────────────────
            // Idle sheet (128×64, 8 cols × 4 rows, 16×16 per frame):
            //   Row 0 — facing DOWN  (front, visor visible)
            //   Row 1 — facing LEFT  (side profile)
            //   Row 2 — facing UP    (back, no visor)
            //   Row 3 — facing LEFT  (second side cycle, reserved)
            // For RIGHT: use row 1 with flip_x = true
            //
            // Run sheet uses the same row convention if it has 4 rows;
            // fall back to row 0 if it only has 1 row (simpler sheets).

            let use_run = matches!(self.anim_state, AnimState::Run);
            let sprite = if use_run { run } else { idle };
            let cell_w = if use_run { KNIGHT_RUN_FRAME_W  } else { KNIGHT_IDLE_FRAME_W };
            let cell_h = if use_run { KNIGHT_RUN_FRAME_H  } else { KNIGHT_IDLE_FRAME_H };
            let frame_count = if use_run { self.run_frame_count } else { self.idle_frame_count };

            // Direction row: idle sheet has 4 rows; run sheet may have fewer.
            let dir_row: u32 = {
                let available_rows = (sprite.height() / cell_h).floor() as u32;
                let wanted = match self.facing {
                    Direction::Down  => 0,
                    Direction::Left  => 1,
                    Direction::Right => 1,  // same row, flip_x below
                    Direction::Up    => 2,
                };
                wanted.min(available_rows.saturating_sub(1))
            };

            let frame = self.anim_frame % frame_count.max(1);

            // Column within the chosen row
            let max_x = frame_count as f32 * cell_w;
            let src_x = (frame as f32 * cell_w).min((max_x - cell_w).max(0.0));
            let src_y = dir_row as f32 * cell_h;
            let src_w = cell_w.min(sprite.width() - src_x).max(1.0);
            let src_h = cell_h.min(sprite.height() - src_y).max(1.0);

            // Flip horizontally when facing RIGHT (reuses the LEFT-facing row)
            let flip_x = self.facing == Direction::Left;

            draw_texture_ex(
                sprite,
                screen_x - half_size,
                screen_y - half_size,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(src_x, src_y, src_w, src_h)),
                    dest_size: Some(vec2(display_size, display_size)),
                    flip_x,
                    ..Default::default()
                },
            );
        } else {
            // Placeholder: Blue rectangle
            let color = match self.anim_state {
                AnimState::Hurt => WHITE,
                AnimState::Attack => YELLOW,
                _ => LEVEL1_PALETTE.accent,
            };
            draw_rectangle(
                screen_x - half_size,
                screen_y - half_size,
                display_size,
                display_size,
                color,
            );

            // Draw eyes to show facing direction (scaled)
            let eye_scale = display_size / TILE_SIZE; // Scale factor
            let eye_offset = if self.facing == Direction::Left { -6.0 * eye_scale } else { 6.0 * eye_scale };
            draw_circle(screen_x + eye_offset, screen_y - 4.0 * eye_scale, 3.0 * eye_scale, WHITE);
            draw_circle(screen_x + eye_offset, screen_y - 4.0 * eye_scale, 1.5 * eye_scale, BLACK);
        }

        // Draw attack effect
        if self.is_attacking {
            let (dx, dy) = self.facing.to_vec();
            let attack_x = screen_x + dx as f32 * TILE_SIZE;
            let attack_y = screen_y + dy as f32 * TILE_SIZE;

            draw_circle_lines(attack_x, attack_y, 8.0, 2.0, YELLOW);
        }
    }

    /// Check if player is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Check if player is at the exit
    pub fn at_exit(&self, level: &Level) -> bool {
        self.grid_x == level.exit_x && self.grid_y == level.exit_y
    }
}