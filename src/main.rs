// Dungeon Diver - Main Entry Point
// TV Browser Game - Rust + WASM
// Built for Chrome 80+ (smart TVs, Android TV)

#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_wraps)]

mod config;
mod game;
mod input;
mod world;
mod entities;
mod rendering;
mod splash_html;

use macroquad::prelude::*;
use game::{Game, ShutdownStage, shutdown_game};
#[cfg(target_arch = "wasm32")]
pub use input::tv_input_manager::{
    mq_handle_action, mq_handle_back, mq_handle_down, mq_handle_left, mq_handle_right,
    mq_handle_up,
};

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
    #[cfg(target_arch = "wasm32")]
    input::tv_input_manager::init_tv_input_manager();

    // Initialize game state
    let mut game = Game::new();

    splash_html::set_loading_progress(0.0);
    game.load_player_sprites().await;
    splash_html::set_loading_progress(14.0);
    game.load_terrain_atlas().await;
    splash_html::set_loading_progress(28.0);
    game.load_items_atlas().await;
    splash_html::set_loading_progress(42.0);
    game.load_enemy_atlas().await;
    splash_html::set_loading_progress(54.0);
    game.load_hit_vfx().await;
    splash_html::set_loading_progress(64.0);
    game.load_font().await;
    splash_html::set_loading_progress(76.0);
    game.load_audio().await;
    splash_html::set_loading_progress(88.0);
    game.load_title_background().await;
    splash_html::set_loading_progress(100.0);
    game.enter_title_music();
    splash_html::hide_loading_splash();

    // Main game loop
    loop {
        // Delta time in seconds
        let dt = get_frame_time();

        // Update game logic
        game.update(dt);

        if game.shutdown_flow.stage == ShutdownStage::Requested {
            if let Some(s) = &game.intro_music {
                macroquad::audio::stop_sound(s);
            }
            if let Some(s) = &game.gameplay_music {
                macroquad::audio::stop_sound(s);
            }
            game.shutdown_flow.mark_finalizing();
        }

        if game.shutdown_flow.stage == ShutdownStage::Finalizing {
            shutdown_game();
            break;
        }

        // Render
        game.draw();

        // Wait for next frame
        next_frame().await;
    }
}
