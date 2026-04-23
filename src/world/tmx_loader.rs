#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::io::Cursor;
use std::path::Path;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, OnceLock};

use macroquad::prelude::Rect;
use tiled::{LayerType, Loader, Map, ObjectShape, PropertyValue};

use crate::config::{LEVEL1_H, LEVEL1_PALETTE, LEVEL1_W};
use crate::world::tiled_visual::{AnimFrame, CellSprite, TiledLayerRaw, TiledVisualRaw};
use crate::world::{
    Chest, Enemy, EnemyKind, FloorButton, GateEntity, Item, ItemType, Level, Tile, Torch, TorchDir, Vase,
};

#[cfg(target_arch = "wasm32")]
static WASM_LEVEL_TMX_RESOURCES: OnceLock<Arc<HashMap<String, Vec<u8>>>> = OnceLock::new();

/// XML files the `tiled` loader reads for level 1 (map → external TSX files).
/// Must stay in sync with `assets/levels/level1.tmx` tileset sources.
/// PNGs are NOT read during parse — they are loaded separately by macroquad.
/// Used only on WASM (native uses std::fs directly).
#[cfg(target_arch = "wasm32")]
const TMX_WASM_PRELOADS: &[&str] = &[
    "assets/levels/level1.tmx",
    "assets/levels/level2.tmx",
    "assets/levels/dungeon_tileset_ii.tsx",
    "assets/levels/decoration.tsx",
    "assets/levels/columns.tsx",
    "assets/levels/vase_shine_anim.tsx",
    "assets/levels/abyss.tsx",
    "assets/levels/torch_default_tileset.tsx",
    "assets/levels/torch_left_tileset.tsx",
    "assets/levels/torch_right_tileset.tsx",
];

/// Tile layers from the TMX that participate in `TiledVisualRaw` / GPU draw.
/// - `torchs` / `torches` (and ASCII case): legacy naming.
/// - `columns`: optional layer for tall column props (often same tilesets as `decoration`).
fn is_tiled_visual_layer(name: &str) -> bool {
    matches!(
        name,
        "floor"
            | "walls"
            | "abyss"
            | "decoration"
            | "decor"
            | "columns"
            | "torchs"
            | "torches"
    ) || name.eq_ignore_ascii_case("torchs")
        || name.eq_ignore_ascii_case("torches")
        || name.eq_ignore_ascii_case("columns")
}

/// Collapse `..` components in a path and convert separators to `/`.
/// Produces a normalized, game-relative path like "assets/tileset/foo/bar.png".
fn normalize_path(path: &Path) -> String {
    let mut components: Vec<String> = Vec::new();
    for comp in path.components() {
        use std::path::Component;
        match comp {
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(s) => components.push(s.to_string_lossy().into_owned()),
            Component::CurDir => {}
            _ => {}
        }
    }
    components.join("/")
}

/// Extract all visual tile layers from an already-parsed `tiled::Map`.
///
/// Only standard visual layers are included (see `is_tiled_visual_layer`).
/// Image-collection tilesets (no single sheet) are skipped gracefully.
pub fn build_tiled_visual_raw(map: &Map) -> TiledVisualRaw {
    let map_w = map.width;
    let map_h = map.height;
    let tile_w = map.tile_width;
    let tile_h = map.tile_height;
    let cell_count = (map_w * map_h) as usize;

    let mut layers: Vec<TiledLayerRaw> = Vec::new();

    for layer in map.layers() {
        let LayerType::Tiles(tile_layer) = layer.layer_type() else { continue };
        if !is_tiled_visual_layer(layer.name.as_str()) {
            continue;
        }

        let mut cells: Vec<Option<CellSprite>> = vec![None; cell_count];

        for y in 0..map_h as i32 {
            for x in 0..map_w as i32 {
                let Some(t) = tile_layer.get_tile(x, y) else { continue };
                let tileset = t.get_tileset();

                let maybe_cell = if let Some(ref sheet_img) = tileset.image {
                    // ── Spritesheet tileset ──────────────────────────────────
                    let cols = tileset.columns;
                    if cols == 0 {
                        continue;
                    }
                    let sheet_path = normalize_path(&sheet_img.source);

                    let src_for = |id: u32| -> Rect {
                        let col = (id % cols) as f32;
                        let row = (id / cols) as f32;
                        let m = tileset.margin as f32;
                        let s = tileset.spacing as f32;
                        let src_x = col * (tile_w as f32 + s) + m;
                        let src_y = row * (tile_h as f32 + s) + m;
                        Rect::new(src_x, src_y, tile_w as f32, tile_h as f32)
                    };

                    let anim_def = t.get_tile().and_then(|tile| tile.animation.clone());
                    let (frames, total_ms) = if let Some(anim_frames) = anim_def {
                        let total: u64 = anim_frames.iter().map(|f| f.duration as u64).sum();
                        let frames: Vec<AnimFrame> = anim_frames
                            .iter()
                            .map(|f| AnimFrame {
                                image_path: sheet_path.clone(),
                                src: src_for(f.tile_id),
                                duration_ms: f.duration,
                            })
                            .collect();
                        (frames, total)
                    } else {
                        (
                            vec![AnimFrame {
                                image_path: sheet_path,
                                src: src_for(t.id()),
                                duration_ms: 0,
                            }],
                            0,
                        )
                    };

                    Some(CellSprite {
                        frames,
                        total_duration_ms: total_ms,
                        flip_h: t.flip_h,
                        flip_v: t.flip_v,
                        flip_d: t.flip_d,
                    })
                } else {
                    // ── Image-collection tileset (each tile is its own PNG) ──
                    let anim_def = t.get_tile().and_then(|tile| tile.animation.clone());
                    if let Some(anim_frames) = anim_def {
                        let total: u64 = anim_frames.iter().map(|f| f.duration as u64).sum();
                        let mut frames: Vec<AnimFrame> = Vec::with_capacity(anim_frames.len());
                        for f in &anim_frames {
                            let Some(frame_tile) = tileset.get_tile(f.tile_id) else { continue };
                            let Some(ref frame_img) = frame_tile.image else { continue };
                            let image_path = normalize_path(&frame_img.source);
                            let src = Rect::new(0.0, 0.0, frame_img.width as f32, frame_img.height as f32);
                            frames.push(AnimFrame { image_path, src, duration_ms: f.duration });
                        }
                        if frames.is_empty() {
                            None
                        } else {
                            Some(CellSprite {
                                frames,
                                total_duration_ms: total,
                                flip_h: t.flip_h,
                                flip_v: t.flip_v,
                                flip_d: t.flip_d,
                            })
                        }
                    } else {
                        // Static tile in image-collection tileset.
                        if let Some(tile) = t.get_tile() {
                            if let Some(ref tile_img) = tile.image {
                                let image_path = normalize_path(&tile_img.source);
                                let src = Rect::new(0.0, 0.0, tile_img.width as f32, tile_img.height as f32);
                                Some(CellSprite {
                                    frames: vec![AnimFrame { image_path, src, duration_ms: 0 }],
                                    total_duration_ms: 0,
                                    flip_h: t.flip_h,
                                    flip_v: t.flip_v,
                                    flip_d: t.flip_d,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some(cell) = maybe_cell {
                    cells[y as usize * map_w as usize + x as usize] = Some(cell);
                }
            }
        }

        layers.push(TiledLayerRaw {
            name: layer.name.clone(),
            width: map_w,
            height: map_h,
            cells,
        });
    }

    TiledVisualRaw {
        map_width: map_w,
        map_height: map_h,
        tile_width: tile_w,
        tile_height: tile_h,
        layers,
    }
}

fn property_string(props: &std::collections::HashMap<String, PropertyValue>, key: &str) -> Option<String> {
    match props.get(key) {
        Some(PropertyValue::StringValue(v)) => Some(v.clone()),
        _ => None,
    }
}

/// Top-left grid cell and size in tiles from a TMX object (rectangle, ellipse, or point → 1×1).
fn object_grid_rect(obj: &tiled::Object, tw: f32, th: f32, ox: f32, oy: f32) -> (i32, i32, i32, i32) {
    let wx = obj.x + ox;
    let wy = obj.y + oy;
    let gx = (wx / tw).floor() as i32;
    let gy = (wy / th).floor() as i32;
    let (ew, eh) = match &obj.shape {
        ObjectShape::Rect { width, height } | ObjectShape::Ellipse { width, height } => {
            let ew = if *width > f32::EPSILON {
                ((*width / tw).round() as i32).max(1)
            } else {
                1
            };
            let eh = if *height > f32::EPSILON {
                ((*height / th).round() as i32).max(1)
            } else {
                1
            };
            (ew, eh)
        }
        _ => (1, 1),
    };
    (gx, gy, ew, eh)
}

#[cfg(target_arch = "wasm32")]
fn path_lookup_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Preload TMX and referenced tileset files into memory so the `tiled` crate can parse them on
/// WASM (no `std::fs`). Call once from `main` before starting a run. Safe to call on native; it
/// is a no-op outside WASM.
///
/// All paths in [`TMX_WASM_PRELOADS`] must load successfully; otherwise the bundle is not
/// installed and level 1 will fall back to the placeholder layout (easy to mistake for “wrong map”).
pub async fn preload_level_tmx_for_wasm() {
    #[cfg(target_arch = "wasm32")]
    {
        use macroquad::file::load_file;

        let mut map = HashMap::new();
        for p in TMX_WASM_PRELOADS {
            match load_file(p).await {
                Ok(bytes) => {
                    map.insert(p.to_string(), bytes);
                }
                Err(e) => {
                    macroquad::prelude::error!(
                        "WASM TMX preload failed for {p}: {e} — ensure `bash build.sh` copied assets into dist/ and URLs use paths like assets/levels/..."
                    );
                    return;
                }
            }
        }
        if map.len() == TMX_WASM_PRELOADS.len() {
            let _ = WASM_LEVEL_TMX_RESOURCES.set(Arc::new(map));
            macroquad::prelude::info!("WASM TMX preload OK ({} files)", TMX_WASM_PRELOADS.len());
        }
    }
}

fn build_level_from_map(map: Map, path: &str) -> Result<Level, String> {
    let mut level = Level::new(map.width as usize, map.height as usize, LEVEL1_PALETTE);

    // Extract visual raw before consuming the map for entity/logic iteration.
    level.tiled_visual_raw = Some(build_tiled_visual_raw(&map));

    for layer in map.layers() {
        match layer.layer_type() {
            LayerType::Tiles(tile_layer) if layer.name == "floor" => {
                for y in 0..map.height as i32 {
                    for x in 0..map.width as i32 {
                        if tile_layer.get_tile(x, y).is_some() {
                            level.set_tile(x, y, Tile::Floor);
                        }
                    }
                }
            }
            LayerType::Tiles(tile_layer) if layer.name == "walls" => {
                for y in 0..map.height as i32 {
                    for x in 0..map.width as i32 {
                        if tile_layer.get_tile(x, y).is_some() {
                            level.set_tile(x, y, Tile::SolidWall);
                        }
                    }
                }
            }
            LayerType::Tiles(tile_layer) if layer.name == "abyss" => {
                for y in 0..map.height as i32 {
                    for x in 0..map.width as i32 {
                        if tile_layer.get_tile(x, y).is_some() {
                            level.set_tile(x, y, Tile::SolidWall);
                        }
                    }
                }
            }
            LayerType::Objects(object_layer) if layer.name == "entities" => {
                let tw = map.tile_width as f32;
                let th = map.tile_height as f32;
                let ox = layer.offset_x;
                let oy = layer.offset_y;
                for obj in object_layer.objects() {
                    let wx = obj.x + ox;
                    let wy = obj.y + oy;
                    let gx = (wx / tw).floor() as i32;
                    let gy = (wy / th).floor() as i32;
                    let obj_type = if !obj.user_type.is_empty() {
                        obj.user_type.as_str()
                    } else {
                        obj.name.as_str()
                    };

                    match obj_type {
                        "spawn" => {
                            level.spawn_x = gx;
                            level.spawn_y = gy;
                        }
                        "door" => {
                            let (dgx, dgy, ew, eh) = object_grid_rect(&obj, tw, th, ox, oy);
                            level.door_x = dgx;
                            level.door_y = dgy;
                            level.door_w = ew;
                            level.door_h = eh;
                            for dy in 0..eh {
                                for dx in 0..ew {
                                    let tx = dgx + dx;
                                    let ty = dgy + dy;
                                    if level.get_tile(tx, ty) == Tile::SolidWall {
                                        level.set_tile(tx, ty, Tile::Door);
                                    }
                                }
                            }
                        }
                        "exit" => {
                            let (ex, ey, ew, eh) = object_grid_rect(&obj, tw, th, ox, oy);
                            level.exit_x = ex;
                            level.exit_y = ey;
                            level.exit_w = ew;
                            level.exit_h = eh;
                        }
                        "chest" => {
                            level.chests.push(Chest::new(gx, gy));
                        }
                        "enemy" => {
                            let kind = property_string(&obj.properties, "kind").unwrap_or_else(|| "zombie".to_string());
                            let enemy_kind = match kind.as_str() {
                                "big_zombie" => EnemyKind::BigZombie,
                                "big_demon" => EnemyKind::BigDemon,
                                _ => EnemyKind::Zombie,
                            };
                            level.enemies.push(Enemy::new_with_kind(gx, gy, enemy_kind));
                        }
                        "item" => {
                            let kind = property_string(&obj.properties, "kind").unwrap_or_else(|| "coin".to_string());
                            let item_type = match kind.as_str() {
                                "blue_coin" => ItemType::BlueCoin,
                                "coin_bag" => ItemType::CoinBag,
                                "key" => ItemType::Key,
                                "potion" => ItemType::Potion,
                                "small_potion" => ItemType::SmallPotion,
                                "big_potion" => ItemType::BigPotion,
                                "shield_potion" => ItemType::ShieldPotion,
                                "big_shield_potion" => ItemType::BigShieldPotion,
                                _ => ItemType::Coin,
                            };
                            level.items.push(Item::new(gx, gy, item_type));
                        }
                        "key" => {
                            level.items.push(Item::new(gx, gy, ItemType::Key));
                        }
                        "gate" => {
                            level.gates.push(GateEntity::new(gx, gy));
                        }
                        "button" => {
                            level.buttons.push(FloorButton::new(gx, gy));
                        }
                        "torch" => {
                            let dir = property_string(&obj.properties, "dir").unwrap_or_else(|| "top".to_string());
                            let torch_dir = match dir.as_str() {
                                "left" => TorchDir::Left,
                                "right" => TorchDir::Right,
                                _ => TorchDir::Top,
                            };
                            level.torches.push(Torch::with_direction(gx, gy, torch_dir));
                        }
                        "vase" => {
                            level.vases.push(Vase::new(gx, gy, false));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    merge_vases_from_vase_shine_anim_tiles(&map, &mut level);

    if level.width != LEVEL1_W || level.height != LEVEL1_H {
        return Err(format!(
            "level dimensions mismatch: expected {}x{}, got {}x{} from {}",
            LEVEL1_W,
            LEVEL1_H,
            level.width,
            level.height,
            Path::new(path).display()
        ));
    }

    Ok(level)
}

/// `decoration` / `decor` tiles using the `vase_shine_anim` tileset become solid, attack-breakable vases
/// (sprite stays on the tile layer; shards drawn from gameplay when smashed).
fn merge_vases_from_vase_shine_anim_tiles(map: &Map, level: &mut Level) {
    for layer in map.layers() {
        let LayerType::Tiles(tile_layer) = layer.layer_type() else {
            continue;
        };
        if !matches!(layer.name.as_str(), "decoration" | "decor") {
            continue;
        }
        for y in 0..map.height as i32 {
            for x in 0..map.width as i32 {
                let Some(t) = tile_layer.get_tile(x, y) else {
                    continue;
                };
                if t.get_tileset().name != "vase_shine_anim" {
                    continue;
                }
                if let Some(v) = level.vases.iter_mut().find(|v| v.grid_x == x && v.grid_y == y) {
                    v.tiled_sprite = true;
                } else {
                    level.vases.push(Vase::new(x, y, true));
                }
            }
        }
    }
}

pub fn load_level_from_tmx(path: &str) -> Result<Level, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(bundle) = WASM_LEVEL_TMX_RESOURCES.get() else {
            return Err(
                "WASM TMX resources not preloaded (call preload_level_tmx_for_wasm from main)".to_string(),
            );
        };
        let bundle = Arc::clone(bundle);
        let mut loader = Loader::with_reader(move |p: &Path| {
            let key = path_lookup_key(p);
            bundle
                .get(&key)
                .or_else(|| bundle.get(key.trim_start_matches("./")))
                .cloned()
                .map(Cursor::new)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "WASM TMX bundle missing key: {key} ({} entries loaded)",
                            bundle.len()
                        ),
                    )
                })
        });
        let map = loader
            .load_tmx_map(path)
            .map_err(|e| format!("failed to load TMX {path}: {e}"))?;
        return build_level_from_map(map, path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut loader = Loader::new();
        let map = loader
            .load_tmx_map(path)
            .map_err(|e| format!("failed to load TMX {path}: {e}"))?;
        build_level_from_map(map, path)
    }
}

#[cfg(test)]
mod tests {
    use super::load_level_from_tmx;

    #[test]
    fn parses_level1_tmx_and_entities() {
        let level = load_level_from_tmx("assets/levels/level1.tmx").expect("level1.tmx should parse");
        assert_eq!(level.width, 16);
        assert_eq!(level.height, 16);
        assert!(
            level.spawn_x >= 0 && level.spawn_y >= 0,
            "spawn must be set"
        );
        assert!(level.exit_x >= 0 && level.exit_y >= 0, "exit must be set");
        assert!(level.exit_w >= 1 && level.exit_h >= 1);
        assert!(level.door_x >= 0 && level.door_y >= 0, "door must be set");
        assert!(level.door_w >= 1 && level.door_h >= 1);
    }
}
