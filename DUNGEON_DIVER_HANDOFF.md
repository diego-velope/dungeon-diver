# Dungeon Diver — Full Session Handoff Document
> Single source of truth for continuing in Claude Code or a new session.
> Last updated: March 2026 — covers everything built and learned.

---

## 1. What Is Dungeon Diver?

Action-adventure dungeon crawler built in **Rust + Macroquad**, compiled to **WebAssembly**, deployed on **Vercel**. Designed for TV remote input (D-pad + OK/Back). Target: Chrome 80+, 1280×720, 32px tile grid.

**Genre:** Top-down grid-based dungeon crawler with roguelite progression  
**Status:** Phase 3 complete, assets being migrated to 0x72 tileset, Phase 4 (combat) pending

---

## 2. Tech Stack

| Item | Detail |
|---|---|
| Language | Rust 2021 |
| Rendering | macroquad 0.4 |
| WASM target | `wasm32-unknown-unknown` |
| JS glue | `mq_js_bundle.js` from miniquad CDN |
| Deployment | Vercel — static, `dist/` committed to repo |
| Build | `bash build.sh` → `dist/` |
| Local preview | `npx serve dist` → `http://localhost:3000` |
| Repo | `dungeon-diver` (private, fill in URL) |

---

## 3. Project File Structure

```
dungeon-diver/
├── src/
│   ├── main.rs         — window config (1280×720), asset loading, game loop
│   ├── game.rs         — GameState machine, update/draw dispatcher
│   ├── player.rs       — Player entity, 4-dir movement, sprite animation
│   ├── level.rs        — Tile map, level loading, terrain draw dispatch
│   ├── input.rs        — TV remote input handler, hold detection
│   ├── camera.rs       — Smooth follow camera, screen shake, level clamping
│   ├── items.rs        — Coins, potions, vases, torches
│   ├── terrain.rs      — TerrainAtlas, wall autotile, floor variants
│   └── constants.rs    — All config values (TILE_SIZE, PLAYER_SPEED, etc.)
├── assets/
│   ├── dg_knight/                  — Blue Knight sprites (CURRENT player)
│   │   ├── Blue Knight idle Sprite-sheet 16x16.png  (128×64, 8×4 frames)
│   │   └── Blue Knight run Sprite-sheet 16x17.png
│   ├── dg_gathering_free_ver/      — OLD tileset (being replaced)
│   │   ├── Set 1.0.png
│   │   ├── Set 1.1.png
│   │   └── Set 1.2.png
│   └── 0x72/                       — NEW tileset (migration in progress)
│       ├── floor_1.png … floor_8.png
│       ├── wall_mid.png, wall_top_mid.png, wall_top_left.png, wall_top_right.png
│       ├── wall_left.png, wall_right.png
│       ├── doors_leaf_closed.png, doors_leaf_open.png, doors_frame_top.png
│       ├── ui_heart_full.png, ui_heart_empty.png, ui_heart_half.png
│       ├── coin_anim_f0-f3.png
│       ├── flask_red.png, flask_blue.png
│       ├── chest_full_open_anim_f0-f2.png
│       ├── goblin_idle/run_anim_f0-f3.png   — Phase 4 ready
│       ├── skelet_idle/run_anim_f0-f3.png   — Phase 4 ready
│       └── tiny_zombie_idle/run_anim_f0-f3.png — Phase 4 ready
├── www/
│   └── index.html      — HTML shell, canvas, miniquad mouse fix
├── dist/               — Pre-built WASM committed to repo for Vercel
├── build.sh
├── Cargo.toml
└── vercel.json
```

---

## 4. Key Constants (`src/constants.rs`)

```rust
pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const TILE_SIZE: f32 = 32.0;       // 16px native × 2× scale
pub const SCREEN_TILES_W: i32 = 40;
pub const SCREEN_TILES_H: i32 = 22;
pub const LEVEL1_W: usize = 40;
pub const LEVEL1_H: usize = 20;
pub const PLAYER_DISPLAY_SIZE: f32 = 64.0;  // 2× tile size for TV visibility
pub const PLAYER_SPEED: f32 = 240.0;
pub const TILE_SIZE_I: i32 = 32;
pub const MAX_LIVES: i32 = 3;
pub const MAX_HP: i32 = 3;
```

---

## 5. Current Game State — Phase Checklist

### Phase 1 — Foundation ✅ COMPLETE
- Grid-based movement (32×32 tiles)
- TV remote input with hold detection
- Camera with smooth follow + screen shake
- Game state machine (Title → Playing → Pause → GameOver → LevelComplete)

### Phase 2 — Player & Movement ✅ COMPLETE
- 4-directional D-pad movement
- Grid-snap movement with pixel interpolation
- Blue Knight sprites loaded and animated
- **4-directional sprite rows** — idle sheet is 8×4 (row 0=down, 1=left, 2=up, 3=left variant; right = row 1 + flip_x)
- Collision with walls
- Player starts facing Down (row 0 = front-facing visor)

### Phase 3 — Level 1 Content ✅ COMPLETE
- Level layout: 40×20 tiles, all rows verified exactly 40 chars
- Terrain atlas with autotiling (currently mid-migration to 0x72)
- Coins (gold + blue), potions, vases, torches placed
- Exit door with glow / door frame
- HUD: hearts, coin counter
- Pause menu

### Phase 4 — Combat ⏳ PENDING
- Attack action (Enter/OK)
- Enemy entity (start with goblin from 0x72)
- Enemy AI (chase player on sight)
- Hit detection
- Knockback

### Phase 5 — Collectibles & Inventory ⏳ PARTIAL
- Coins ✅, Potions ✅, Vases ✅
- 3-slot inventory UI ⏳
- Auto-equip ⏳

### Phase 6+ — Remaining phases
See CONTEXT.md in the repo for full roadmap (levels 2-10, progression, deploy)

---

## 6. Tileset Migration Status (IMPORTANT — in progress)

### What we discovered about the old tileset (dg_gathering_free_ver)
- Set 1.0/1.1/1.2 are 16×16 atlas sheets
- The original autotile table had **wrong coordinates** — pointed at architectural elements instead of wall tiles
- The level layout rows were **wrong lengths** (32–38 chars instead of 40), causing walls to stop mid-row
- Both bugs were fixed before deciding to switch tilesets

### New tileset: 0x72 DungeonTileset II v1.7
- **License:** Free / MIT
- **All tiles:** 16×16px native, drawn at 2× = 32px to match TILE_SIZE
- **Key advantage:** Individual named PNG files instead of atlas math
- **Enemies included:** goblin, skelet, tiny_zombie, zombie, oglin, imp, orc, big_demon, chort, necromancer, and more — all with idle + run 4-frame animations

### New terrain.rs (0x72 version) — what it does

```
draw_wall() autotile logic:
  south open → wall_top_left / wall_top_mid / wall_top_right  (top cap)
  west/east open, south closed → wall_left / wall_right        (side face)
  surrounded → wall_mid                                        (interior)

draw_floor() → stable hash → picks floor_1 through floor_8
draw_door()  → floor_1 base + doors_leaf_closed + doors_frame_top above
```

### Asset path in terrain.rs
```rust
let p = "assets/0x72";  // all 0x72 files go here
```

### Files needed in assets/0x72/ (47 total)
```
floor_1.png through floor_8.png
wall_mid.png, wall_top_mid.png, wall_top_left.png, wall_top_right.png
wall_left.png, wall_right.png
doors_leaf_closed.png, doors_leaf_open.png, doors_frame_top.png
ui_heart_full.png, ui_heart_empty.png, ui_heart_half.png
coin_anim_f0.png through coin_anim_f3.png
flask_red.png, flask_blue.png, flask_big_red.png, flask_big_blue.png
chest_full_open_anim_f0/f1/f2.png, chest_empty_open_anim_f0/f1/f2.png
skull.png, crate.png
goblin_idle_anim_f0-f3.png, goblin_run_anim_f0-f3.png
skelet_idle_anim_f0-f3.png, skelet_run_anim_f0-f3.png
tiny_zombie_idle_anim_f0-f3.png, tiny_zombie_run_anim_f0-f3.png
```
A copy script `copy_0x72_assets.sh` was generated — edit the `SRC=` path and run it.

---

## 7. Player Sprite Sheet Details

### Blue Knight idle (128×64, 8 cols × 4 rows, 16×16 per frame)
| Row | Direction | Notes |
|---|---|---|
| 0 | Down (facing camera) | Front-facing, visor visible |
| 1 | Left | Side profile |
| 2 | Up (facing away) | Back, no visor |
| 3 | Left variant | Second cycle, reserved |

**Right direction** = row 1 with `flip_x = true`  
**Initial facing** = `Direction::Down` (spawns facing camera)

### 0x72 knight_m sprites (individual files, 16×28px)
Available as `knight_m_idle_anim_f0-f3.png` and `knight_m_run_anim_f0-f3.png`
These are front-facing only (like most 0x72 characters). Could replace Blue Knight
but would lose 4-directional rotation. Decision deferred.

---

## 8. Level Layout — Level 1 (16x16, verified)

```
"##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",  // row 0
            "#LT@...........RT#",  // row 1
            "#.........BP...E#",  // row 2  
            "#......C......RT#",  // row 3
            "#.....##-#-#-#.G..#",  // row 4
            "#.....#-.H.#....#",  // row 5
            "#.....T...#-#-#-#-#-#",  // row 6
            "#.B....C....SP..#",  // row 7
            "#.G........B...#",  // row 8
            "#...C.......C..#",  // row 9
            "#.........C...RT#",  // row 10
            "#LT.............#",  // row 11
            "#.......C......#",  // row 12
            "#..C...........#",  // row 13
            "#..........G...#",  // row 14
            "################",  // row 15
```

**Critical rule:** Every row must be EXACTLY 16 characters. The parser silently skips
out-of-bounds chars — wrong lengths cause walls to vanish mid-row.

---

## 9. Build & Deploy Pipeline

### Local dev
```bash
cargo run                     # native window, fast iteration
```

### WASM build + test
```bash
bash build.sh                 # compiles → assembles dist/
npx serve dist                # serve at localhost:3000
```

### Deploy to Vercel
```bash
bash build.sh
git add dist/
git commit -m "describe changes"
git push                      # Vercel auto-deploys in ~10s
```

### vercel.json (critical — WASM needs correct Content-Type)
```json
{
  "version": 2,
  "framework": null,
  "outputDirectory": "dist",
  "buildCommand": "",
  "routes": [
    { "src": "/(.*)\\.wasm", "headers": { "Content-Type": "application/wasm" }, "dest": "/$1.wasm" },
    { "src": "/assets/(.*)", "dest": "/assets/$1" },
    { "src": "/(.*)", "dest": "/$1" }
  ]
}
```

---

## 10. Known Bugs & Fixes (NEVER FORGET THESE)

### miniquad 0.4.x RefCell panic on mouse events
**Symptom:** Wall of `panic_already_borrowed` errors in console on mouse move  
**Cause:** miniquad 0.4 has a bug with simultaneous mouse + focus events  
**Fix:** Always include this in index.html after `load("dungeon-diver.wasm")`:
```js
setTimeout(function () {
  var canvas = document.getElementById("glcanvas");
  if (canvas) {
    canvas.onmousemove = null;
    canvas.onmousedown = null;
    canvas.onmouseup   = null;
    canvas.onmouseenter = null;
    canvas.onmouseleave = null;
    canvas.onfocus     = null;
    canvas.oncontextmenu = function(e){ e.preventDefault(); };
  }
}, 800);  // 800ms gives miniquad time to attach its handlers first
```

### Asset filenames are case-sensitive on Vercel (Linux)
`Astronaut.png` ≠ `astronaut.png`. Always use lowercase filenames in Rust code
and on disk. The crash on a 404 causes a cascade of RefCell panics that looks
unrelated.

### WASM requires HTTP — never open dist/index.html directly
Use `npx serve dist` or any local HTTP server. `file://` protocol blocks WASM.

### Level layout rows must be exactly N chars wide
The parser has `if x >= level.width { continue; }` — silently skips. Wrong-length
rows cause walls to disappear mid-row. Always verify with:
```python
for i, row in enumerate(layout):
    assert len(row) == 40, f"Row {i}: {len(row)} chars"
```

### Harmless console warnings (ignore these)
```
No glRenderbufferStorageMultisample function in gl.js
Plugin macroquad_audio is present in JS bundle but not used
```
These are normal — miniquad logs stubs for optional WebGL2 functions.

---

## 11. Game Dev Lessons Learned This Session

### Tileset / Pixel Art

**Tile size must be a single universal constant.**
If TILE_SIZE=32 then: tiles draw at 32px, player draws at 32px (or 2×32 for visibility),
enemies draw at 32px, everything divides evenly. Mixing sizes (player at 64, tiles at 32)
means the player always looks misaligned.

**Always pixel-inspect before writing tile coordinates.**
The most common mistake: guessing `(col, row)` in an atlas without verifying. Extract
every cell at 4-8× zoom with coordinate labels. 30 minutes of inspection saves hours of
wrong-tile debugging.

**Named files > atlas coordinates.**
The 0x72 tileset uses individual named PNGs. `load_texture("wall_mid.png")` is infinitely
clearer than `draw_cell(&atlas, 5, 3, sx, sy)`. When possible, prefer named assets.

**Autotiling simplified to 3 cases covers 95% of dungeons:**
1. South is floor → top-cap tile (player sees top of wall)
2. East/West is floor → side-face tile (player sees side of wall)
3. Surrounded → interior tile (never visible to player, use anything solid)

**BSP room generation vs random placement:**
Random room placement = overlapping, gaps, no logic. BSP = divide space in half
recursively, each leaf cell becomes a room. Result always looks intentional.
Reference image used BSP; that's why it looked clean even with simple tiles.

### Rust + Macroquad + WASM

**The full working pattern for loading 0x72 individual sprites:**
```rust
let t = load_texture("assets/0x72/wall_mid.png").await.ok()?;
t.set_filter(FilterMode::Nearest);  // ALWAYS — prevents blurry pixel art
```

**draw_texture_ex for scaling:**
```rust
draw_texture_ex(&tex, sx, sy, WHITE, DrawTextureParams {
    dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),  // scale 16→32
    source: Some(Rect::new(frame_x, frame_y, frame_w, frame_h)), // for atlases
    flip_x: facing == Direction::Right,           // mirror for direction
    rotation: angle,                              // radians for backflip etc
    ..Default::default()
});
```

**Camera follows player — pixel-snap to prevent jitter:**
```rust
let render_cam_x = camera.x.floor();
let render_cam_y = camera.y.floor();
let screen_x = (world_x - render_cam_x).floor();
```

**Frame time — always cap it:**
```rust
let dt = get_frame_time().min(0.05);  // cap at 50ms (20 FPS minimum)
// prevents physics explosion if tab loses focus
```

**Stable per-tile hash for floor variety (no randomness needed):**
```rust
let h = ((gx.wrapping_mul(2971) ^ gy.wrapping_mul(1619)) as u32)
    .wrapping_add(gx.wrapping_mul(gy) as u32);
let variant = (h as usize) % NUM_VARIANTS;
```

**4-directional sprite rows:**
```rust
let dir_row: u32 = match facing {
    Direction::Down  => 0,
    Direction::Left  => 1,
    Direction::Up    => 2,
    Direction::Right => 1,  // same row, flip_x = true
};
let src_y = dir_row as f32 * frame_h;
```

**Grid movement with pixel interpolation:**
```rust
// Grid position = where you ARE (tile coords)
// Pixel position = where you APPEAR (interpolated between tiles)
// On input: update grid_x/grid_y immediately, set is_moving=true
// Each frame: pixel pos lerps toward grid pos via move_progress
move_progress += dt * (PLAYER_SPEED / TILE_SIZE);
if move_progress >= 1.0 {
    px = grid_x as f32 * TILE_SIZE + TILE_SIZE/2.0;  // snap
    is_moving = false;
}
```

### Workflow

**Claude ↔ Cursor handoff works best when you:**
1. Keep a `CONTEXT.md` / `HANDOFF.md` at repo root — paste it at session start
2. Use `.cursorrules` for persistent context Cursor reads automatically
3. Give explicit scope: "Today: Phase 4 combat only. Don't add anything else."
4. Phase-by-phase is faster than all-at-once — clean checkpoints, easier debugging

**Debugging WASM crashes:**
- First check: is the asset filename case-correct?
- Second check: is the WASM served over HTTP (not file://)?
- Third check: is `Content-Type: application/wasm` set in vercel.json?
- Fourth: add the mouse handler null-out fix if you see RefCell errors

**When frustrated with terrain generation / room layout:**
Stop prompting. Go back to first principles: measure pixel sizes, draw the grid
on paper, verify row lengths in code. 95% of "generation looks wrong" bugs are
actually wrong data (wrong row lengths, wrong tile coordinates) not wrong logic.

---

## 12. Next Steps — Where to Pick Up

### Immediate (finish migration)
1. Run `copy_0x72_assets.sh` to populate `assets/0x72/`
2. Replace `src/terrain.rs` with the 0x72 version
3. Replace `src/level.rs` with the updated door draw version
4. `cargo run` — verify dungeon looks correct with new tiles
5. Update `items.rs` to load coin/flask/heart from `assets/0x72/` instead of drawing shapes

### Phase 4 — Combat (start here after migration)
1. Create `src/enemy.rs`:
   - `Enemy` struct: `grid_x, grid_y, hp, facing, anim_frame, anim_timer`
   - Load goblin sprites: `goblin_idle_anim_f0-f3.png` from `assets/0x72/`
   - Simple AI: if player within 5 tiles → move toward player each N frames
2. Add attack to `player.rs`:
   - On Enter press: set `is_attacking=true`, `attack_cooldown=0.3s`
   - `get_attack_position()` returns tile in front of player
3. Add to `game.rs`:
   - `enemies: Vec<Enemy>`
   - Each frame: check if player attack position overlaps enemy grid pos
   - Each frame: check if enemy grid pos overlaps player grid pos → damage

### Suggested first enemy placement in level layout
Add `Z` chars to the layout (already has a `'Z'` match arm in `level.rs`):
```
Row 6:  place a goblin near the inner room
Row 12: place skeleton in the right room
```

---

## 13. Quick Reference — Cargo.toml

```toml
[package]
name = "dungeon-diver"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = "0.4"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## 14. Useful Links

- macroquad docs: https://docs.rs/macroquad/latest/macroquad/
- macroquad examples: https://github.com/not-fl3/macroquad/tree/master/examples
- mq_js_bundle.js: https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
- 0x72 tileset: https://0x72.itch.io/dungeontileset-ii
- Cosmic Climber reference repo: https://github.com/diego-velope/cosmic-climber-wasm-rust

---

## 15. Session History Summary

Started this project after completing **Cosmic Climber** (vertical platformer) and
**Glider** (Alto's Adventure-style runner) — both Rust + Macroquad + WASM.

**Dungeon Diver evolution:**
1. Started with dg_gathering tileset — functional but atlas coordinates were wrong
2. Fixed level layout (all rows wrong lengths — silent parse bug)
3. Fixed terrain autotiling — 3-case system (top-cap, side-face, interior)
4. Fixed player sprite — 4-directional rows, correct initial facing
5. Migrated to 0x72 DungeonTileset II v1.7 — much richer content, named files,
   enemies pre-included for Phase 4

**The biggest insight of the session:**
"The gap" between reference art and current state is almost always wrong data
(tile coordinates, row lengths, scale mismatches), not wrong logic. Fix the data
first, then fix the logic.
