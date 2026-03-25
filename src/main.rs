// Dungeon Diver - Main Entry Point
// TV Browser Game - Rust + WASM
// Built for Chrome 80+ (smart TVs, Android TV)

#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_wraps)]

mod constants;
mod input;
mod level;
mod camera;
mod game;
mod player;
mod items;
mod terrain;
mod enemy;

use macroquad::prelude::*;
use game::Game;

/// Window configuration
fn window_conf() -> Conf {
    Conf {
        window_title: "Dungeon Diver".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        fullscreen: false,
        platform: macroquad::miniquad::conf::Platform::default(),
        sample_count: 1,
        high_dpi: false,
        ..Default::default()
    }
}

/// Main game loop
#[macroquad::main(window_conf)]
async fn main() {
    // Initialize game state
    let mut game = Game::new();

    // Preload player sprites + terrain tilesets
    game.load_player_sprites().await;
    game.load_terrain_atlas().await;
    game.load_items_atlas().await;
    game.load_enemy_atlas().await;
    game.load_font().await;
    game.load_audio().await;
    game.load_title_background().await;

    // Main game loop
    loop {
        // Delta time in seconds
        let dt = get_frame_time();

        // Update game logic
        game.update(dt);

        // Render
        game.draw();

        // Wait for next frame
        next_frame().await;
    }
}
