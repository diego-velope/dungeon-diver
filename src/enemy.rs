// Dungeon Diver - Enemy Entity
// Zombie enemies with simple AI and combat
//
// ── SPRITE SHEET SPEC (verified by pixel measurement) ────────────────────────
//
//  All sheets use 32×32 px frames, 4 rows for 4 directions:
//    Row 0 → Down  (front-facing, both arms visible)
//    Row 1 → Left  (left-profile)
//    Row 2 → Up    (back/away-facing)
//    Row 3 → Right (right-profile)
//
//  idle:  256×128  8 cols × 4 rows  (8 frames per direction)
//  run:   256×128  8 cols × 4 rows  (8 frames per direction)
//  hurt:   64×128  2 cols × 4 rows  (2 frames per direction: flash + hurt)
//  death: 256×128  8 cols × 4 rows  (8 frames per direction)

use macroquad::prelude::*;
use crate::constants::*;
use crate::input::Direction;
use crate::level::Level;

/// Enemy states for animation and behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Idle,
    Chasing,
    Attacking,
    Hurt,
    Dying,
    Dead,
}

/// Preloaded enemy spritesheets
pub struct EnemyAtlas {
    pub idle:  Texture2D,
    pub run:   Texture2D,
    pub hurt:  Texture2D,
    pub death: Texture2D,
}

/// Native frame size in pixels for every zombie sheet.
const FRAME_W: f32 = 32.0;
const FRAME_H: f32 = 32.0;

impl EnemyAtlas {
    pub async fn load() -> Option<Self> {
        let idle  = load_texture("assets/enemies_zombies/zombie_idle.png").await.ok()?;
        let run   = load_texture("assets/enemies_zombies/zombie_run.png").await.ok()?;
        let hurt  = load_texture("assets/enemies_zombies/zombie_hurt.png").await.ok()?;
        let death = load_texture("assets/enemies_zombies/zombie_death.png").await.ok()?;

        idle .set_filter(FilterMode::Nearest);
        run  .set_filter(FilterMode::Nearest);
        hurt .set_filter(FilterMode::Nearest);
        death.set_filter(FilterMode::Nearest);

        Some(Self { idle, run, hurt, death })
    }

    /// Number of animation frames per direction row.
    /// All sheets: width / 32.0 (verified).
    pub fn frames_for(sprite: &Texture2D) -> usize {
        (sprite.width() / FRAME_W).floor() as usize
    }

    pub fn idle_frame_count(&self)  -> usize { Self::frames_for(&self.idle)  }
    pub fn run_frame_count(&self)   -> usize { Self::frames_for(&self.run)   }
    pub fn hurt_frame_count(&self)  -> usize { Self::frames_for(&self.hurt)  }
    pub fn death_frame_count(&self) -> usize { Self::frames_for(&self.death) }
}

/// Zombie enemy entity
pub struct Enemy {
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f32,
    pub y: f32,

    pub hp: i32,
    pub max_hp: i32,
    pub attack_cooldown: f32,

    pub facing: Direction,
    pub move_progress: f32,
    pub is_moving: bool,
    pub move_timer: f32,

    pub state: EnemyState,
    pub anim_frame: usize,
    pub anim_timer: f32,
    pub death_timer: f32,
}

impl Enemy {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        let x = grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        Self {
            grid_x, grid_y, x, y,
            hp: ENEMY_HP,
            max_hp: ENEMY_HP,
            attack_cooldown: 0.0,
            facing: Direction::Down,
            move_progress: 0.0,
            is_moving: false,
            move_timer: 0.0,
            state: EnemyState::Idle,
            anim_frame: 0,
            anim_timer: 0.0,
            death_timer: 0.0,
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        // Guard both Dead AND Dying — otherwise repeated hits while the death
        // animation plays reset anim_frame to 0 and the zombie never disappears.
        if self.state == EnemyState::Dead || self.state == EnemyState::Dying { return; }
        self.hp -= amount;
        if self.hp <= 0 {
            self.hp = 0;
            self.state = EnemyState::Dying;
            self.anim_frame = 0;
            self.anim_timer = 0.0;
        } else {
            self.state = EnemyState::Hurt;
            self.anim_frame = 0;
            self.anim_timer = 0.0;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state != EnemyState::Dead
    }

    // ── update (uses level reference) ────────────────────────────────────────

    pub fn update(&mut self, dt: f32, player_pos: (i32, i32), level: &Level) {
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }

        if self.state == EnemyState::Dying {
            self.update_death(dt);
            return;
        }
        if self.state == EnemyState::Hurt {
            self.anim_timer += dt;
            if self.anim_timer >= 0.2 {
                self.state = EnemyState::Idle;
                self.anim_timer = 0.0;
            }
            return;
        }
        if self.is_moving {
            self.update_movement(dt);
            return;
        }
        self.update_ai(dt, player_pos, |dir, me| {
            let (dx, dy) = dir.to_vec();
            let nx = me.grid_x + dx;
            let ny = me.grid_y + dy;
            if level.is_valid(nx, ny) {
                me.facing = dir;
                me.grid_x = nx;
                me.grid_y = ny;
                me.is_moving = true;
                me.move_progress = 0.0;
            } else {
                me.facing = dir;
            }
        });
        self.update_animation(dt);
    }

    // ── update (uses raw bounds + tiles — avoids borrow issues) ──────────────

    pub fn update_with_bounds(
        &mut self,
        dt: f32,
        player_pos: (i32, i32),
        level_w: i32,
        level_h: i32,
        tiles: &[Vec<crate::level::Tile>],
    ) {
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }

        if self.state == EnemyState::Dying {
            self.update_death(dt);
            return;
        }
        if self.state == EnemyState::Hurt {
            self.anim_timer += dt;
            if self.anim_timer >= 0.2 {
                self.state = EnemyState::Idle;
                self.anim_timer = 0.0;
            }
            return;
        }
        if self.is_moving {
            self.update_movement(dt);
            return;
        }
        self.update_ai(dt, player_pos, |dir, me| {
            let (dx, dy) = dir.to_vec();
            let nx = me.grid_x + dx;
            let ny = me.grid_y + dy;
            let valid = nx >= 0 && nx < level_w && ny >= 0 && ny < level_h
                && !tiles[ny as usize][nx as usize].is_solid();
            if valid {
                me.facing = dir;
                me.grid_x = nx;
                me.grid_y = ny;
                me.is_moving = true;
                me.move_progress = 0.0;
            } else {
                me.facing = dir;
            }
        });
        self.update_animation(dt);
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn update_death(&mut self, dt: f32) {
        self.death_timer += dt;
        self.anim_timer  += dt;
        if self.anim_timer >= 0.15 {
            self.anim_timer = 0.0;
            self.anim_frame += 1;
            // death sheet has exactly 8 frames per row (256 / 32 = 8)
            if self.anim_frame >= 8 {
                self.state = EnemyState::Dead;
            }
        }
    }

    fn update_movement(&mut self, dt: f32) {
        self.move_progress += dt * (PLAYER_SPEED / TILE_SIZE) * 0.5;
        if self.move_progress >= 1.0 {
            self.move_progress = 0.0;
            self.is_moving = false;
            self.x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            self.y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        } else {
            let (dx, dy) = self.facing.to_vec();
            let sx = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let sy = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let tx = (self.grid_x + dx) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let ty = (self.grid_y + dy) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            self.x = sx + (tx - sx) * self.move_progress;
            self.y = sy + (ty - sy) * self.move_progress;
        }
    }

    fn update_ai<F>(&mut self, dt: f32, player_pos: (i32, i32), mut try_move: F)
    where
        F: FnMut(Direction, &mut Enemy),
    {
        self.move_timer += dt;
        if self.move_timer < ENEMY_MOVE_INTERVAL { return; }
        self.move_timer = 0.0;

        let (px, py) = player_pos;
        let dx = (px - self.grid_x).abs();
        let dy = (py - self.grid_y).abs();

        if dx.max(dy) > ENEMY_ACTIVATION_RANGE {
            self.state = EnemyState::Idle;
            return;
        }

        self.state = EnemyState::Chasing;
        let dir = if px > self.grid_x && dx >= dy      { Direction::Right }
                  else if px < self.grid_x && dx >= dy { Direction::Left  }
                  else if py > self.grid_y              { Direction::Down  }
                  else                                  { Direction::Up    };
        try_move(dir, self);
    }

    fn update_animation(&mut self, dt: f32) {
        self.anim_timer += dt;
        let frame_time = if self.state == EnemyState::Chasing || self.is_moving {
            0.12
        } else {
            0.2
        };
        if self.anim_timer >= frame_time {
            self.anim_timer = 0.0;
            self.anim_frame += 1;
        }
    }

    // ── draw ─────────────────────────────────────────────────────────────────

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: &EnemyAtlas) {
        if self.state == EnemyState::Dead { return; }

        let sx = self.x - camera_x;
        let sy = self.y - camera_y;

        // Pick spritesheet for current state
        let sprite = match self.state {
            EnemyState::Dying                          => &atlas.death,
            EnemyState::Hurt                           => &atlas.hurt,
            EnemyState::Chasing | EnemyState::Attacking => &atlas.run,
            _                                          => &atlas.idle,
        };

        // ── direction row ─────────────────────────────────────────────────
        // All sheets: Row 0=Down  Row 1=Left  Row 2=Up  Row 3=Right
        let total_rows = (sprite.height() / FRAME_H).floor() as usize;
        let dir_row = match self.facing {
            Direction::Down  => 0,
            Direction::Left  => 1,
            Direction::Up    => 2,
            Direction::Right => 3,
        }.min(total_rows.saturating_sub(1));  // clamp so we never read past sheet

        // ── animation frame ───────────────────────────────────────────────
        let frames_per_row = (sprite.width() / FRAME_W).floor() as usize;
        let frame_idx = self.anim_frame % frames_per_row.max(1);

        let src_x = frame_idx as f32 * FRAME_W;
        let src_y = dir_row   as f32 * FRAME_H;

        draw_texture_ex(
            sprite,
            sx - TILE_SIZE / 2.0,
            sy - TILE_SIZE / 2.0,
            WHITE,
            DrawTextureParams {
                source:    Some(Rect::new(src_x, src_y, FRAME_W, FRAME_H)),
                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
        );

        // ── health bar (shown when hurt or chasing) ───────────────────────
        if self.state == EnemyState::Chasing || self.state == EnemyState::Hurt {
            let bar_w   = TILE_SIZE * 0.75;
            let bar_h   = 4.0;
            let bar_x   = sx - bar_w / 2.0;
            let bar_y   = sy - TILE_SIZE / 2.0 - 8.0;
            let hp_frac = self.hp as f32 / self.max_hp as f32;

            draw_rectangle(bar_x, bar_y, bar_w,           bar_h, Color { r:0.3, g:0.0, b:0.0, a:0.8 });
            draw_rectangle(bar_x, bar_y, bar_w * hp_frac, bar_h, Color { r:0.9, g:0.1, b:0.1, a:0.9 });
        }
    }
}