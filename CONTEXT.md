# Dungeon Diver - Project Context

**Last Updated:** 2025-03-20
**Current Phase:** Phase 3 Complete (Level 1 Content)
**Build Status:** WASM builds successfully, game playable in browser

---

## Project Overview

Dungeon Diver is a graphically rich action-adventure dungeon crawler built with Rust and WebAssembly, designed specifically for TV remote control input on smart TVs, Android TV, and set-top boxes.

### Genre
Action-Adventure with Roguelite Progression

### Target Platform
- Chrome 80+ (February 2020 baseline)
- WebAssembly (MVP support)
- 1280×720 resolution
- TV remote input (D-pad + OK/Back buttons)

---

## Technical Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Graphics Engine | macroquad 0.4 |
| Platform | WebAssembly |
| Build Tool | wasm-pack |
| Target Renderer | WebGL (via macroquad/miniquad) |

### Key Dependencies (Cargo.toml)
```toml
[dependencies]
macroquad = "0.4"

[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-O4']  # Maximum optimization
```

---

## Current Game State (Phase 3 Complete)

### Implemented Features
- ✅ Grid-based movement system (32×32 pixel tiles)
- ✅ TV remote input handler with hold detection
- ✅ Camera system with smooth follow and screen centering
- ✅ Player entity with Blue Knight sprites (idle + run animations)
- ✅ Level 1: Tutorial Dungeon (40×20 tiles, fills screen)
- ✅ Terrain atlas system (Gathering Set 1.0/1.1/1.2)
- ✅ Collectibles: Coins (gold/blue), Potions, Vases
- ✅ Torches with flickering glow effects
- ✅ Exit door with animated glow
- ✅ Basic game states (Title, Playing, PauseMenu, Inventory, GameOver, LevelComplete)
- ✅ HUD: Hearts display, coin counter
- ✅ WASM build pipeline

### Design Decisions Made

| Decision | Value | Rationale |
|----------|-------|-----------|
| Tile Size | 32×32 pixels | TV visibility at 10ft distance |
| Level Size | 40×20 tiles (1280×640 px) | Fills screen width, minimal scrolling |
| Player Size | 64×64 pixels (2× tiles) | More visible on TV |
| Player Speed | 240 pixels/sec | Smooth movement with larger tiles |
| Knight Sprite Frame | 16×16 (idle), 16×17 (run) | From asset filenames |
| Terrain Cell Size | 16×16 → scaled 2× to 32×32 | Matches Gathering assets |

---

## File Structure

```
dungeon-diver/
├── Cargo.toml              # Project config, macroquad 0.4
├── build.sh                # WASM build script
├── src/
│   ├── main.rs             # Entry point, asset loading, game loop
│   ├── game.rs             # GameState machine, update/draw loops
│   ├── player.rs           # Player entity, movement, sprite rendering
│   ├── level.rs            # Tile map, level loading, rendering
│   ├── input.rs            # TV remote handler (hold detection)
│   ├── camera.rs           # Viewport, follow, clamping
│   ├── items.rs            # Coins, potions, vases, torches
│   ├── terrain.rs          # TerrainAtlas, autotiling
│   └── constants.rs        # All config values
├── assets/
│   ├── dg_knight/          # Blue Knight sprites
│   ├── dg_gathering_free_ver/  # Terrain tiles, items
│   └── [other asset folders]
├── www/
│   └── index.html          # HTML wrapper, canvas, loading screen
└── pkg/                    # Generated WASM output
```

---

## Key Constants (src/constants.rs)

```rust
// Screen
pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;

// Tiles
pub const TILE_SIZE: f32 = 32.0;
pub const SCREEN_TILES_W: i32 = 40;
pub const SCREEN_TILES_H: i32 = 22;

// Level 1 Dimensions
pub const LEVEL1_W: usize = 40;
pub const LEVEL1_H: usize = 20;

// Player
pub const PLAYER_DISPLAY_SIZE: f32 = 64.0;  // 2× tile size
pub const PLAYER_SPEED: f32 = 240.0;

// Sprite Frames (from filenames)
pub const KNIGHT_IDLE_FRAME_W: f32 = 16.0;
pub const KNIGHT_IDLE_FRAME_H: f32 = 16.0;
pub const KNIGHT_RUN_FRAME_W: f32 = 16.0;
pub const KNIGHT_RUN_FRAME_H: f32 = 17.0;

// Lives System
pub const MAX_LIVES: i32 = 3;
```

---

## Level 1: Tutorial Dungeon

### Layout (40×20 tiles)
```
########################################
#@...............T.................#
#.............#######..............#
#...C.............................E..#
#.............#.............######
#.............#.....T............#
#.T..........###.......V........#
#.............#..................#
#.............#.....##..........#
#.....#######.....#..#.....P...#
#.....#.....#.....#..#..........#
#.....#..V..#..C..#..#.....##...#
#.....#.....#.....#..#..........#
#.....#######.....#..#..........#
#.............#..................#
#.............#.......T........#
#.......C.....P...............B.#
#.............##################
#.............#..................#
#.............#..................#
#.............#..................#
########################################
```

### Legend
- `#` = Wall (terrain atlas with 4-bit autotiling)
- `.` = Floor (terrain atlas variants)
- `@` = Player spawn
- `E` = Exit (glowing door)
- `C` = Gold coin
- `B` = Blue coin
- `P` = Potion (heals)
- `V` = Vase (breakable, may contain coin)
- `T` = Torch (flickering glow)

### Color Palette
```rust
pub const LEVEL1_PALETTE: Palette = Palette {
    wall_top:    Color { r: 0.36, g: 0.45, b: 0.55, a: 1.0 },  // Gray-blue
    wall_side:   Color { r: 0.25, g: 0.33, b: 0.42, a: 1.0 },
    floor:       Color { r: 0.20, g: 0.20, b: 0.30, a: 1.0 },
    floor_alt:   Color { r: 0.18, g: 0.18, b: 0.28, a: 1.0 },
    accent:      Color { r: 0.42, g: 0.70, b: 0.95, a: 1.0 },  // Blue glow
    bg_top:      Color { r: 0.12, g: 0.12, b: 0.22, a: 1.0 },
    bg_bot:      Color { r: 0.06, g: 0.06, b: 0.12, a: 1.0 },
    text:        Color { r: 0.90, g: 0.90, b: 0.95, a: 1.0 },
};
```

---

## Asset Documentation

### Player Sprites (`assets/dg_knight/`)

| Asset | File | Cell Size | Display Size | Use |
|-------|------|-----------|--------------|-----|
| Blue Knight Idle | `Blue Knight idle Sprite-sheet 16x16.png` | 16×16 px | 64×64 px | Standing state |
| Blue Knight Run | `Blue Knight run Sprite-sheet 16x17.png` | 16×17 px | 64×64 px | Moving animation |

**Frame Counting:** `floor(texture_width / cell_width)`
- Not guessed from filenames
- Derived programmatically in `Player::load_sprites()`

### Terrain Tiles (`assets/dg_gathering_free_ver/`)

Required files (names must match exactly):

| File | Size | Use |
|------|------|-----|
| `Set 1.0.png` | 192×64 | Floor variants (12 tiles) |
| `Set 1.1.png` | 288×112 | Wall autotile (18×7 tiles) |
| `Set 1.2.png` | 256×96 | Floor variants (16×6 tiles) |

**Scaling:** 16×16 cells → 2× scale → 32×32 tiles
**Wall Autotiling:** 4-bit neighbor (top-right-bottom-left)
**Floor Variety:** Random selection from all three sets

### Collectibles & Objects

| Asset | File | Size | Scale |
|-------|------|------|-------|
| Torch | `Torch Yellow.png` | 16×16 | 2× |
| Coin Sheet | `Coin Sheet.png` | Multi-frame | 2× |
| Blue Coin Sheet | `BlueCoin Sheet.png` | Multi-frame | 2× |
| Potions | `Potions.png` | Multi-frame | 2× |
| Vase | `Vase Shine Anim.png` | Multi-frame | 2× |

---

## Implementation Roadmap

### Phase 1: Foundation ✅ COMPLETE
- ✅ Initialize Rust project with macroquad
- ✅ Create constants file with all config
- ✅ Implement 32×32 tile grid system
- ✅ Create GameState enum and basic loop
- ✅ TV remote input handler (hold detection)
- ✅ Camera system (follow player smoothly)

### Phase 2: Player & Movement ✅ COMPLETE
- ✅ Player entity with grid position
- ✅ 4-directional D-pad movement
- ✅ Load and render Blue Knight sprite
- ✅ Idle animation state
- ✅ Run animation (16 frames)
- ✅ Collision with walls

### Phase 3: Level 1 Content ✅ COMPLETE
- ✅ Define Level 1 tile map (40×20)
- ✅ Load dungeon tiles from sprite sheet
- ✅ Render tile map with camera
- ✅ Place exit door with glow
- ✅ Place torches with glow effect
- ✅ Terrain atlas integration (user added)

### Phase 4: Combat System ⏳ PENDING
- ⏳ Attack action (OK/Enter button)
- ⏳ Hit detection in facing direction
- ⏳ Enemy entity (Zombie)
- ⏳ Zombie AI (chase player on sight)
- ⏳ Enemy hurt/death animations
- ⏳ Knockback effect

### Phase 5: Collectibles & Inventory ⏳ PENDING
- ✅ Coin pickup entity (done)
- ✅ Potion pickup (done)
- ✅ Breakable vases (done)
- ⏳ 3-slot inventory system
- ⏳ Auto-equip better items

### Phase 6: UI & Menus ⏳ PARTIAL
- ✅ Pause menu (single column)
- ⏳ Inventory screen (placeholder only)
- ⏳ Options screen
- ✅ Hearts display
- ✅ Coin counter
- ⏳ Game Over screen polish

### Phase 7: Lives & Progression ⏳ PENDING
- ⏳ 3 lives system
- ⏳ Respawn at level start on death
- ⏳ Game Over → restart Level 1
- ⏳ Level complete transition
- ⏳ Persistent score/coins

### Phase 8: Polish & Effects ⏳ PENDING
- ⏳ Particle system (hit, pickup)
- ⏳ Screen shake on combat
- ⏳ Torch flicker animation
- ⏳ Entrance animations
- ⏳ Sound effects (optional)

### Phase 9: Remaining Levels (2-10) ⏳ PENDING
| Level | Theme | Mechanic |
|-------|-------|----------|
| 2 | Crystal Caves | Slippery floors |
| 3 | Fungus Forest | Spreading hazards |
| 4 | Ember Ruins | Fire jets |
| 5 | Frozen Tundra | Ice physics |
| 6 | Shadow Keep | Limited visibility |
| 7 | Desert Temple | Sand pits (slow) |
| 8 | Flooded Depths | Water sections |
| 9 | Sky Fortress | Wind gusts |
| 10 | Void Throne | Boss fight |

### Phase 10: Build & Deploy ⏳ PENDING
- ⏳ Optimize WASM size (< 2MB)
- ⏳ Test on Chrome 80+
- ⏳ TV remote input testing
- ⏳ Deploy to production

---

## TV Remote Controls

| Button | Action |
|--------|--------|
| D-Pad Up/Down/Left/Right | Move player (hold to repeat) |
| OK / Enter | Attack enemy / Select option |
| Back / Return / ESC | Open Pause Menu |

### Pause Menu
```
┌─────────────────┐
│ Return to game  │
│ Inventory       │
│ Options         │
│ Exit game       │
└─────────────────┘
```
Navigate with D-Pad Up/Down, select with OK/Enter.

---

## Build & Run

### Development (Native)
```bash
cargo run
```

### Build WASM
```bash
wasm-pack build --release --target web
```

### Serve Locally
```bash
npx serve www
# Open http://localhost:3000
```

### Build Script
```bash
./build.sh
```

---

## Known Issues & Notes

1. **WASM Size:** Current debug build ~16MB. Release optimization will reduce significantly.
2. **Terrain Fallback:** If terrain atlas fails to load, game falls back to colored rectangles.
3. **Sprite Frame Counting:** Uses `floor(texture_width / cell_width)` - not hardcoded.
4. **Camera Centering:** Levels smaller than screen are centered automatically.
5. **Missing Features:** Enemies, combat, lives system, inventory UI not yet implemented.

---

## Next Steps (Phase 4: Combat System)

1. Create `src/enemy.rs` with Zombie entity
2. Implement attack action in `src/player.rs`
3. Add hit detection in `src/game.rs`
4. Implement Zombie AI (chase player)
5. Add hurt/death animations for enemies
6. Add knockback effect on hit

---

## User Feedback History

1. **Tile Size:** User requested larger tiles for TV visibility → Changed from 16×16 to 32×32
2. **Player Size:** User couldn't see player well → Increased to 64×64 pixels (2× tiles)
3. **Level Position:** Level appeared on right side → Fixed camera clamping to center
4. **Terrain:** User integrated Gathering terrain atlas system
5. **Asset Documentation:** User documented sprite frame sizes in README.md

---

*This document is auto-generated. Update as the project progresses.*
