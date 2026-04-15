# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **TV platform layer (PAL)** — `web/pal`: platform detection (Tizen, webOS, Vizio, Fire TV, Android TV, browser), per-platform key maps, Tizen media-key registration, and host shutdown hooks (e.g. webOS `platformBack`, Tizen app exit, Android `AndroidJsInterface.shutdown`).
- **WASM / HTML shell** — Miniquad plugin wiring `mq_shutdown_game` and `mq_handle_{up,down,left,right,action,back}`; patches `WebAssembly.instantiate` / `instantiateStreaming` to bind PAL key events to Rust exports after load.
- **Audio** — Intro/menu loop (`intro_music.mp3`) and gameplay loop (`background_music.wav`); SFX for coins, blue coin, coin bag, potions, key, and pickups; volumes respect in-game settings (master × music).
- **Combat hit VFX system** — New `src/game/hit_vfx.rs` module loads and renders `player_hit_spritesheet` / `enemy_hit_spritesheet` in-world, spawns effects one tile in front of attacker, updates by frame-time, and draws by facing direction with flip support.
- **Combat SFX outcomes** — Attack audio now maps to results: random `punch_1|punch_2` for enemy hits, random `sword_slash_1|2|4` on player misses, `sword_slash_3` on damaging player hits, and `sword_slash_finisher` on killing blows.
- **Settings** — In-game settings overlay (e.g. music volume) and title/pause menu layout constants.
- **Fonts** — Press Start 2P and ThaleahFat for UI.
- **Art assets** — UI kit, focus/click states, letters atlas, plate, loading GIF; gameplay background split (`background.png` vs `background_only.png`).
- **Rust modules** — `config`, `game` (with `settings` / `shutdown`), `input` (`input_handler`, `tv_input_manager` on WASM), `world`, `entities`, `rendering`.
- **Shutdown flow** — Coordinated exit: music stops, `shutdown_game`, and JS `mq_shutdown_game` / `TV_PAL.shutdown()`.

### Changed

- **Build** — `build.sh` uses project-root paths; packages `web/` → `dist/` (HTML, optional `web/*.js`, full `web/pal`), copies `assets/`, still fetches `mq_js_bundle.js` when missing.
- **Web entry / splash** — `web/index.html` now uses a Toon Dash-style splash with `assets/images/background.png`, `assets/images/loadingBar.png`, `@font-face` for `PressStart2P-Regular.ttf`, progress label/status text, and no controls hint.
- **WASM loading hooks** — Added `mq_set_progress` and `mq_hide_splash` integration so Rust reports incremental loading progress and hides splash only after asset preload completes.
- **Combat rules** — Melee now requires adjacent face-to-face orientation, enemy attacks are gated by a 3.0s cooldown, and player attacks are gated by a 1.5s cooldown with HUD readiness feedback.
- **HUD** — Added top-left mini attack cooldown bar (loading-bar frame + blue refill) showing when player attack is ready.
- **Lifecycle** — `visibilitychange` suspends/resumes wrapped `AudioContext` instances when the app is hidden or visible.
- **Enemy rendering** — Left/right share one side-profile sprite row with `flip_x`; display size via `ENEMY_DISPLAY_SIZE` (1.5× tile); clamped texture source rects; health bar aligned to larger sprite.
- **Project hygiene** — `.gitignore` prefers `.cursor/rules` over `.cursorrules`; handoff doc filename normalized to `DUNGEON_DIVER_HANDOFF.md`.

### Fixed

- **Enemy movement** — Movement interpolation uses `move_start_x` / `move_start_y` so motion tweens from the previous tile to the destination (avoids wrong direction when grid position already updated).
- **Enemy overlap** — Enemy movement validation now blocks stepping onto the player tile, preventing stacked overlap and repeated contact-kill behavior.
- **Player attack reliability** — Attack resolution no longer depends on the short animation window; introduced pending-hit consumption and timer reset at attack start to prevent missed Enter presses.
- **Player hit VFX rendering** — Removed legacy yellow debug attack circle and corrected spritesheet slicing (64×64 cells) / row selection so horizontal attacks render full effect instead of clipped tail frames.

### Removed

- **`.cursorrules/*.mdc`** — TV Cursor rule bundle removed from the repo on this branch (use `.cursor/rules` locally if needed).

---

When you cut a release, move items under `[Unreleased]` into a dated section such as `## [0.2.0] - YYYY-MM-DD` and set the version in `Cargo.toml` to match.
