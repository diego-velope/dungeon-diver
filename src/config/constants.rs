// Dungeon Diver - Constants
// All game configuration values in one place

use macroquad::prelude::*;

// ============================================================================
// SCREEN & DISPLAY
// ============================================================================

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;

// ============================================================================
// 🎮 TESTING CONTROLS - CHANGE THESE VALUES TO TEST DIFFERENT SCALES
// ============================================================================
//
// Try these TILE_SIZE values:
//   - 32.0 = original (40x20 level fits on screen)
//   - 48.0 = medium (27x15 level fits)
//   - 64.0 = large, recommended for TV (20x11 level fits) ← CURRENT
//
// When changing TILE_SIZE, the level size automatically adjusts via
// SCREEN_TILES_W/H - no need to manually calculate!
//
pub const TILE_SIZE: f32 = 64.0;
pub const TILE_SIZE_I: i32 = 64;

/// Seconds for gate crossfade from `doors_leaf_closed` to `doors_leaf_open`.
pub const GATE_OPEN_ANIM_DURATION: f32 = 0.5;

/// Tiled map tile size for levels using `dungeon_tileset_ii` (authoring pixels per cell).
pub const DUNGEON_TILESET_II_CELL_PX: f32 = 16.0;
/// Native pixel size of `doors_leaf_*.png` in that tileset (2×2 cells).
pub const DUNGEON_TILESET_II_DOOR_LEAF_PX: f32 = 32.0;
/// How many 16×16 cells wide a typical `doors_leaf_*` spans (place `exit` on the left cell).
pub const EXIT_DOOR_LEAF_TILE_SPAN_W: i32 = 2;

// Grid dimensions in tiles (auto-calculated - fits level on screen)
pub const SCREEN_TILES_W: i32 = (SCREEN_W / TILE_SIZE) as i32;  // 20 at 64px
pub const SCREEN_TILES_H: i32 = (SCREEN_H / TILE_SIZE) as i32;  // 11 at 64px

// ============================================================================
// 🗺️ LEVEL SIZE TESTING - CHANGE HERE TO TEST DIFFERENT LAYOUTS
// ============================================================================
// Smaller level = bigger individual tiles, larger level = smaller tiles
//
// Try these LEVEL1_W/H combinations:
//   - 12 x 12  = tiny, quick to test
//   - 16 x 16  = compact, good for development ← CURRENT
//   - 20 x 15  = fills most of 1280x720 at 64px tiles
//   - 40 x 20  = original full-screen layout (use with 32px tiles)
//
// IMPORTANT: If you change these, update the level layout string in
// src/level.rs to match! (search for "let layout = [")
//
pub const LEVEL1_W: usize = 16;
pub const LEVEL1_H: usize = 16;
pub const LEVEL_LARGE_W: usize = 20;
pub const LEVEL_LARGE_H: usize = 20;

// ============================================================================
// COLORS - LEVEL 1: Tutorial Dungeon (Gray/Blue Theme)
// ============================================================================

pub struct Palette {
    pub wall_top: Color,
    pub wall_side: Color,
    pub floor: Color,
    pub floor_alt: Color,
    pub accent: Color,
    pub bg_top: Color,
    pub bg_bot: Color,
    pub text: Color,
    pub text_shadow: Color,
}

// Helper to create normalized color from hex values
const fn norm_rgb(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub const LEVEL1_PALETTE: Palette = {
    let (wall_top_r, wall_top_g, wall_top_b) = norm_rgb(0x5A, 0x6A, 0x7A);
    let (wall_side_r, wall_side_g, wall_side_b) = norm_rgb(0x3A, 0x4A, 0x5A);
    let (floor_r, floor_g, floor_b) = norm_rgb(0x2A, 0x2A, 0x3A);
    let (floor_alt_r, floor_alt_g, floor_alt_b) = norm_rgb(0x25, 0x25, 0x35);
    let (accent_r, accent_g, accent_b) = norm_rgb(0x4A, 0x90, 0xD9);
    let (bg_top_r, bg_top_g, bg_top_b) = norm_rgb(0x1A, 0x1A, 0x2A);
    let (bg_bot_r, bg_bot_g, bg_bot_b) = norm_rgb(0x0A, 0x0A, 0x1A);

    Palette {
        wall_top:    Color { r: wall_top_r, g: wall_top_g, b: wall_top_b, a: 1.0 },
        wall_side:   Color { r: wall_side_r, g: wall_side_g, b: wall_side_b, a: 1.0 },
        floor:       Color { r: floor_r, g: floor_g, b: floor_b, a: 1.0 },
        floor_alt:   Color { r: floor_alt_r, g: floor_alt_g, b: floor_alt_b, a: 1.0 },
        accent:      Color { r: accent_r, g: accent_g, b: accent_b, a: 1.0 },
        bg_top:      Color { r: bg_top_r, g: bg_top_g, b: bg_top_b, a: 1.0 },
        bg_bot:      Color { r: bg_bot_r, g: bg_bot_g, b: bg_bot_b, a: 1.0 },
        text:        Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        text_shadow: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 },
    }
};

// UI Colors
pub const UI_BG: Color = Color { r: 0.1, g: 0.1, b: 0.15, a: 0.95 };
pub const UI_BORDER: Color = Color { r: 0.3, g: 0.4, b: 0.5, a: 1.0 };
pub const UI_HIGHLIGHT: Color = Color { r: 0.4, g: 0.7, b: 1.0, a: 1.0 };
pub const UI_SELECTED: Color = Color { r: 0.5, g: 0.8, b: 1.0, a: 1.0 };

// ============================================================================
// PLAYER
// ============================================================================

// Player speed scales with TILE_SIZE so movement feels consistent
// Formula: tiles per second * TILE_SIZE. 3.75 tiles/sec feels good.
pub const PLAYER_SPEED: f32 = 3.75 * TILE_SIZE;  // 240 at 64px, 120 at 32px

pub const PLAYER_ATTACK_COOLDOWN: f32 = 1.5;  // Seconds between attacks
pub const PLAYER_ATTACK_RANGE: f32 = TILE_SIZE;  // Attack range = 1 tile
pub const PLAYER_INVINCIBLE_TIME: f32 = 1.0;  // Seconds of invincibility after hit

// Player sprite size multiplier - change to test different player sizes:
//   - TILE_SIZE * 1.0 = same size as tile (compact) ← CURRENT
//   - TILE_SIZE * 1.5 = 1.5x tile size (balanced)
//   - TILE_SIZE * 2.0 = 2x tile size (best for TV visibility)
pub const PLAYER_DISPLAY_SIZE: f32 = TILE_SIZE;  // 64x64 at 64px tiles

// Zombie sheets use 32×32 frames but the figure sits small in the cell (lots of padding).
// Scale up so silhouettes read closer to the knight at PLAYER_DISPLAY_SIZE.
pub const ENEMY_DISPLAY_SIZE: f32 = TILE_SIZE * 1.5;

// Blue Knight sprite sheets: fixed cell size per asset filename (do not infer from width/frame guess).
// Frame count = floor(texture_width / cell_w) so strips with 2, 4, 6, etc. frames all work.
pub const KNIGHT_IDLE_FRAME_W: f32 = 16.0;
pub const KNIGHT_IDLE_FRAME_H: f32 = 16.0;
pub const KNIGHT_RUN_FRAME_W: f32 = 16.0;
pub const KNIGHT_RUN_FRAME_H: f32 = 17.0;

// Animation speeds
pub const PLAYER_IDLE_FRAME_TIME: f32 = 0.2;  // Seconds per idle frame
pub const PLAYER_RUN_FRAME_TIME: f32 = 0.08;  // Seconds per run frame

// ============================================================================
// GAMEPLAY
// ============================================================================

pub const MAX_LIVES: i32 = 3;
pub const STARTING_LIVES: i32 = 3;
// Health is measured in half-hearts:
// 3 full hearts = 6 half-hearts.
pub const MAX_HP: i32 = 6;
pub const STARTING_HP: i32 = 6;

pub const POTION_HEAL: i32 = 1;  // HP restored by potion
pub const MAX_SHIELD_CHARGES: i32 = 3;
// Gold coin = 1 unit, blue coin = 2 units (see ItemType::BlueCoin).
pub const COIN_VALUE: i32 = 1;

// ============================================================================
// ENEMIES
// ============================================================================

pub const ENEMY_HP: i32 = 3;                    // Zombies take 3 hits to kill
pub const ENEMY_MOVE_INTERVAL: f32 = 0.5;       // Move every 0.5 seconds
pub const ENEMY_ACTIVATION_RANGE: i32 = 5;      // Only chase if within 5 tiles
pub const ENEMY_DAMAGE: i32 = 1;                // Zombies deal 1 damage on contact
pub const ENEMY_ATTACK_COOLDOWN: f32 = 3.0;     // Seconds between enemy attacks
pub const BIG_ZOMBIE_HP: i32 = 5;
pub const BIG_ZOMBIE_MOVE_INTERVAL: f32 = 0.7;
pub const BIG_ZOMBIE_DAMAGE: i32 = 1;
pub const BIG_ZOMBIE_ATTACK_COOLDOWN: f32 = 3.5;
pub const BIG_ZOMBIE_ACTIVATION_RANGE: i32 = 5;
pub const BIG_ZOMBIE_DISPLAY_SIZE: f32 = TILE_SIZE * 1.75;

pub const BIG_DEMON_HP: i32 = 4;
pub const BIG_DEMON_MOVE_INTERVAL: f32 = 0.35;
pub const BIG_DEMON_DAMAGE: i32 = 2;
pub const BIG_DEMON_ATTACK_COOLDOWN: f32 = 2.5;
pub const BIG_DEMON_ACTIVATION_RANGE: i32 = 7;
pub const BIG_DEMON_DISPLAY_SIZE: f32 = TILE_SIZE * 1.8;

// ============================================================================
// COMBAT VFX
// ============================================================================

pub const HIT_VFX_PLAYER_FRAME_TIME: f32 = 0.07;
pub const HIT_VFX_ENEMY_FRAME_TIME: f32 = 0.06;
pub const HIT_VFX_SCALE: f32 = 1.0;

// ============================================================================
// HUD - ATTACK COOLDOWN BAR
// ============================================================================

pub const HUD_ATTACK_BAR_FRAME_W: f32 = 220.0;
pub const HUD_ATTACK_BAR_FRAME_H: f32 = 48.0;
pub const HUD_ATTACK_BAR_INSET_X: f32 = 14.0;
pub const HUD_ATTACK_BAR_INSET_Y: f32 = 11.0;
pub const HUD_ATTACK_BAR_FILL_W: f32 = 192.0;
pub const HUD_ATTACK_BAR_FILL_H: f32 = 25.0;
pub const HUD_SHIELD_BAR_FRAME_W: f32 = 150.0;
pub const HUD_SHIELD_BAR_FRAME_H: f32 = 36.0;
pub const HUD_SHIELD_BAR_INSET_X: f32 = 11.0;
pub const HUD_SHIELD_BAR_INSET_Y: f32 = 8.0;

// ============================================================================
// CAMERA
// ============================================================================

pub const CAMERA_SMOOTH: f32 = 0.15;  // Camera follow smoothing (0-1)
pub const CAMERA_OFFSET_Y: f32 = 0.0;  // Y offset from player center

// ============================================================================
// INPUT
// ============================================================================

pub const HOLD_THRESHOLD: f32 = 0.2;  // Seconds before a hold is registered
pub const INPUT_REPEAT_DELAY: f32 = 0.15;  // Delay between repeated inputs when held

// ============================================================================
// PARTICLES & EFFECTS
// ============================================================================

pub const MAX_PARTICLES: usize = 100;
pub const PARTICLE_LIFESPAN: f32 = 0.5;
pub const SCREEN_SHAKE_DURATION: f32 = 0.15;
pub const SCREEN_SHAKE_INTENSITY: f32 = 3.0;
/// Full-screen red flash intensity (0–1); paired with `Camera::shake` on real damage.
pub const HURT_REACTION_FLASH: f32 = 0.26;
/// Slices of `invincible_time` / shatter-windup time where the sprite is hidden (see `Player::draw`).
pub const IFRAME_FLASH_SLICE: f32 = 0.1;
/// Pre-break: vase flickers (same as player iframes) for this long, then shatters and drops coins.
pub const VASE_SHATTER_WINDUP: f32 = 0.32;

// ============================================================================
// TEXT SIZES (TV-optimized, 10ft viewing)
// ============================================================================

pub const TEXT_TITLE: u16 = 64;
pub const TEXT_LARGE: u16 = 48;
pub const TEXT_MEDIUM: u16 = 36;
pub const TEXT_NORMAL: u16 = 28;
pub const TEXT_SMALL: u16 = 22;

// ============================================================================
// MENU LAYOUT
// ============================================================================

pub const MENU_ITEM_HEIGHT: f32 = 60.0;
pub const MENU_PADDING: f32 = 20.0;
pub const TITLE_MENU_BUTTON_W: f32 = 300.0;
pub const TITLE_MENU_BUTTON_H: f32 = 70.0;
pub const TITLE_MENU_BUTTON_GAP: f32 = 20.0;
/// Y position of the top edge of the first main-menu button (below baked-in title art).
pub const TITLE_MENU_START_Y: f32 = 300.0;
/// ThaleahFat size for labels on title menu buttons (matches mockup ~28px).
pub const TITLE_MENU_FONT_SIZE: u16 = 28;

/// Centered dark square behind pause menu title + buttons.
pub const PAUSE_MENU_PANEL_SIDE: f32 = 480.0;
pub const PAUSE_MENU_BUTTON_W: f32 = 280.0;
pub const PAUSE_MENU_BUTTON_H: f32 = 64.0;
pub const PAUSE_MENU_BUTTON_GAP: f32 = 12.0;
