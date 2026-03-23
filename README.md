# Dungeon Diver

A graphically rich action-adventure dungeon crawler built with Rust and WebAssembly, designed specifically for TV remote control input on smart TVs, Android TV, and set-top boxes.

## About

Dungeon Diver is a 4-directional dungeon crawler with roguelite elements. Navigate through 10 handcrafted levels, each with unique themes, enemies, and mechanics. Collect weapons, potions, and keys as you dive deeper into the dungeon.

### Genre

Action-Adventure with Roguelite Progression

### Key Features

- **TV-Optimized Controls**: Designed for standard TV remotes (D-pad + OK/Back buttons)
- **10 Handcrafted Levels**: Each with unique visual themes and gameplay mechanics
- **3-Lives System**: Lose a life = respawn at start of current level
- **16x16 Grid-Based Movement**: Precise, tactical positioning
- **Pause-Anytime**: Perfect for casual TV viewing
- **Chrome 80+ Compatible**: Runs on older smart TVs from 2020+

## Controls

| Remote Button | Action |
|---------------|--------|
| D-Pad (Up/Down/Left/Right) | Move player |
| OK / Enter | Attack enemy / Select option |
| Back / Return | Open Pause Menu |

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

## Levels

| Level | Theme | Mechanic |
|-------|-------|----------|
| 1 | Tutorial Dungeon | Basic movement tutorial |
| 2 | Crystal Caves | Slippery floors |
| 3 | Fungus Forest | Spreading hazards |
| 4 | Ember Ruins | Fire jets |
| 5 | Frozen Tundra | Ice physics |
| 6 | Shadow Keep | Limited visibility |
| 7 | Desert Temple | Sand pits (slow movement) |
| 8 | Flooded Depths | Water sections |
| 9 | Sky Fortress | Wind gusts |
| 10 | Void Throne | Boss fight |

## Lives System

- Start with **3 lives**
- Lose a life → Respawn at the start of the current level
- Lose all 3 lives → Game Over, restart from Level 1
- Checkpoints are per-level (not mid-level)

## Tech Stack

- **Language**: Rust
- **Graphics**: macroquad (2D engine optimized for WASM)
- **Platform**: WebAssembly
- **Target**: Chrome 80+ (February 2020)

## Development

```bash
# Build WASM package
wasm-pack build --release --target web

# Serve locally for testing
npx serve www
```

### Player sprite assets (`assets/dg_knight/`)

The Blue Knight sheets are sliced using **fixed cell sizes** from the filenames (not guessed frame counts):

- **Idle**: `Blue Knight idle Sprite-sheet 16x16.png` — horizontal strip, **16×16 px** per frame. Number of frames = `floor(image_width / 16)`.
- **Run**: `Blue Knight run Sprite-sheet 16x17.png` — **16×17 px** per frame. Number of frames = `floor(image_width / 16)`.

Each frame is drawn at **64×64** (`PLAYER_DISPLAY_SIZE`). If you see two characters in one sprite or a tiny sliver, the PNG width is not a multiple of 16 or the wrong file is in that path.

### Terrain tilesets (`assets/dg_gathering_free_ver/`)

Level geometry uses **Set 1.0**, **Set 1.1**, and **Set 1.2** at **16×16 px** cells, scaled **2×** to match the game’s **32×32** grid (`TILE_SIZE`). Walls use **4-bit neighbor autotiling** on Set 1.1; floors pick among several cells from all three sheets for variety.

Required files (names must match exactly):

- `Set 1.0.png` (192×64)
- `Set 1.1.png` (288×112)
- `Set 1.2.png` (256×96)

If these fail to load, the level falls back to the old flat colored rectangles. Wall/floor **(col, row)** mappings live in `src/terrain.rs` — tweak `WALL_AUTOTILE_SET11` or `FLOOR_VARIANTS` if a tile looks wrong.

## Game Design Philosophy

Dungeon Diver respects the constraints of TV gaming:

- **Large text** (32px+) readable from 10ft distance
- **Simple controls** - no complex button combinations
- **Pause-friendly** - TV gamers are often distracted
- **Under 2MB WASM** - fast loading on older devices
- **Grid-based combat** - strategic, not twitch-based

## License

MIT
