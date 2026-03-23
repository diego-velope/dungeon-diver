// Dungeon Diver - Items & Collectibles
// Coins, potions, breakable vases

use macroquad::prelude::*;
use crate::constants::*;

/// Preloaded item sprites needed for interactive objects (chests, keys, etc).
pub struct ItemsAtlas {
    pub chest_full_open: Vec<Texture2D>,  // 3 frames
    pub chest_empty_open: Vec<Texture2D>, // 3 frames
    pub key_tex: Texture2D,

    // Coin visuals (dg_gathering_free_ver spritesheets)
    pub coin_sheet: Texture2D,
    pub blue_coin_sheet: Texture2D,

    // 0x72 coin bag (single sprite)
    pub coin_bag: Texture2D,

    // 0x72 hearts (UI)
    pub heart_full: Texture2D,
    pub heart_half: Texture2D,
    pub heart_empty: Texture2D,

    // Potions (dg_gathering_free_ver)
    pub potion_big: Texture2D,
    pub potion_small: Texture2D,

    // Torches (dg_gathering_free_ver)
    pub torch_left: Texture2D,
    pub torch_right: Texture2D,
    pub torch_top: Texture2D,
}

impl ItemsAtlas {
    pub async fn load() -> Option<Self> {
        let key_tex = load_texture("assets/0x72/key.png").await.ok()?;
        let coin_sheet = load_texture("assets/dg_gathering_free_ver/Coin Sheet.png").await.ok()?;
        let blue_coin_sheet = load_texture("assets/dg_gathering_free_ver/BlueCoin Sheet.png").await.ok()?;
        let coin_bag = load_texture("assets/0x72/coin_bag.png").await.ok()?;
        let heart_full = load_texture("assets/0x72/ui_heart_full.png").await.ok()?;
        let heart_half = load_texture("assets/0x72/ui_heart_half.png").await.ok()?;
        let heart_empty = load_texture("assets/0x72/ui_heart_empty.png").await.ok()?;

        // Load potions
        let potion_big = load_texture("assets/dg_gathering_free_ver/potion_big_red_1.png").await.ok()?;
        let potion_small = load_texture("assets/dg_gathering_free_ver/potion_small_red_2.png").await.ok()?;

        // Load torches
        let torch_left = load_texture("assets/dg_gathering_free_ver/torch_left.png").await.ok()?;
        let torch_right = load_texture("assets/dg_gathering_free_ver/torch_right.png").await.ok()?;
        let torch_top = load_texture("assets/dg_gathering_free_ver/torch_top.png").await.ok()?;

        // Load chest frame strips (0x72 exports each frame as its own named PNG).
        let mut chest_full_open = Vec::new();
        for f in 0..=2 {
            let tex = load_texture(&format!(
                "assets/0x72/chest_full_open_anim_f{}.png",
                f
            ))
            .await
            .ok()?;
            chest_full_open.push(tex);
        }

        let mut chest_empty_open = Vec::new();
        for f in 0..=2 {
            let tex = load_texture(&format!(
                "assets/0x72/chest_empty_open_anim_f{}.png",
                f
            ))
            .await
            .ok()?;
            chest_empty_open.push(tex);
        }

        // Ensure pixel art stays crisp.
        let key_tex = key_tex;
        key_tex.set_filter(FilterMode::Nearest);
        for t in &mut chest_full_open {
            t.set_filter(FilterMode::Nearest);
        }
        for t in &mut chest_empty_open {
            t.set_filter(FilterMode::Nearest);
        }

        let coin_sheet = coin_sheet;
        coin_sheet.set_filter(FilterMode::Nearest);
        let blue_coin_sheet = blue_coin_sheet;
        blue_coin_sheet.set_filter(FilterMode::Nearest);
        let coin_bag = coin_bag;
        coin_bag.set_filter(FilterMode::Nearest);
        heart_full.set_filter(FilterMode::Nearest);
        heart_half.set_filter(FilterMode::Nearest);
        heart_empty.set_filter(FilterMode::Nearest);

        // Set filter for new sprites
        let potion_big = potion_big;
        potion_big.set_filter(FilterMode::Nearest);
        let potion_small = potion_small;
        potion_small.set_filter(FilterMode::Nearest);
        let torch_left = torch_left;
        torch_left.set_filter(FilterMode::Nearest);
        let torch_right = torch_right;
        torch_right.set_filter(FilterMode::Nearest);
        let torch_top = torch_top;
        torch_top.set_filter(FilterMode::Nearest);

        Some(Self {
            chest_full_open,
            chest_empty_open,
            key_tex,
            coin_sheet,
            blue_coin_sheet,
            coin_bag,
            heart_full,
            heart_half,
            heart_empty,
            potion_big,
            potion_small,
            torch_left,
            torch_right,
            torch_top,
        })
    }
}

/// Breakable/interactive chest for Level 1.
/// Stepping on it triggers the open animation; after the animation finishes,
/// the player receives the key (`player.has_key = true`).
pub struct Chest {
    pub grid_x: i32,
    pub grid_y: i32,
    pub opened: bool, // opening animation started
    pub anim_t: f32,
    pub key_given: bool,
}

impl Chest {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x,
            grid_y,
            opened: false,
            anim_t: 0.0,
            key_given: false,
        }
    }

    pub fn try_open(&mut self, player_x: i32, player_y: i32) {
        if self.key_given || self.opened {
            return;
        }
        if self.grid_x == player_x && self.grid_y == player_y {
            self.opened = true;
            self.anim_t = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.opened || self.key_given {
            return;
        }
        // Advance the open animation; once finished, the key is considered granted.
        self.anim_t += dt;
        let full_open_duration = CHEST_FULL_OPEN_FRAMES as f32 * CHEST_FRAME_TIME;
        if self.anim_t >= full_open_duration {
            self.key_given = true;
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: &ItemsAtlas) {
        let sx = self.grid_x as f32 * TILE_SIZE - camera_x;
        let sy = self.grid_y as f32 * TILE_SIZE - camera_y;

        // Chest frames are 16x16 native; draw scaled to the 1-tile (32x32) slot.
        let idx = |t: f32, frames: usize| -> usize {
            let frame = (t / CHEST_FRAME_TIME).floor() as usize;
            frame.min(frames.saturating_sub(1))
        };

        if self.opened && !self.key_given {
            let i = idx(self.anim_t, atlas.chest_full_open.len());
            draw_texture_ex(
                &atlas.chest_full_open[i],
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );
        } else if self.key_given {
            let full_d = CHEST_FRAME_TIME * CHEST_FULL_OPEN_FRAMES as f32;
            let empty_t = (self.anim_t - full_d).max(0.0);
            // After key given, keep cycling empty-open frames for visual feedback.
            let frame = (empty_t / CHEST_FRAME_TIME).floor() as usize;
            let i = frame % atlas.chest_empty_open.len();
            draw_texture_ex(
                &atlas.chest_empty_open[i],
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );
        } else {
            // Unopened chest: show the first full-open frame as a "closed/opening" look.
            draw_texture_ex(
                &atlas.chest_full_open[0],
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );
        }

        // Optional key overlay so the player clearly "receives the key".
        if !self.key_given {
            let key_dest = TILE_SIZE * 0.55;
            draw_texture_ex(
                &atlas.key_tex,
                sx + (TILE_SIZE - key_dest) / 2.0,
                sy + (TILE_SIZE - key_dest) / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(key_dest, key_dest)),
                    ..Default::default()
                },
            );
        }
    }
}

const CHEST_FULL_OPEN_FRAMES: usize = 3;
const CHEST_FRAME_TIME: f32 = 0.15;

/// Item types that can be picked up
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    Coin,
    BlueCoin,
    CoinBag,
    Potion,
    BigPotion,
    SmallPotion,
}

/// Collectible item on the level
pub struct Item {
    pub grid_x: i32,
    pub grid_y: i32,
    pub item_type: ItemType,
    pub collected: bool,
    pub bob_offset: f32,
    pub bob_time: f32,
}

impl Item {
    pub fn new(grid_x: i32, grid_y: i32, item_type: ItemType) -> Self {
        Self {
            grid_x,
            grid_y,
            item_type,
            collected: false,
            bob_offset: 0.0,
            bob_time: (grid_x as f32 * 0.5) % 3.14, // Random start offset
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Bobbing animation
        self.bob_time += dt * 3.0;
        self.bob_offset = (self.bob_time).sin() * 4.0; // Slightly more bob for visibility
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: Option<&ItemsAtlas>) {
        if self.collected {
            return;
        }

        let screen_x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_x;
        let screen_y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_y + self.bob_offset;

        // Top-left destination for texture rendering.
        let dx = screen_x - TILE_SIZE / 2.0;
        let dy = screen_y - TILE_SIZE / 2.0;

        match self.item_type {
            ItemType::Coin => {
                if let Some(atlas) = atlas {
                    let frame_w = 16.0;
                    let frame_h = 16.0;
                    let frame_count = (atlas.coin_sheet.width() / frame_w).floor() as usize;
                    let f = if frame_count == 0 { 0 } else { ((self.bob_time * 2.0).floor() as usize) % frame_count };

                    draw_texture_ex(
                        &atlas.coin_sheet,
                        dx,
                        dy,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(f as f32 * frame_w, 0.0, frame_w, frame_h)),
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback: gold coin shape
                    let coin_size = 10.0;
                    draw_circle(screen_x, screen_y, coin_size, YELLOW);
                    draw_circle_lines(screen_x, screen_y, coin_size, 3.0, Color { r: 0.8, g: 0.6, b: 0.0, a: 1.0 });
                }
            }
            ItemType::BlueCoin => {
                if let Some(atlas) = atlas {
                    let frame_w = 16.0;
                    let frame_h = 16.0;
                    let frame_count = (atlas.blue_coin_sheet.width() / frame_w).floor() as usize;
                    let f = if frame_count == 0 { 0 } else { ((self.bob_time * 2.0).floor() as usize) % frame_count };

                    draw_texture_ex(
                        &atlas.blue_coin_sheet,
                        dx,
                        dy,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(f as f32 * frame_w, 0.0, frame_w, frame_h)),
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback: blue coin shape
                    let coin_size = 10.0;
                    draw_circle(screen_x, screen_y, coin_size, LEVEL1_PALETTE.accent);
                    draw_circle_lines(screen_x, screen_y, coin_size, 3.0, Color { r: 0.2, g: 0.3, b: 0.5, a: 1.0 });
                }
            }
            ItemType::CoinBag => {
                if let Some(atlas) = atlas {
                    draw_texture_ex(
                        &atlas.coin_bag,
                        dx,
                        dy,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback coin bag: simple bag outline
                    draw_rectangle(dx + 8.0, dy + 10.0, 16.0, 18.0, YELLOW);
                    draw_rectangle_lines(dx + 8.0, dy + 10.0, 16.0, 18.0, 2.0, UI_BORDER);
                }
            }
            ItemType::Potion => {
                // Scale factor based on TILE_SIZE (base art is 16px)
                let scale = TILE_SIZE / 16.0;  // 2.0 at 32px tiles, 4.0 at 64px tiles
                // Health potion
                let potion_color = Color { r: 0.9, g: 0.2, b: 0.4, a: 1.0 };
                // Bottle
                draw_rectangle(screen_x - 6.0 * scale, screen_y - 10.0 * scale, 16.0 * scale, 24.0 * scale, potion_color);
                draw_rectangle(screen_x - 7.0 * scale, screen_y + 10.0 * scale, 18.0 * scale, 5.0 * scale, potion_color);
                // Cork
                draw_rectangle(screen_x - 5.0 * scale, screen_y - 15.0 * scale, 10.0 * scale, 5.0 * scale, Color { r: 0.6, g: 0.4, b: 0.2, a: 1.0 });
                // Shine
                draw_circle(screen_x + 3.0, screen_y - 5.0, 2.0, Color { r: 1.0, g: 0.8, b: 0.8, a: 0.8 });
            }
            ItemType::BigPotion => {
                if let Some(atlas) = atlas {
                    draw_texture_ex(
                        &atlas.potion_big,
                        dx,
                        dy,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback: larger red potion
                    let scale = TILE_SIZE / 16.0;
                    let potion_color = Color { r: 0.9, g: 0.2, b: 0.4, a: 1.0 };
                    draw_rectangle(screen_x - 8.0 * scale, screen_y - 12.0 * scale, 20.0 * scale, 30.0 * scale, potion_color);
                    draw_rectangle(screen_x - 6.0 * scale, screen_y - 18.0 * scale, 12.0 * scale, 6.0 * scale, Color { r: 0.6, g: 0.4, b: 0.2, a: 1.0 });
                }
            }
            ItemType::SmallPotion => {
                if let Some(atlas) = atlas {
                    draw_texture_ex(
                        &atlas.potion_small,
                        dx,
                        dy,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback: smaller red potion
                    let scale = TILE_SIZE / 16.0;
                    let potion_color = Color { r: 0.9, g: 0.2, b: 0.4, a: 1.0 };
                    draw_rectangle(screen_x - 4.0 * scale, screen_y - 6.0 * scale, 10.0 * scale, 16.0 * scale, potion_color);
                    draw_rectangle(screen_x - 3.0 * scale, screen_y - 10.0 * scale, 8.0 * scale, 4.0 * scale, Color { r: 0.6, g: 0.4, b: 0.2, a: 1.0 });
                }
            }
        }
    }

    pub fn collect(&mut self) -> i32 {
        if self.collected {
            return 0;
        }
        self.collected = true;
        match self.item_type {
            ItemType::Coin => COIN_VALUE,
            ItemType::BlueCoin => COIN_VALUE * 2,
            ItemType::CoinBag => 5,
            ItemType::Potion => POTION_HEAL,
            ItemType::BigPotion => POTION_HEAL * 2,  // Big potion heals 2 HP
            ItemType::SmallPotion => POTION_HEAL,    // Small potion heals 1 HP
        }
    }
}

/// Breakable vase
pub struct Vase {
    pub grid_x: i32,
    pub grid_y: i32,
    pub broken: bool,
    pub contents: Option<ItemType>,
}

impl Vase {
    pub fn new(grid_x: i32, grid_y: i32, contents: Option<ItemType>) -> Self {
        Self {
            grid_x,
            grid_y,
            broken: false,
            contents,
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        if self.broken {
            // Draw broken shards
            let screen_x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_x;
            let screen_y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_y;

            let scale = TILE_SIZE / 16.0;  // Auto-scale with tile size
            let shard_color = Color { r: 0.6, g: 0.5, b: 0.4, a: 1.0 };
            draw_rectangle(screen_x - 10.0, screen_y + 4.0, 6.0 * scale, 6.0 * scale, shard_color);
            draw_rectangle(screen_x + 4.0, screen_y + 5.0, 5.0 * scale, 4.0 * scale, shard_color);
            draw_rectangle(screen_x - 4.0, screen_y + 8.0, 8.0 * scale, 3.0 * scale, shard_color);
            return;
        }

        let screen_x = self.grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_x;
        let screen_y = self.grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 - camera_y;

        let scale = TILE_SIZE / 16.0;  // Auto-scale with tile size

        // Vase body (oval-ish)
        let vase_color = Color { r: 0.7, g: 0.6, b: 0.5, a: 1.0 };
        draw_ellipse(screen_x, screen_y, 22.0 * scale, 28.0 * scale, 0.0, vase_color);
        // Rim
        draw_ellipse_lines(screen_x, screen_y - 3.0 * scale, 26.0 * scale, 6.0 * scale, 0.0, 3.0, Color { r: 0.5, g: 0.4, b: 0.35, a: 1.0 });
        // Shine
        draw_ellipse(screen_x - 5.0, screen_y - 6.0, 6.0, 8.0, 0.0, Color { r: 0.9, g: 0.8, b: 0.7, a: 0.5 });
    }

    pub fn break_vase(&mut self) -> Option<ItemType> {
        if self.broken {
            return None;
        }
        self.broken = true;
        self.contents
    }
}

/// Torch direction for sprite selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TorchDir {
    Top,    // T - torch_top.png
    Left,   // LT - torch_left.png
    Right,  // RT - torch_right.png
}

/// Torch with glow effect and animation
pub struct Torch {
    pub grid_x: i32,
    pub grid_y: i32,
    pub flicker_time: f32,
    pub glow_intensity: f32,
    pub direction: TorchDir,
    // Animation state
    anim_frame: usize,
    anim_timer: f32,
}

impl Torch {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x,
            grid_y,
            flicker_time: (grid_x as f32 * 0.7) % 3.14,
            glow_intensity: 1.0,
            direction: TorchDir::Top,  // default to top-facing torch
            anim_frame: 0,
            anim_timer: 0.0,
        }
    }

    pub fn with_direction(grid_x: i32, grid_y: i32, direction: TorchDir) -> Self {
        Self {
            grid_x,
            grid_y,
            flicker_time: (grid_x as f32 * 0.7) % 3.14,
            glow_intensity: 1.0,
            direction,
            anim_frame: 0,
            anim_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.flicker_time += dt * 10.0;
        self.glow_intensity = 0.7 + (self.flicker_time).sin() * 0.2 + (self.flicker_time * 2.3).cos() * 0.1;

        // Update animation (0.15 seconds per frame for torch flicker)
        self.anim_timer += dt;
        if self.anim_timer >= 0.15 {
            self.anim_timer = 0.0;
            self.anim_frame += 1;
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.grid_x as f32 * TILE_SIZE - camera_x;
        let screen_y = self.grid_y as f32 * TILE_SIZE - camera_y;

        let scale = TILE_SIZE / 16.0;  // Auto-scale with tile size

        // Glow effect
        let glow_size = 50.0 * scale * self.glow_intensity;
        draw_circle(
            screen_x + TILE_SIZE / 2.0,
            screen_y + TILE_SIZE / 2.0 - 8.0 * scale,
            glow_size,
            Color { r: 1.0, g: 0.7, b: 0.3, a: 0.12 * self.glow_intensity }
        );

        // Draw torch sprite (will use atlas if available, otherwise fallback)
        // Note: The atlas-based drawing is handled in level.rs via ItemsAtlas
        // This fallback is for when atlas is not available

        // Torch handle
        let center_x = screen_x + TILE_SIZE / 2.0;
        let center_y = screen_y + TILE_SIZE / 2.0;
        draw_rectangle(center_x - 2.0, center_y, 4.0, 14.0 * scale, Color { r: 0.3, g: 0.2, b: 0.1, a: 1.0 });

        // Flame
        let flame_size = 10.0 * scale * self.glow_intensity;
        let flame_y = center_y - 8.0 * scale;

        // Outer flame (orange)
        draw_ellipse(
            center_x,
            flame_y,
            flame_size,
            flame_size * 1.3,
            0.0,
            Color { r: 1.0, g: 0.5, b: 0.0, a: 0.9 }
        );

        // Inner flame (yellow)
        draw_ellipse(
            center_x,
            flame_y + 3.0,
            flame_size * 0.6,
            flame_size * 0.8,
            0.0,
            Color { r: 1.0, g: 0.9, b: 0.3, a: 0.9 }
        );
    }

    /// Draw the torch with sprite atlas
    pub fn draw_with_atlas(&self, camera_x: f32, camera_y: f32, atlas: &ItemsAtlas) {
        let screen_x = self.grid_x as f32 * TILE_SIZE - camera_x;
        let screen_y = self.grid_y as f32 * TILE_SIZE - camera_y;

        let scale = TILE_SIZE / 16.0;

        // Glow effect
        let glow_size = 50.0 * scale * self.glow_intensity;
        draw_circle(
            screen_x + TILE_SIZE / 2.0,
            screen_y + TILE_SIZE / 2.0,
            glow_size,
            Color { r: 1.0, g: 0.7, b: 0.3, a: 0.12 * self.glow_intensity }
        );

        // Select sprite based on direction
        let sprite = match self.direction {
            TorchDir::Top => &atlas.torch_top,
            TorchDir::Left => &atlas.torch_left,
            TorchDir::Right => &atlas.torch_right,
        };

        // Extract frame from spritesheet (16x16 frames, horizontal strip)
        let frame_w = 16.0;
        let frame_h = 16.0;
        let frame_count = (sprite.width() / frame_w).floor() as usize;
        let frame_idx = if frame_count == 0 {
            0
        } else {
            self.anim_frame % frame_count
        };

        draw_texture_ex(
            sprite,
            screen_x,
            screen_y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(frame_idx as f32 * frame_w, 0.0, frame_w, frame_h)),
                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
        );
    }
}
