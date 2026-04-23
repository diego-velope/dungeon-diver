use std::collections::HashMap;

use macroquad::prelude::*;

use crate::config::{SCREEN_TILES_H, SCREEN_TILES_W, TILE_SIZE};

/// Maps Tiled tile flip flags to macroquad [`DrawTextureParams`] `rotation` (radians) and mirrors.
/// Follows the usual TMX + [`tiled::LayerTileData`] model and the cocos2d-x `CCTMXLayer::setupTileSprite` split
/// for the diagonal case (rotation + optional `flip_x` when the diagonal bit is set).
pub fn tiled_flips_to_rotation_and_mirror(
    flip_h: bool,
    flip_v: bool,
    flip_d: bool,
) -> (f32, bool, bool) {
    if !flip_d {
        return (0.0, flip_h, flip_v);
    }
    // When `flip_d` is on, the other two select among 90°/270° and variants (Tiled has no free rotation).
    const R90: f32 = std::f32::consts::FRAC_PI_2;
    const R270: f32 = 3.0 * std::f32::consts::FRAC_PI_2;
    match (flip_h, flip_v) {
        (true, false) => (R90, false, false),
        (false, true) => (R270, false, false),
        (true, true) => (R90, true, false),
        (false, false) => (R270, true, false),
    }
}

/// A single animation frame.
///
/// For spritesheet tilesets all frames share the same `image_path` but have
/// different `src` rects. For image-collection tilesets every frame can have
/// its own `image_path`; `src` covers the full image (origin 0,0).
#[derive(Clone, Debug)]
pub struct AnimFrame {
    /// Normalized game-relative path to the PNG for this frame.
    pub image_path: String,
    /// Source rectangle inside the PNG (source pixels).
    pub src: Rect,
    /// Duration in milliseconds. 0 for static (single-frame) tiles.
    pub duration_ms: u32,
}

/// Per-cell sprite extracted from a TMX tile layer.
///
/// Static tiles have exactly one frame with `duration_ms = 0`.
/// Animated tiles have ≥2 frames with the durations from the TSX `<animation>`.
#[derive(Clone, Debug)]
pub struct CellSprite {
    /// Animation frames (len ≥ 1). Single entry → static tile.
    pub frames: Vec<AnimFrame>,
    /// Sum of all frame durations in ms. 0 for static tiles.
    pub total_duration_ms: u64,
    pub flip_h: bool,
    pub flip_v: bool,
    /// Tiled "flip diagonally" (TMX). Needed for 90°/270° style orientation; it is *not* only H/V.
    pub flip_d: bool,
}

impl CellSprite {
    /// Returns the frame active at `time_ms` milliseconds since epoch.
    pub fn current_frame(&self, time_ms: u64) -> &AnimFrame {
        if self.frames.len() == 1 || self.total_duration_ms == 0 {
            return &self.frames[0];
        }
        let t = time_ms % self.total_duration_ms;
        let mut elapsed = 0u64;
        for frame in &self.frames {
            elapsed += frame.duration_ms as u64;
            if t < elapsed {
                return frame;
            }
        }
        self.frames.last().unwrap()
    }
}

/// A single tile layer extracted from a TMX map.
#[derive(Clone)]
pub struct TiledLayerRaw {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Row-major grid: `cells[y * width + x]`. `None` means empty cell.
    pub cells: Vec<Option<CellSprite>>,
}

impl TiledLayerRaw {
    pub fn get(&self, x: i32, y: i32) -> Option<&CellSprite> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        self.cells[y as usize * self.width as usize + x as usize].as_ref()
    }
}

/// All visual tile layers extracted from a TMX map. No GPU resources — plain data.
#[derive(Clone)]
pub struct TiledVisualRaw {
    pub map_width: u32,
    pub map_height: u32,
    /// Tile width in the source PNG (e.g. 16 px).
    pub tile_width: u32,
    /// Tile height in the source PNG (e.g. 16 px).
    pub tile_height: u32,
    /// Layers in draw order (floor, then walls, then decoration).
    pub layers: Vec<TiledLayerRaw>,
}

impl TiledVisualRaw {
    /// Returns all unique, normalized image paths referenced by any frame of any cell.
    pub fn image_paths(&self) -> Vec<String> {
        let mut paths: std::collections::BTreeSet<String> = Default::default();
        for layer in &self.layers {
            for cell in layer.cells.iter().flatten() {
                for frame in &cell.frames {
                    paths.insert(frame.image_path.clone());
                }
            }
        }
        paths.into_iter().collect()
    }
}

/// Ready-to-draw Tiled visual map (owns GPU texture handles).
pub struct TiledVisualMap {
    raw: TiledVisualRaw,
    textures: HashMap<String, Texture2D>,
}

impl TiledVisualMap {
    pub fn build(raw: TiledVisualRaw, textures: HashMap<String, Texture2D>) -> Self {
        Self { raw, textures }
    }

    /// Authoring tile size from TMX (usually 16×16). Used to scale sprites whose PNG
    /// is wider/taller than one cell (e.g. 32×16 lintel, 32×32 door).
    #[inline]
    fn map_tile_w(&self) -> f32 {
        self.raw.tile_width as f32
    }

    #[inline]
    fn map_tile_h(&self) -> f32 {
        self.raw.tile_height as f32
    }

    #[inline]
    /// Skip `vase_shine` tile: cell is smashed, or pre-break flicker (same list as `Level::vase_shine_tiled_skips`).
    fn skip_tiled_vase_shine(x: i32, y: i32, image_path: &str, vase_shine_skips: &[(i32, i32)]) -> bool {
        image_path.contains("vase_shine_anim")
            && vase_shine_skips.iter().any(|&(bx, by)| bx == x && by == y)
    }

    #[inline]
    fn is_torch_layer(name: &str) -> bool {
        name.eq_ignore_ascii_case("torchs") || name.eq_ignore_ascii_case("torches")
    }

    /// Tall art on the `columns` layer (e.g. 16×48 `column.png` / `column_wall.png`) always sorts
    /// **under** the player — no Y-sort occlusion; the knight stays in front of the full column quad.
    #[inline]
    fn is_columns_layer(name: &str) -> bool {
        name.eq_ignore_ascii_case("columns")
    }

    /// 16px sconce art from `decoration/torch/...` on `decoration` (or any non-`torches` layer) is
    /// skipped in `draw` and painted in `draw_sconce_overlay` **after** tall `column` quads in the
    /// foreground, so the column is not drawn on top of the mount/flame. The dedicated `torches`
    /// layer is not deferred here; it is still handled by `draw_foreground_pass` (two sub-passes).
    #[inline]
    fn is_deferred_sconce_path(path: &str) -> bool {
        path.contains("decoration/torch")
    }

    /// Draw all tile layers, camera-culled to visible screen area.
    /// `tiled_vase_shine_skips`: grid cells to hide `vase_shine` (smashed or pre-break flicker off-phase).
    pub fn draw(&self, camera_x: f32, camera_y: f32, tiled_vase_shine_skips: &[(i32, i32)]) {
        let map_w = self.raw.map_width as i32;
        let map_h = self.raw.map_height as i32;

        let start_x = ((camera_x / TILE_SIZE).floor() as i32 - 1).max(0);
        let start_y = ((camera_y / TILE_SIZE).floor() as i32 - 1).max(0);
        let end_x = (start_x + SCREEN_TILES_W + 2).min(map_w);
        let end_y = (start_y + SCREEN_TILES_H + 2).min(map_h);

        // Shared animation clock: milliseconds since start, wraps after ~49 days.
        let time_ms = (get_time() * 1000.0) as u64;

        for layer in &self.raw.layers {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    let Some(cell) = layer.get(x, y) else { continue };
                    let frame = cell.current_frame(time_ms);
                    if Self::skip_tiled_vase_shine(x, y, &frame.image_path, tiled_vase_shine_skips) {
                        continue;
                    }
                    let is_torch_layer = layer.name.eq_ignore_ascii_case("torchs")
                        || layer.name.eq_ignore_ascii_case("torches");
                    if is_torch_layer {
                        // Torch tile layers are depth-sorted in foreground passes,
                        // otherwise columns can occlude them completely.
                        continue;
                    }
                    if Self::is_deferred_sconce_path(&frame.image_path) {
                        // Tall `column` on `walls` is drawn in foreground passes; drawing sconces here
                        // would be covered by the column. See `draw_sconce_overlay`.
                        continue;
                    }
                    if frame.src.h > self.map_tile_h() {
                        // Tall sprites are rendered in the foreground pass so
                        // the player can walk "behind" them.
                        continue;
                    }
                    let Some(tex) = self.textures.get(&frame.image_path) else { continue };

                    let tw = self.map_tile_w();
                    let th = self.map_tile_h();
                    let dest_w = TILE_SIZE * (frame.src.w / tw);
                    let dest_h = TILE_SIZE * (frame.src.h / th);

                    let screen_x = x as f32 * TILE_SIZE - camera_x;
                    let screen_y = y as f32 * TILE_SIZE - camera_y;
                    // Tiled aligns oversized sprites to the bottom of the cell (orthogonal).
                    let draw_x = screen_x;
                    let draw_y = screen_y + TILE_SIZE - dest_h;
                    let (rot, mirror_x, mirror_y) = tiled_flips_to_rotation_and_mirror(
                        cell.flip_h,
                        cell.flip_v,
                        cell.flip_d,
                    );

                    draw_texture_ex(
                        tex,
                        draw_x,
                        draw_y,
                        WHITE,
                        DrawTextureParams {
                            source: Some(frame.src),
                            dest_size: Some(vec2(dest_w, dest_h)),
                            rotation: rot,
                            flip_x: mirror_x,
                            flip_y: mirror_y,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// 16px sconce tiles whose paths match `is_deferred_sconce_path` (authored on e.g. `decoration`
    /// above a `column` on `walls`). Call **after** `draw_foreground_after_player` so tall column
    /// quads are under the mount/flame.
    pub fn draw_sconce_overlay(
        &self,
        camera_x: f32,
        camera_y: f32,
        tiled_vase_shine_skips: &[(i32, i32)],
    ) {
        let map_w = self.raw.map_width as i32;
        let map_h = self.raw.map_height as i32;
        let start_x = ((camera_x / TILE_SIZE).floor() as i32 - 1).max(0);
        let start_y = ((camera_y / TILE_SIZE).floor() as i32 - 1).max(0);
        let end_x = (start_x + SCREEN_TILES_W + 2).min(map_w);
        let end_y = (start_y + SCREEN_TILES_H + 2).min(map_h);
        let time_ms = (get_time() * 1000.0) as u64;

        for layer in &self.raw.layers {
            if Self::is_torch_layer(&layer.name) {
                continue;
            }
            for y in start_y..end_y {
                for x in start_x..end_x {
                    let Some(cell) = layer.get(x, y) else { continue };
                    let frame = cell.current_frame(time_ms);
                    if Self::skip_tiled_vase_shine(x, y, &frame.image_path, tiled_vase_shine_skips) {
                        continue;
                    }
                    if !Self::is_deferred_sconce_path(&frame.image_path) {
                        continue;
                    }
                    if frame.src.h > self.map_tile_h() {
                        continue;
                    }
                    let Some(tex) = self.textures.get(&frame.image_path) else { continue };
                    let tw = self.map_tile_w();
                    let th = self.map_tile_h();
                    let dest_w = TILE_SIZE * (frame.src.w / tw);
                    let dest_h = TILE_SIZE * (frame.src.h / th);
                    let screen_x = x as f32 * TILE_SIZE - camera_x;
                    let screen_y = y as f32 * TILE_SIZE - camera_y;
                    let draw_x = screen_x;
                    let draw_y = screen_y + TILE_SIZE - dest_h;
                    let (rot, mirror_x, mirror_y) = tiled_flips_to_rotation_and_mirror(
                        cell.flip_h,
                        cell.flip_v,
                        cell.flip_d,
                    );
                    draw_texture_ex(
                        tex,
                        draw_x,
                        draw_y,
                        WHITE,
                        DrawTextureParams {
                            source: Some(frame.src),
                            dest_size: Some(vec2(dest_w, dest_h)),
                            rotation: rot,
                            flip_x: mirror_x,
                            flip_y: mirror_y,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Tall tiles (height greater than map tile height) drawn **before** the player: tile base
    /// north of the player, or on the same row and **west** of the player (player walks in front
    /// when south or east of that tile).
    pub fn draw_foreground_before_player(
        &self,
        camera_x: f32,
        camera_y: f32,
        player_grid_x: i32,
        player_grid_y: i32,
        door_unlocked: bool,
        door_grid_x: i32,
        door_grid_y: i32,
        tiled_vase_shine_skips: &[(i32, i32)],
    ) {
        self.draw_foreground_pass(
            camera_x,
            camera_y,
            player_grid_x,
            player_grid_y,
            true,
            door_unlocked,
            door_grid_x,
            door_grid_y,
            tiled_vase_shine_skips,
        );
    }

    /// Tall tiles drawn **after** the player: base **south** of the player, or on the same row
    /// **east** of the player.
    pub fn draw_foreground_after_player(
        &self,
        camera_x: f32,
        camera_y: f32,
        player_grid_x: i32,
        player_grid_y: i32,
        door_unlocked: bool,
        door_grid_x: i32,
        door_grid_y: i32,
        tiled_vase_shine_skips: &[(i32, i32)],
    ) {
        self.draw_foreground_pass(
            camera_x,
            camera_y,
            player_grid_x,
            player_grid_y,
            false,
            door_unlocked,
            door_grid_x,
            door_grid_y,
            tiled_vase_shine_skips,
        );
    }

    /// Orthogonal top-down: a tall tile is drawn *before* the player when the player should walk
    /// in front: tile base is strictly **north** (`ty < py`) or same row and **west** of the
    /// player (`ty == py && tx < px`). Bases on the same row to the **east** (`tx > px`) or south
    /// are drawn in the *after* pass.
    #[inline]
    fn tall_sorts_before_player(tx: i32, ty: i32, px: i32, py: i32) -> bool {
        ty < py || (ty == py && tx < px)
    }

    /// When the door is unlocked we draw `doors_leaf_open` in `Level::draw_exit_door_unlock_overlay`.
    /// Skip TMX `doors_leaf_closed` over the **door** span so the open texture is not covered.
    fn skip_tmx_closed_door_leaf_for_door_overlay(
        door_unlocked: bool,
        tx: i32,
        _ty: i32,
        door_x: i32,
        _door_y: i32,
        image_path: &str,
    ) -> bool {
        if !door_unlocked {
            return false;
        }
        if !image_path.contains("doors_leaf_closed") {
            return false;
        }
        tx >= door_x && tx < door_x + crate::config::EXIT_DOOR_LEAF_TILE_SPAN_W
    }

    fn draw_foreground_pass(
        &self,
        camera_x: f32,
        camera_y: f32,
        player_grid_x: i32,
        player_grid_y: i32,
        before_player: bool,
        door_unlocked: bool,
        door_grid_x: i32,
        door_grid_y: i32,
        tiled_vase_shine_skips: &[(i32, i32)],
    ) {
        let map_w = self.raw.map_width as i32;
        let map_h = self.raw.map_height as i32;

        let start_x = ((camera_x / TILE_SIZE).floor() as i32 - 1).max(0);
        let start_y = ((camera_y / TILE_SIZE).floor() as i32 - 2).max(0);
        let end_x = (start_x + SCREEN_TILES_W + 2).min(map_w);
        let end_y = (start_y + SCREEN_TILES_H + 3).min(map_h);

        let time_ms = (get_time() * 1000.0) as u64;

        // Tall quads first, then the dedicated `torches` / `torchs` layer, so 16px torches in that
        // layer are not covered by a tall on the same row. Torch art on e.g. `decoration` is drawn
        // in the base pass in map layer order (columns on `walls`, then antorcha on `decoration`) and
        // is not player depth-sorted — so it will not "vanish" under the player like fg-sorted quads.
        for pass_torchs_only in [false, true] {
            for layer in &self.raw.layers {
                let is_torch_layer = Self::is_torch_layer(&layer.name);
                if is_torch_layer != pass_torchs_only {
                    continue;
                }
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let Some(cell) = layer.get(x, y) else { continue };
                        let frame = cell.current_frame(time_ms);
                        if Self::skip_tiled_vase_shine(x, y, &frame.image_path, tiled_vase_shine_skips) {
                            continue;
                        }
                        if frame.src.h <= self.map_tile_h() && !is_torch_layer {
                            continue;
                        }
                        if Self::skip_tmx_closed_door_leaf_for_door_overlay(
                            door_unlocked,
                            x,
                            y,
                            door_grid_x,
                            door_grid_y,
                            &frame.image_path,
                        ) {
                            continue;
                        }
                        // `columns` (16×48 etc.): always under the player — never use Y-sort for these quads.
                        let draws_before = if Self::is_columns_layer(&layer.name) {
                            true
                        } else {
                            Self::tall_sorts_before_player(x, y, player_grid_x, player_grid_y)
                        };
                        if before_player != draws_before {
                            continue;
                        }
                        let Some(tex) = self.textures.get(&frame.image_path) else { continue };

                        let width_scale = frame.src.w / self.map_tile_w();
                        let height_scale = frame.src.h / self.map_tile_h();
                        let dest_w = TILE_SIZE * width_scale;
                        let dest_h = TILE_SIZE * height_scale;

                        let screen_x = x as f32 * TILE_SIZE - camera_x;
                        let screen_y = y as f32 * TILE_SIZE - camera_y;
                        let draw_y = screen_y + TILE_SIZE - dest_h;
                        let (rot, mirror_x, mirror_y) = tiled_flips_to_rotation_and_mirror(
                            cell.flip_h,
                            cell.flip_v,
                            cell.flip_d,
                        );

                        draw_texture_ex(
                            tex,
                            screen_x,
                            draw_y,
                            WHITE,
                            DrawTextureParams {
                                source: Some(frame.src),
                                dest_size: Some(vec2(dest_w, dest_h)),
                                rotation: rot,
                                flip_x: mirror_x,
                                flip_y: mirror_y,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }
}
