use macroquad::prelude::*;

use crate::config::*;
use crate::input::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitVfxKind {
    PlayerHit,
    EnemyHit,
}

#[derive(Clone)]
struct HitVfxSprite {
    texture: Texture2D,
    frame_w: f32,
    frame_h: f32,
    frames_per_row: usize,
    rows: usize,
    frame_time: f32,
}

impl HitVfxSprite {
    fn from_texture(texture: Texture2D, frame_w: f32, frame_h: f32, frame_time: f32) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let frames_per_row = (texture.width() / frame_w).floor().max(1.0) as usize;
        let rows = (texture.height() / frame_h).floor().max(1.0) as usize;
        Self {
            texture,
            frame_w,
            frame_h,
            frames_per_row,
            rows,
            frame_time,
        }
    }
}

pub struct HitVfxAtlas {
    player_hit: HitVfxSprite,
    enemy_hit: HitVfxSprite,
}

impl HitVfxAtlas {
    pub async fn load() -> Option<Self> {
        let player_tex = load_texture("assets/vfx/player_hit_spritesheet.png").await.ok()?;
        let enemy_tex = load_texture("assets/vfx/enemy_hit_spritesheet.png").await.ok()?;

        Some(Self {
            // Current assets:
            // - player_hit_spritesheet.png: 320x128  => 5 cols x 2 rows (64x64)
            // - enemy_hit_spritesheet.png:  768x128  => 12 cols x 2 rows (64x64)
            player_hit: HitVfxSprite::from_texture(player_tex, 64.0, 64.0, HIT_VFX_PLAYER_FRAME_TIME),
            enemy_hit: HitVfxSprite::from_texture(enemy_tex, 64.0, 64.0, HIT_VFX_ENEMY_FRAME_TIME),
        })
    }

    fn sprite_for(&self, kind: HitVfxKind) -> &HitVfxSprite {
        match kind {
            HitVfxKind::PlayerHit => &self.player_hit,
            HitVfxKind::EnemyHit => &self.enemy_hit,
        }
    }
}

#[derive(Clone)]
pub struct HitVfxInstance {
    kind: HitVfxKind,
    tile_x: i32,
    tile_y: i32,
    facing: Direction,
    elapsed: f32,
    active: bool,
}

impl HitVfxInstance {
    pub fn spawn(kind: HitVfxKind, tile_x: i32, tile_y: i32, facing: Direction) -> Self {
        Self {
            kind,
            tile_x,
            tile_y,
            facing,
            elapsed: 0.0,
            active: true,
        }
    }

    pub fn update(&mut self, dt: f32, atlas: &HitVfxAtlas) {
        if !self.active {
            return;
        }
        self.elapsed += dt;
        let sprite = atlas.sprite_for(self.kind);
        let lifetime = sprite.frame_time * sprite.frames_per_row as f32;
        if self.elapsed >= lifetime {
            self.active = false;
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: &HitVfxAtlas) {
        if !self.active {
            return;
        }

        let sprite = atlas.sprite_for(self.kind);
        let frame_idx = ((self.elapsed / sprite.frame_time) as usize).min(sprite.frames_per_row.saturating_sub(1));
        // `player_hit_spritesheet.png` renders correctly from row 0 for all facings.
        // Row 1 contains partial/offset art and clips for horizontal attacks.
        let row = 0.min(sprite.rows.saturating_sub(1));
        let src_x = frame_idx as f32 * sprite.frame_w;
        let src_y = row as f32 * sprite.frame_h;

        let center_x = self.tile_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let center_y = self.tile_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let screen_x = center_x - camera_x;
        let screen_y = center_y - camera_y;
        let draw_size = TILE_SIZE * HIT_VFX_SCALE;
        let flip_x = self.facing == Direction::Left;

        draw_texture_ex(
            &sprite.texture,
            screen_x - draw_size / 2.0,
            screen_y - draw_size / 2.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(src_x, src_y, sprite.frame_w, sprite.frame_h)),
                dest_size: Some(vec2(draw_size, draw_size)),
                flip_x,
                ..Default::default()
            },
        );
    }
}
