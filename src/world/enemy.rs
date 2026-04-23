use macroquad::prelude::*;
use crate::config::*;
use crate::input::Direction;
use crate::world::Level;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Idle,
    Chasing,
    Attacking,
    Hurt,
    Dying,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Zombie,
    BigZombie,
    BigDemon,
}

pub struct EnemyAtlas {
    pub idle: Texture2D,
    pub run: Texture2D,
    pub hurt: Texture2D,
    pub death: Texture2D,
    pub big_zombie_idle: Vec<Texture2D>,
    pub big_zombie_run: Vec<Texture2D>,
    pub big_demon_idle: Vec<Texture2D>,
    pub big_demon_run: Vec<Texture2D>,
}

const FRAME_W: f32 = 32.0;
const FRAME_H: f32 = 32.0;

impl EnemyAtlas {
    async fn load_frame_set(prefix: &str) -> Option<Vec<Texture2D>> {
        let mut frames = Vec::new();
        for idx in 0..=3 {
            let tex = load_texture(&format!("assets/sprites/enemies/{}_f{}.png", prefix, idx)).await.ok()?;
            tex.set_filter(FilterMode::Nearest);
            frames.push(tex);
        }
        Some(frames)
    }

    pub async fn load() -> Option<Self> {
        let idle = load_texture("assets/sprites/enemies/zombie_idle.png").await.ok()?;
        let run = load_texture("assets/sprites/enemies/zombie_run.png").await.ok()?;
        let hurt = load_texture("assets/sprites/enemies/zombie_hurt.png").await.ok()?;
        let death = load_texture("assets/sprites/enemies/zombie_death.png").await.ok()?;
        idle.set_filter(FilterMode::Nearest);
        run.set_filter(FilterMode::Nearest);
        hurt.set_filter(FilterMode::Nearest);
        death.set_filter(FilterMode::Nearest);

        Some(Self {
            idle,
            run,
            hurt,
            death,
            big_zombie_idle: Self::load_frame_set("big_zombie_idle_anim").await?,
            big_zombie_run: Self::load_frame_set("big_zombie_run_anim").await?,
            big_demon_idle: Self::load_frame_set("big_demon_idle_anim").await?,
            big_demon_run: Self::load_frame_set("big_demon_run_anim").await?,
        })
    }
}

pub struct Enemy {
    pub kind: EnemyKind,
    pub grid_x: i32,
    pub grid_y: i32,
    pub move_start_x: i32,
    pub move_start_y: i32,
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
        Self::new_with_kind(grid_x, grid_y, EnemyKind::Zombie)
    }

    pub fn new_with_kind(grid_x: i32, grid_y: i32, kind: EnemyKind) -> Self {
        let x = grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        Self {
            kind,
            grid_x,
            grid_y,
            move_start_x: grid_x,
            move_start_y: grid_y,
            x,
            y,
            hp: Self::max_hp_for(kind),
            max_hp: Self::max_hp_for(kind),
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

    pub fn damage_for(&self) -> i32 {
        match self.kind {
            EnemyKind::Zombie => ENEMY_DAMAGE,
            EnemyKind::BigZombie => BIG_ZOMBIE_DAMAGE,
            EnemyKind::BigDemon => BIG_DEMON_DAMAGE,
        }
    }

    pub fn attack_cooldown_for(&self) -> f32 {
        match self.kind {
            EnemyKind::Zombie => ENEMY_ATTACK_COOLDOWN,
            EnemyKind::BigZombie => BIG_ZOMBIE_ATTACK_COOLDOWN,
            EnemyKind::BigDemon => BIG_DEMON_ATTACK_COOLDOWN,
        }
    }

    fn move_interval_for(kind: EnemyKind) -> f32 {
        match kind {
            EnemyKind::Zombie => ENEMY_MOVE_INTERVAL,
            EnemyKind::BigZombie => BIG_ZOMBIE_MOVE_INTERVAL,
            EnemyKind::BigDemon => BIG_DEMON_MOVE_INTERVAL,
        }
    }

    fn activation_range_for(kind: EnemyKind) -> i32 {
        match kind {
            EnemyKind::Zombie => ENEMY_ACTIVATION_RANGE,
            EnemyKind::BigZombie => BIG_ZOMBIE_ACTIVATION_RANGE,
            EnemyKind::BigDemon => BIG_DEMON_ACTIVATION_RANGE,
        }
    }

    fn max_hp_for(kind: EnemyKind) -> i32 {
        match kind {
            EnemyKind::Zombie => ENEMY_HP,
            EnemyKind::BigZombie => BIG_ZOMBIE_HP,
            EnemyKind::BigDemon => BIG_DEMON_HP,
        }
    }

    fn display_size_for(kind: EnemyKind) -> f32 {
        match kind {
            EnemyKind::Zombie => ENEMY_DISPLAY_SIZE,
            EnemyKind::BigZombie => BIG_ZOMBIE_DISPLAY_SIZE,
            EnemyKind::BigDemon => BIG_DEMON_DISPLAY_SIZE,
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        if self.state == EnemyState::Dead || self.state == EnemyState::Dying {
            return;
        }
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

    pub fn get_attack_position(&self) -> (i32, i32) {
        let (dx, dy) = self.facing.to_vec();
        (self.grid_x + dx, self.grid_y + dy)
    }

    pub fn is_alive(&self) -> bool {
        self.state != EnemyState::Dead
    }

    pub fn update(&mut self, dt: f32, player_pos: (i32, i32), level: &Level) {
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
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
                me.move_start_x = me.grid_x;
                me.move_start_y = me.grid_y;
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

    pub fn update_with_bounds(
        &mut self,
        dt: f32,
        player_pos: (i32, i32),
        level_w: i32,
        level_h: i32,
        tiles: &[Vec<crate::world::Tile>],
    ) {
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
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
            let valid = nx >= 0
                && nx < level_w
                && ny >= 0
                && ny < level_h
                && !tiles[ny as usize][nx as usize].is_solid()
                && (nx, ny) != player_pos;
            if valid {
                me.facing = dir;
                me.move_start_x = me.grid_x;
                me.move_start_y = me.grid_y;
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

    fn update_death(&mut self, dt: f32) {
        self.death_timer += dt;
        self.anim_timer += dt;
        let death_frames = if self.kind == EnemyKind::Zombie { 8 } else { 4 };
        if self.anim_timer >= 0.15 {
            self.anim_timer = 0.0;
            self.anim_frame += 1;
            if self.anim_frame >= death_frames {
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
            let start_x = self.move_start_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let start_y = self.move_start_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let target_x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let target_y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            self.x = start_x + (target_x - start_x) * self.move_progress;
            self.y = start_y + (target_y - start_y) * self.move_progress;
        }
    }

    fn update_ai<F>(&mut self, dt: f32, player_pos: (i32, i32), mut try_move: F)
    where
        F: FnMut(Direction, &mut Enemy),
    {
        self.move_timer += dt;
        if self.move_timer < Self::move_interval_for(self.kind) {
            return;
        }
        self.move_timer = 0.0;

        let (px, py) = player_pos;
        let dx = (px - self.grid_x).abs();
        let dy = (py - self.grid_y).abs();
        if dx.max(dy) > Self::activation_range_for(self.kind) {
            self.state = EnemyState::Idle;
            return;
        }

        self.state = EnemyState::Chasing;
        let dir = if px > self.grid_x && dx >= dy {
            Direction::Right
        } else if px < self.grid_x && dx >= dy {
            Direction::Left
        } else if py > self.grid_y {
            Direction::Down
        } else {
            Direction::Up
        };
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

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: &EnemyAtlas) {
        if self.state == EnemyState::Dead {
            return;
        }
        let sx = self.x - camera_x;
        let sy = self.y - camera_y;
        match self.kind {
            EnemyKind::Zombie => self.draw_zombie(sx, sy, atlas),
            EnemyKind::BigZombie => self.draw_frame_enemy(
                sx,
                sy,
                self.state == EnemyState::Chasing || self.is_moving,
                &atlas.big_zombie_idle,
                &atlas.big_zombie_run,
            ),
            EnemyKind::BigDemon => self.draw_frame_enemy(
                sx,
                sy,
                self.state == EnemyState::Chasing || self.is_moving,
                &atlas.big_demon_idle,
                &atlas.big_demon_run,
            ),
        }
        if self.state == EnemyState::Chasing || self.state == EnemyState::Hurt {
            let display_size = Self::display_size_for(self.kind);
            let bar_w = display_size * 0.75;
            let bar_h = 4.0;
            let bar_x = sx - bar_w / 2.0;
            let bar_y = sy - display_size / 2.0 - 8.0;
            let hp_frac = self.hp as f32 / self.max_hp as f32;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color { r: 0.3, g: 0.0, b: 0.0, a: 0.8 });
            draw_rectangle(bar_x, bar_y, bar_w * hp_frac, bar_h, Color { r: 0.9, g: 0.1, b: 0.1, a: 0.9 });
        }
    }

    fn draw_zombie(&self, sx: f32, sy: f32, atlas: &EnemyAtlas) {
        let sprite = match self.state {
            EnemyState::Dying => &atlas.death,
            EnemyState::Hurt => &atlas.hurt,
            EnemyState::Chasing | EnemyState::Attacking => &atlas.run,
            _ => &atlas.idle,
        };
        let total_rows = (sprite.height() / FRAME_H).floor() as usize;
        let dir_row = match self.facing {
            Direction::Down => 0,
            Direction::Left => 1,
            Direction::Right => 1,
            Direction::Up => 2,
        }
        .min(total_rows.saturating_sub(1));
        let flip_x = self.facing == Direction::Left;
        let frames_per_row = (sprite.width() / FRAME_W).floor() as usize;
        let frame_idx = self.anim_frame % frames_per_row.max(1);
        let src_x = frame_idx as f32 * FRAME_W;
        let src_y = dir_row as f32 * FRAME_H;
        let src_w = FRAME_W.min(sprite.width() - src_x).max(1.0);
        let src_h = FRAME_H.min(sprite.height() - src_y).max(1.0);
        let display_size = Self::display_size_for(self.kind);
        let half_size = display_size / 2.0;
        draw_texture_ex(
            sprite,
            sx - half_size,
            sy - half_size,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(src_x, src_y, src_w, src_h)),
                dest_size: Some(vec2(display_size, display_size)),
                flip_x,
                ..Default::default()
            },
        );
    }

    fn draw_frame_enemy(
        &self,
        sx: f32,
        sy: f32,
        moving: bool,
        idle_frames: &[Texture2D],
        run_frames: &[Texture2D],
    ) {
        let frames = if moving { run_frames } else { idle_frames };
        if frames.is_empty() {
            return;
        }
        let tex = &frames[self.anim_frame % frames.len()];
        let display_size = Self::display_size_for(self.kind);
        let half_size = display_size / 2.0;
        draw_texture_ex(
            tex,
            sx - half_size,
            sy - half_size,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(display_size, display_size)),
                flip_x: self.facing == Direction::Left,
                ..Default::default()
            },
        );
    }
}