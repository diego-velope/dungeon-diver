// Dungeon Diver — terrain rendering using 0x72 DungeonTileset II v1.7
//
// All source sprites are 16×16 px native, drawn at 2× → TILE_SIZE px.
// (TILE_SIZE=64 means we scale 16→64 = 4×, handled by dest_size below)

use macroquad::prelude::*;
use crate::constants::TILE_SIZE;

pub struct TerrainAtlas {
    // ── floors ──────────────────────────────────────────────────────────────
    pub floors: [Texture2D; 8],

    // ── wall sprites (only 2 base sprites!) ──────────────────────────────────────
    // We compose walls by rotating wall_top_mid to different angles
    pub wall_mid:       Texture2D,  // brick face - used for all wall bases
    pub wall_top_mid:   Texture2D,  // top surface - can be rotated 0°, 90°, 180°, 270°

    // ── exit door ───────────────────────────────────────────────────────────
    pub door_leaf_closed: Texture2D,
    pub door_leaf_open:   Texture2D,
    pub door_frame_top:   Texture2D,
}

impl TerrainAtlas {
    pub async fn load() -> Option<Self> {
        let p = "assets/0x72";

        macro_rules! tex {
            ($name:expr) => {{
                let t = load_texture(&format!("{}/{}.png", p, $name)).await.ok()?;
                t.set_filter(FilterMode::Nearest);
                t
            }};
        }

        Some(Self {
            floors: [
                tex!("floor_1"), tex!("floor_2"), tex!("floor_3"), tex!("floor_4"),
                tex!("floor_5"), tex!("floor_6"), tex!("floor_7"), tex!("floor_8"),
            ],
            wall_mid:         tex!("wall_mid"),
            wall_top_mid:     tex!("wall_top_mid"),
            door_leaf_closed: tex!("doors_leaf_closed"),
            door_leaf_open:   tex!("doors_leaf_open"),
            door_frame_top:   tex!("doors_frame_top"),
        })
    }

    // ── internal draw helpers ────────────────────────────────────────────────

    /// Draw a texture with rotation (in degrees) and scale.
    /// pivot is (0.5, 0.5) = center of texture.
    fn draw_rotated(&self, tex: &Texture2D, sx: f32, sy: f32, rotation: f32) {
        draw_texture_ex(
            tex, sx, sy, WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                rotation: rotation.to_radians(),
                pivot: Some(vec2(0.5, 0.5)),  // Rotate around center
                ..Default::default()
            },
        );
    }

    /// Draw a 16×16 sprite scaled up to TILE_SIZE × TILE_SIZE.
    /// Optional rotation in degrees (0, 90, 180, 270).
    fn draw_tile(&self, tex: &Texture2D, sx: f32, sy: f32, rotation: f32) {
        if rotation == 0.0 {
            // No rotation - faster path
            draw_texture_ex(
                tex, sx, sy, WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );
        } else {
            // With rotation
            draw_texture_ex(
                tex, sx, sy, WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                    rotation: rotation.to_radians(),
                    pivot: Some(vec2(0.5, 0.5)),  // Rotate around center
                    ..Default::default()
                },
            );
        }
    }

    /// Convenience method for no rotation (0°).
    fn draw_tile_0(&self, tex: &Texture2D, sx: f32, sy: f32) {
        self.draw_tile(tex, sx, sy, 0.0);
    }

    fn draw_sized(&self, tex: &Texture2D, sx: f32, sy: f32, w: f32, h: f32) {
        draw_texture_ex(
            tex, sx, sy, WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
    }

    // ── public API (called from level.rs) ────────────────────────────────────

    /// Stable pseudo-random floor tile per world coordinate.
    pub fn draw_floor(&self, gx: i32, gy: i32, sx: f32, sy: f32) {
        let h = ((gx.wrapping_mul(2971) ^ gy.wrapping_mul(1619)) as u32)
            .wrapping_add(gx.wrapping_mul(gy) as u32);
        self.draw_tile(&self.floors[(h as usize) % self.floors.len()], sx, sy, 0.0);
    }

    /// Draw wall using ROTATION degrees for clean composition.
    ///
    /// ROTATION SYSTEM:
    ///   0°   = south view (shows bottom/south cap)
    ///   90°  = west view (shows left/east side face)
    ///   180° = north view (shows top/north cap)
    ///   270° = east view (shows right/west side face)
    ///
    /// KEY COMBINATIONS:
    ///   #  = wall_mid at 0°
    ///   #-  = wall_mid 0° + wall_top_mid 0° (bottom cap)
    ///   #+  = wall_mid 0° + wall_top_mid 180° (top cap)
    ///   #|  = wall_mid 0° + wall_top_mid 270° (right side face)
    ///   |#  = wall_mid 0° + wall_top_mid 90° (left side face)
    pub fn draw_wall(&self, tile: crate::level::WallSprite, sx: f32, sy: f32) {
        use crate::level::WallSprite;

        match tile {
            WallSprite::Floor => return,
            WallSprite::Door => return,

            WallSprite::Mid => {
                self.draw_tile_0(&self.wall_mid, sx, sy);
            }

            WallSprite::Left => {
                // |# = left side face = 90° rotation
                self.draw_tile_0(&self.wall_mid, sx, sy);
                self.draw_tile(&self.wall_top_mid, sx, sy, 90.0);
            }

            WallSprite::Right => {
                // #| = right side face = 270° rotation
                self.draw_tile_0(&self.wall_mid, sx, sy);
                self.draw_tile(&self.wall_top_mid, sx, sy, 270.0);
            }

            WallSprite::TopMid => {
                // _ = bottom cap = 0°
                self.draw_tile(&self.wall_top_mid, sx, sy, 0.0);
            }

            WallSprite::TopLeft => {
                // _| = 0° (cap) + 90° (left side)
                self.draw_tile(&self.wall_top_mid, sx, sy, 0.0);
                self.draw_tile(&self.wall_top_mid, sx, sy, 90.0);
            }

            WallSprite::TopRight => {
                // _ = 0° (cap) + 270° (right side)
                self.draw_tile(&self.wall_top_mid, sx, sy, 0.0);
                self.draw_tile(&self.wall_top_mid, sx, sy, 270.0);
            }

            WallSprite::BottomMid => {
                // #- = solid + bottom cap = 0° + 0°
                self.draw_tile_0(&self.wall_mid, sx, sy);
                self.draw_tile(&self.wall_top_mid, sx, sy, 0.0);
            }
        }
    }

    /// Draw the exit door tile.
    /// door_leaf is 32×32 native → drawn at TILE_SIZE×TILE_SIZE.
    /// door_frame_top is 32×16 native → drawn at TILE_SIZE×(TILE_SIZE*0.5) above.
    pub fn draw_door(&self, sx: f32, sy: f32, unlocked: bool) {
        // Floor base
        self.draw_tile(&self.floors[0], sx, sy, 0.0);

        // Door leaf
        let leaf = if unlocked { &self.door_leaf_open } else { &self.door_leaf_closed };
        self.draw_sized(leaf, sx, sy, TILE_SIZE, TILE_SIZE);

        // Arch frame above the tile
        let frame_h = TILE_SIZE * 0.5;
        self.draw_sized(&self.door_frame_top, sx, sy - frame_h, TILE_SIZE, frame_h);
    }
}
