// Dungeon Diver - Game State & Main Game Loop
// Handles game states, update loop, and rendering

use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound, play_sound, PlaySoundParams, set_sound_volume};
use crate::constants::*;
use crate::input::*;
use crate::level::*;
use crate::camera::Camera;
use crate::items::ItemsAtlas;
use crate::player::Player;
use crate::terrain::TerrainAtlas;
use crate::enemy::EnemyAtlas;

/// Game states for state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Title,
    Playing,
    PauseMenu,
    Inventory,
    GameOver,
    LevelComplete,
}

/// Main game structure
pub struct Game {
    pub state: GameState,
    pub level: Option<Level>,
    pub player: Option<Player>,
    pub camera: Camera,
    pub input: InputHandler,
    pub pause_selection: usize,
    pub screen_flash: f32,
    pub coins: i32,
    current_level: u8,

    // Level-complete transition timer (for door "pulse" -> load next).
    level_complete_timer: f32,
    level_complete_duration: f32,
    // Preloaded player textures
    player_idle_tex: Option<Texture2D>,
    player_run_tex: Option<Texture2D>,
    /// Gathering "Set" tile atlases for terrain (optional — falls back to flat colors)
    terrain: Option<TerrainAtlas>,
    /// 0x72 sprites for interactive items (chests, keys, etc).
    items_atlas: Option<ItemsAtlas>,
    /// Zombie enemy spritesheets
    enemy_atlas: Option<EnemyAtlas>,
    /// Custom pixel font
    pub font: Option<Font>,
    /// Title screen background
    title_background: Option<Texture2D>,
    // --- Audio and Settings ---
    pub bgm: Option<Sound>,
    pub music_volume: i32,
    pub pending_music_volume: i32,
    pub settings_open: bool,
    pub settings_selection: usize,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
            level: None,
            player: None,
            camera: Camera::new(),
            input: InputHandler::new(),
            pause_selection: 0,
            screen_flash: 0.0,
            coins: 0,
            current_level: 1,
            level_complete_timer: 0.0,
            level_complete_duration: 1.2,
            player_idle_tex: None,
            player_run_tex: None,
            terrain: None,
            items_atlas: None,
            enemy_atlas: None,
            font: None,
            title_background: None,
            bgm: None,
            music_volume: 10,
            pending_music_volume: 10,
            settings_open: false,
            settings_selection: 0,
        }
    }

    /// Load Set 1.0 / 1.1 / 1.2 terrain atlases
    pub async fn load_terrain_atlas(&mut self) {
        self.terrain = TerrainAtlas::load().await;
    }

    pub async fn load_items_atlas(&mut self) {
        self.items_atlas = ItemsAtlas::load().await;
    }

    pub async fn load_enemy_atlas(&mut self) {
        self.enemy_atlas = EnemyAtlas::load().await;
    }

    pub async fn load_font(&mut self) {
        if let Ok(font_data) = load_file("assets/fonts/PixelifySans-Regular.ttf").await {
            self.font = load_ttf_font_from_bytes(&font_data).ok();
        }
    }

    pub async fn load_audio(&mut self) {
        if let Ok(sound) = load_sound("assets/audio/water_and_flint.mp3").await {
            self.bgm = Some(sound);
        }
    }

    pub async fn load_title_background(&mut self) {
        if let Ok(tex) = load_texture("assets/images/background.png").await {
            tex.set_filter(FilterMode::Linear);
            self.title_background = Some(tex);
        }
    }

    /// Load player sprites (call before starting game)
    pub async fn load_player_sprites(&mut self) {
        // Load Blue Knight idle sprite
        if let Ok(tex) = load_texture("assets/dg_knight/Blue Knight idle Sprite-sheet 16x16.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.player_idle_tex = Some(tex);
        }

        // Load Blue Knight run sprite
        if let Ok(tex) = load_texture("assets/dg_knight/Blue Knight run Sprite-sheet 16x17.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.player_run_tex = Some(tex);
        }
    }

    /// Start a new game
    pub fn start(&mut self) {
        self.level = Some(Level::load_level_1());
        let level = self.level.as_ref().unwrap();

        // Create player at spawn point
        let mut player = Player::new(level.spawn_x, level.spawn_y);

        // Set preloaded sprites if available
        if let (Some(ref idle), Some(ref run)) = (&self.player_idle_tex, &self.player_run_tex) {
            player.set_sprites(idle.clone(), run.clone());
        }

        self.player = Some(player);

        // Set camera to player position
        self.camera.set_target(
            level.spawn_x as f32 * TILE_SIZE,
            level.spawn_y as f32 * TILE_SIZE,
        );

        self.state = GameState::Playing;
        self.screen_flash = 0.3; // Flash on level start
        self.coins = 0;
        self.current_level = 1;
        self.level_complete_timer = 0.0;

        // Start background music loop
        if let Some(sound) = &self.bgm {
            play_sound(
                &sound,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_volume as f32 / 10.0,
                },
            );
        }
    }

    /// Load player sprites (call once at startup)
    pub async fn load_assets(&mut self) {
        if let Some(ref mut player) = &mut self.player {
            player.load_sprites().await;
        }
    }

    /// Main update loop - call once per frame
    pub fn update(&mut self, dt: f32) {
        // Update input
        let actions = self.input.update(dt);

        // State machine
        match self.state {
            GameState::Title => {
                self.update_title(&actions);
            }
            GameState::Playing => {
                self.update_playing(&actions, dt);
            }
            GameState::PauseMenu => {
                self.update_pause_menu(&actions);
            }
            GameState::Inventory => {
                self.update_inventory(&actions);
            }
            GameState::GameOver => {
                self.update_game_over(&actions);
            }
            GameState::LevelComplete => {
                self.update_level_complete(&actions, dt);
            }
        }

        // Update camera to follow player
        if let Some(ref player) = self.player {
            self.camera.set_target(player.x, player.y);
        }
        self.camera.update(dt);
        if let Some(ref level) = self.level {
            let level_w = level.width as f32 * TILE_SIZE;
            let level_h = level.height as f32 * TILE_SIZE;
            self.camera.clamp_to_level(level_w, level_h);
        }

        // Update screen flash
        if self.screen_flash > 0.0 {
            self.screen_flash -= dt;
        }
    }

    fn update_title(&mut self, actions: &[InputAction]) {
        for &action in actions {
            match action {
                InputAction::Confirm | InputAction::Attack => {
                    self.start();
                }
                _ => {}
            }
        }
    }

    fn update_playing(&mut self, actions: &[InputAction], dt: f32) {
        // Check for pause
        for &action in actions {
            if action == InputAction::Pause {
                self.state = GameState::PauseMenu;
                self.pause_selection = 0;
                return;
            }
        }

        // Update level (torches, items)
        if let Some(ref mut level) = &mut self.level {
            level.update(dt);
        }

        // Update player and check collisions
        if let (Some(ref mut level), Some(ref mut player)) = (&mut self.level, &mut self.player) {
            player.update(dt, level, actions);

            // Chest/key logic: stepping on a chest starts its open animation.
            // When the animation completes, the key is granted and the door unlocks.
            for chest in &mut level.chests {
                chest.try_open(player.grid_x, player.grid_y);
            }
            if !player.has_key && level.chests.iter().any(|c| c.key_given) {
                player.has_key = true;
                level.door_unlocked = true;
            }

            // Check item collection
            for item in &mut level.items {
                if !item.collected && item.grid_x == player.grid_x && item.grid_y == player.grid_y {
                    let value = item.collect();
                    if value > 0 {
                        self.coins += value;
                        self.screen_flash = 0.2; // Small flash on pickup
                    }
                    // Handle potion healing
                    if value == POTION_HEAL {
                        player.heal(value);
                    }
                }
            }

            // Check vase breaking (player walks into vase)
            for vase in &mut level.vases {
                if !vase.broken && vase.grid_x == player.grid_x && vase.grid_y == player.grid_y {
                    if let Some(item_type) = vase.break_vase() {
                        // Spawn item from vase
                        level.items.push(crate::items::Item::new(vase.grid_x, vase.grid_y, item_type));
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // COMBAT: Player attack → Enemy damage
            // ═══════════════════════════════════════════════════════════════════════════════
            if player.is_attacking {
                let (attack_x, attack_y) = player.get_attack_position();
                for enemy in &mut level.enemies {
                    if enemy.is_alive() && enemy.grid_x == attack_x && enemy.grid_y == attack_y {
                        enemy.take_damage(1); // 1 damage per hit
                        player.is_attacking = false; // Reset attack after hit
                        break; // Only hit one enemy per attack
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // COMBAT: Enemy → Player damage (contact damage)
            // ═══════════════════════════════════════════════════════════════════════════════
            for enemy in &level.enemies {
                if enemy.is_alive() && enemy.grid_x == player.grid_x && enemy.grid_y == player.grid_y {
                    if player.invincible_time <= 0.0 {
                        player.take_damage(ENEMY_DAMAGE);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // ENEMY UPDATES: AI and movement
            // ═══════════════════════════════════════════════════════════════════════════════
            let player_pos = (player.grid_x, player.grid_y);
            // Collect level dimensions for bounds checking
            let level_w = level.width as i32;
            let level_h = level.height as i32;

            for enemy in &mut level.enemies {
                enemy.update_with_bounds(dt, player_pos, level_w, level_h, &level.tiles);
            }
            // Remove enemies whose death animation has finished.
            level.enemies.retain(|e| e.state != crate::enemy::EnemyState::Dead);

            // Check if player reached exit
            if player.at_exit(level) && player.has_key {
                self.state = GameState::LevelComplete;
                self.level_complete_timer = self.level_complete_duration;
            }

            // Check if player died
            if !player.is_alive() {
                self.state = GameState::GameOver;
            }
        }
    }

    fn update_pause_menu(&mut self, actions: &[InputAction]) {
        if self.settings_open {
            self.update_settings(actions);
            return;
        }

        const MENU_ITEMS: &[&str] = &["Return to game", "Inventory", "Options", "Exit game"];

        for &action in actions {
            match action {
                InputAction::MoveDown => {
                    self.pause_selection = (self.pause_selection + 1) % MENU_ITEMS.len();
                }
                InputAction::MoveUp => {
                    self.pause_selection = if self.pause_selection == 0 {
                        MENU_ITEMS.len() - 1
                    } else {
                        self.pause_selection - 1
                    };
                }
                InputAction::Confirm | InputAction::Attack => {
                    match self.pause_selection {
                        0 => self.state = GameState::Playing, // Return to game
                        1 => self.state = GameState::Inventory, // Inventory
                        2 => {
                            self.settings_open = true;
                            self.pending_music_volume = self.music_volume;
                            self.settings_selection = 0;
                        }
                        3 => self.state = GameState::Title, // Exit game
                        _ => {}
                    }
                }
                InputAction::Pause | InputAction::Cancel => {
                    self.state = GameState::Playing;
                }
                _ => {}
            }
        }
    }

    fn update_settings(&mut self, actions: &[InputAction]) {
        for &action in actions {
            match action {
                InputAction::MoveLeft => {
                    if self.settings_selection == 0 {
                        self.pending_music_volume = (self.pending_music_volume - 1).max(0);
                    }
                }
                InputAction::MoveRight => {
                    if self.settings_selection == 0 {
                        self.pending_music_volume = (self.pending_music_volume + 1).min(10);
                    }
                }
                InputAction::MoveDown | InputAction::MoveUp => {
                    self.settings_selection = (self.settings_selection + 1) % 2;
                }
                InputAction::Confirm | InputAction::Attack => {
                    if self.settings_selection == 1 {
                        // Save
                        self.music_volume = self.pending_music_volume;
                        self.settings_open = false;
                        // Apply volume live
                        if let Some(sound) = &self.bgm {
                            set_sound_volume(&sound, self.music_volume as f32 / 10.0);
                        }
                    } else {
                        // Toggle to Save button
                        self.settings_selection = 1;
                    }
                }
                InputAction::Cancel | InputAction::Pause => {
                    self.settings_open = false;
                    self.pending_music_volume = self.music_volume;
                }
                _ => {}
            }
        }
    }

    fn update_inventory(&mut self, actions: &[InputAction]) {
        for &action in actions {
            match action {
                InputAction::Pause | InputAction::Cancel => {
                    self.state = GameState::PauseMenu;
                }
                _ => {}
            }
        }
    }

    fn update_game_over(&mut self, actions: &[InputAction]) {
        for &action in actions {
            match action {
                InputAction::Confirm | InputAction::Attack => {
                    self.state = GameState::Title;
                    self.player = None;
                    self.level = None;
                }
                _ => {}
            }
        }
    }

    fn update_level_complete(&mut self, actions: &[InputAction], dt: f32) {
        // Allow skipping the transition with OK/Enter, but default is automatic.
        for &action in actions {
            if matches!(action, InputAction::Confirm | InputAction::Attack) {
                self.level_complete_timer = 0.0;
            }
        }

        if self.level_complete_timer > 0.0 {
            self.level_complete_timer -= dt;
        }

        if self.level_complete_timer <= 0.0 {
            // Load next level; keep coin score persistent.
            self.current_level = self.current_level.saturating_add(1);
            let next = match self.current_level {
                2 => Level::load_level_2(),
                3 => Level::load_level_3(),
                _ => Level::load_level_3(), // fallback for level 4+
            };
            self.load_level_and_spawn_player(next);
        }
    }

    fn load_level_and_spawn_player(&mut self, level: Level) {
        let spawn_x = level.spawn_x;
        let spawn_y = level.spawn_y;

        self.level = Some(level);
        let mut player = Player::new(spawn_x, spawn_y);

        // Re-apply preloaded sprites if available (native/wasm asset load).
        if let (Some(ref idle), Some(ref run)) = (&self.player_idle_tex, &self.player_run_tex) {
            player.set_sprites(idle.clone(), run.clone());
        }

        self.player = Some(player);
        self.camera.set_target(
            spawn_x as f32 * TILE_SIZE,
            spawn_y as f32 * TILE_SIZE,
        );

        self.state = GameState::Playing;
        self.screen_flash = 0.2;
        self.level_complete_timer = 0.0;
    }

    /// Main draw loop - call once per frame
    pub fn draw(&self) {
        // Clear screen with background gradient
        clear_background(LIGHTGRAY);

        match self.state {
            GameState::Title => {
                self.draw_title();
            }
            GameState::Playing => {
                self.draw_playing();
            }
            GameState::PauseMenu => {
                self.draw_playing(); // Draw game behind menu
                if self.settings_open {
                    self.draw_settings_overlay();
                } else {
                    self.draw_pause_menu();
                }
            }
            GameState::Inventory => {
                self.draw_playing(); // Draw game behind inventory
                self.draw_inventory();
            }
            GameState::GameOver => {
                self.draw_playing(); // Draw game behind text
                self.draw_game_over();
            }
            GameState::LevelComplete => {
                self.draw_playing(); // Draw game behind text
                self.draw_level_complete();
            }
        }

        // Draw screen flash
        if self.screen_flash > 0.0 {
            draw_rectangle(
                0.0, 0.0, SCREEN_W, SCREEN_H,
                Color { r: 1.0, g: 1.0, b: 1.0, a: self.screen_flash }
            );
        }
    }

    fn draw_title(&self) {
        // Draw background image
        if let Some(ref bg) = self.title_background {
            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(SCREEN_W, SCREEN_H)),
                    ..Default::default()
                },
            );
        } else {
            // Fallback: gradient background
            for i in 0..SCREEN_H as i32 {
                let t = i as f32 / SCREEN_H;
                let color = Color {
                    r: LEVEL1_PALETTE.bg_top.r * (1.0 - t) + LEVEL1_PALETTE.bg_bot.r * t,
                    g: LEVEL1_PALETTE.bg_top.g * (1.0 - t) + LEVEL1_PALETTE.bg_bot.g * t,
                    b: LEVEL1_PALETTE.bg_top.b * (1.0 - t) + LEVEL1_PALETTE.bg_bot.b * t,
                    a: 1.0,
                };
                draw_rectangle(0.0, i as f32, SCREEN_W, 1.0, color);
            }
        }

        // Title
        let title = "DUNGEON DIVER";
        let subtitle = "Press ENTER or OK to Start";

        // Draw semi-transparent black box behind text
        let box_padding = 10.0;
        let box_width = 400.0;
        let box_height = 140.0;
        let box_x = SCREEN_W / 2.0 - box_width / 2.0;
        let box_y = SCREEN_H / 2.0 - box_height / 2.0;

        draw_rectangle(
            box_x - box_padding,
            box_y - box_padding,
            box_width + box_padding * 2.0,
            box_height + box_padding * 2.0,
            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 }
        );

        // Draw title text - centered properly
        draw_text_ex_centered(
            title,
            SCREEN_W / 2.0 - 40.0,
            SCREEN_H / 2.0 - 40.0,
            TextParams {
                font_size: TEXT_TITLE,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.text,
                ..Default::default()
            },
        );

        draw_text_ex_centered(
            subtitle,
            SCREEN_W / 2.0 - 40.0,
            SCREEN_H / 2.0 + 40.0,
            TextParams {
                font_size: TEXT_MEDIUM,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.text,
                ..Default::default()
            },
        );
    }

    fn draw_playing(&self) {
        if let Some(ref level) = self.level {
            // Draw background
            draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, level.palette.bg_bot);

            // Get camera offset
            let (cam_x, cam_y) = self.camera.get_render_offset();

            // Draw level (textured terrain if atlases loaded)
            level.draw(cam_x, cam_y, self.terrain.as_ref(), self.items_atlas.as_ref());

            // Draw enemies (behind player)
            if let Some(ref atlas) = self.enemy_atlas {
                for enemy in &level.enemies {
                    if enemy.is_alive() {
                        enemy.draw(cam_x, cam_y, atlas);
                    }
                }
            }

            // Draw player
            if let Some(ref player) = self.player {
                player.draw(cam_x, cam_y);
            }

            // Draw HUD (hearts, coins, level label)
            self.draw_hud();
        }
    }

    fn draw_hud(&self) {
        if let Some(ref player) = self.player {
            let padding = 20.0;
            let heart_size = 32.0;
            let heart_spacing = 40.0;

            // Hearts UI: health is tracked in half-hearts.
            let half_hp = player.hp.clamp(0, player.max_hp);
            let full_hearts = half_hp / 2;
            let has_half = (half_hp % 2) == 1;
            let max_hearts = player.max_hp / 2;

            let atlas = self.items_atlas.as_ref();
            for i in 0..max_hearts {
                let i_f = i as f32;
                let x_center = padding + i_f * heart_spacing;
                let y_center = padding;
                let top_left_x = x_center - heart_size / 2.0;
                let top_left_y = y_center - heart_size / 2.0;

                if let Some(atlas) = atlas {
                    if i < full_hearts {
                        draw_texture_ex(
                            &atlas.heart_full,
                            top_left_x,
                            top_left_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(heart_size, heart_size)),
                                ..Default::default()
                            },
                        );
                    } else if i == full_hearts && has_half {
                        draw_texture_ex(
                            &atlas.heart_half,
                            top_left_x,
                            top_left_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(heart_size, heart_size)),
                                ..Default::default()
                            },
                        );
                    } else {
                        draw_texture_ex(
                            &atlas.heart_empty,
                            top_left_x,
                            top_left_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(heart_size, heart_size)),
                                ..Default::default()
                            },
                        );
                    }
                } else {
                    // Fallback: procedural hearts.
                    if i < full_hearts {
                        draw_heart(x_center, y_center, heart_size, LEVEL1_PALETTE.accent);
                    } else if i == full_hearts && has_half {
                        let half_color = Color { r: LEVEL1_PALETTE.accent.r, g: LEVEL1_PALETTE.accent.g, b: LEVEL1_PALETTE.accent.b, a: 0.6 };
                        draw_heart(x_center, y_center, heart_size, half_color);
                    } else {
                        draw_heart_outline(x_center, y_center, heart_size, UI_BORDER);
                    }
                }
            }

            // Draw coins counter
            let coin_text = format!("Coins: {}", self.coins);
            draw_text_ex(
                &coin_text,
                padding,
                padding + heart_size + 10.0,
                TextParams {
                    font_size: TEXT_NORMAL,
                    font: self.font.as_ref(),
                    color: YELLOW,
                    ..Default::default()
                },
            );

            // Draw level label - top center (more padding from top)
            let level_text = format!("Level {}", self.current_level);
            draw_text_ex_centered(
                &level_text,
                SCREEN_W / 2.0,
                padding + 35.0,
                TextParams {
                    font_size: TEXT_LARGE,
                    font: self.font.as_ref(),
                    color: LEVEL1_PALETTE.text,
                    ..Default::default()
                },
            );
        }
    }

    fn draw_pause_menu(&self) {
        const MENU_ITEMS: &[&str] = &["Return to game", "Inventory", "Options", "Exit game"];

        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color { r: 0.0, g: 0.0, b: 0.1, a: 0.5 });

        // Menu box
        let menu_w = 450.0;
        let menu_h = MENU_ITEMS.len() as f32 * MENU_ITEM_HEIGHT + MENU_PADDING * 4.0;
        let menu_x = (SCREEN_W - menu_w) / 2.0;
        let menu_y = (SCREEN_H - menu_h) / 2.0;

        draw_rectangle(menu_x, menu_y, menu_w, menu_h, UI_BG);
        draw_rectangle_lines(menu_x, menu_y, menu_w, menu_h, 3.0, UI_BORDER);

        draw_text_ex_centered(
            "PAUSED",
            (SCREEN_W / 2.0) - 15.0,
            menu_y + 60.0,
            TextParams {
                font_size: TEXT_LARGE,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.accent,
                ..Default::default()
            },
        );

        // Draw menu items
        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let item_y = menu_y + MENU_PADDING + 80.0 + i as f32 * MENU_ITEM_HEIGHT;

            // Highlight selected item
            if i == self.pause_selection {
                draw_rectangle(
                    menu_x + 20.0,
                    item_y - 30.0,
                    menu_w - 40.0,
                    MENU_ITEM_HEIGHT - 10.0,
                    UI_HIGHLIGHT
                );
            }

            // Draw text
            let color = if i == self.pause_selection { WHITE } else { LEVEL1_PALETTE.text };
            draw_text_ex_centered(
                item,
                (SCREEN_W / 2.0) - 15.0,
                item_y + 15.0,
                TextParams {
                    font_size: TEXT_NORMAL,
                    font: self.font.as_ref(),
                    color,
                    ..Default::default()
                },
            );
        }
    }

    fn draw_settings_overlay(&self) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color { r: 0.0, g: 0.0, b: 0.1, a: 0.5 });

        let menu_w = 500.0;
        let menu_h = 400.0;
        let menu_x = (SCREEN_W - menu_w) / 2.0;
        let menu_y = (SCREEN_H - menu_h) / 2.0;

        draw_rectangle(menu_x, menu_y, menu_w, menu_h, UI_BG);
        draw_rectangle_lines(menu_x, menu_y, menu_w, menu_h, 3.0, UI_BORDER);

        draw_text_ex_centered(
            "SETTINGS",
            (SCREEN_W / 2.0) - 20.0,
            menu_y + 60.0,
            TextParams {
                font_size: TEXT_LARGE,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.accent,
                ..Default::default()
            },
        );

        // Music Volume Row
        let vol_y = menu_y + 160.0;
        let is_vol_sel = self.settings_selection == 0;

        if is_vol_sel {
            draw_rectangle(menu_x + 30.0, vol_y - 40.0, menu_w - 60.0, 100.0, UI_HIGHLIGHT);
        }

        let vol_text = format!("Music Volume: {}/10", self.pending_music_volume);
        draw_text_ex_centered(
            &vol_text,
            (SCREEN_W / 2.0) - 15.0,
            vol_y,
            TextParams {
                font_size: TEXT_NORMAL,
                font: self.font.as_ref(),
                color: if is_vol_sel { WHITE } else { LEVEL1_PALETTE.text },
                ..Default::default()
            },
        );

        // Volume bar
        let bar_w = 300.0;
        let bar_h = 10.0;
        let bar_x = (SCREEN_W - bar_w) / 2.0;
        let bar_y = vol_y + 10.0;
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color { r: 0.2, g: 0.2, b: 0.3, a: 1.0 });
        let fill_w = bar_w * (self.pending_music_volume as f32 / 10.0);
        let bar_color = if is_vol_sel { WHITE } else { LEVEL1_PALETTE.accent };
        draw_rectangle(bar_x, bar_y, fill_w, bar_h, bar_color);

        draw_text_ex_centered(
            "< Left / Right >",
            SCREEN_W / 2.0,
            vol_y + 60.0,
            TextParams {
                font_size: TEXT_SMALL,
                font: self.font.as_ref(),
                color: if is_vol_sel { WHITE } else { LEVEL1_PALETTE.text },
                ..Default::default()
            },
        );

        // Save Button
        let save_y = menu_y + 320.0;
        let is_save_sel = self.settings_selection == 1;

        if is_save_sel {
            draw_rectangle(menu_x + 100.0, save_y - 30.0, menu_w - 200.0, 60.0, UI_HIGHLIGHT);
        }

        draw_text_ex_centered(
            "SAVE & RETURN",
            (SCREEN_W / 2.0) - 15.0,
            save_y + 10.0,
            TextParams {
                font_size: TEXT_NORMAL,
                font: self.font.as_ref(),
                color: if is_save_sel { WHITE } else { LEVEL1_PALETTE.text },
                ..Default::default()
            },
        );
    }

    fn draw_inventory(&self) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, UI_BG);

        draw_text_ex_centered(
            "INVENTORY",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0,
            TextParams {
                font_size: TEXT_LARGE,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.text,
                ..Default::default()
            },
        );

        draw_text_ex_centered(
            "Press BACK to return",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0 + 50.0,
            TextParams {
                font_size: TEXT_NORMAL,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.accent,
                ..Default::default()
            },
        );
    }

    fn draw_game_over(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.8 });

        draw_text_ex_centered(
            "GAME OVER",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0 - 30.0,
            TextParams {
                font_size: TEXT_TITLE,
                font: self.font.as_ref(),
                color: RED,
                ..Default::default()
            },
        );

        draw_text_ex_centered(
            "Press ENTER to restart",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0 + 30.0,
            TextParams {
                font_size: TEXT_MEDIUM,
                font: self.font.as_ref(),
                color: LEVEL1_PALETTE.text,
                ..Default::default()
            },
        );
    }

    fn draw_level_complete(&self) {
        // Dark overlay + a short pulse around the exit door.
        let overlay_a = 0.65;
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color { r: 0.0, g: 0.0, b: 0.0, a: overlay_a });

        if let Some(ref level) = self.level {
            let (cam_x, cam_y) = self.camera.get_render_offset();
            let door_x = level.exit_x as f32 * TILE_SIZE - cam_x;
            let door_y = level.exit_y as f32 * TILE_SIZE - cam_y;

            // Pulse decreases as timer runs out.
            let t = (self.level_complete_timer / self.level_complete_duration).clamp(0.0, 1.0);
            let pulse = 0.15 + (t * std::f32::consts::PI * 2.0).sin().abs() * 0.20;

            // Outer glow
            draw_rectangle(
                door_x - 4.0,
                door_y - 4.0,
                TILE_SIZE + 8.0,
                TILE_SIZE + 8.0,
                Color { r: 0.4, g: 0.8, b: 1.0, a: pulse },
            );
            // Inner accent
            draw_rectangle(
                door_x,
                door_y,
                TILE_SIZE,
                TILE_SIZE,
                Color { r: LEVEL1_PALETTE.accent.r, g: LEVEL1_PALETTE.accent.g, b: LEVEL1_PALETTE.accent.b, a: pulse * 0.7 },
            );
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to draw centered text
fn draw_text_ex_centered(text: &str, x: f32, y: f32, params: TextParams) {
    let dims = measure_text(text, None, params.font_size, 1.0);
    let final_params = TextParams {
        font_size: params.font_size,
        font: params.font,
        color: params.color,
        ..Default::default()
    };
    draw_text_ex(text, x - dims.width / 2.0, y - dims.height / 2.0, final_params);
}

/// Draw a heart shape for HUD
fn draw_heart(x: f32, y: f32, size: f32, color: Color) {
    let s = size / 2.0;
    // Simple heart shape using two circles and a triangle
    draw_circle(x - s / 2.0, y - s / 2.0, s / 2.0, color);
    draw_circle(x + s / 2.0, y - s / 2.0, s / 2.0, color);
    // Triangle pointing down
    let points = vec![
        Vec2::new(x - s, y - s / 4.0),
        Vec2::new(x + s, y - s / 4.0),
        Vec2::new(x, y + s),
    ];
    draw_triangle(
        points[0], points[1], points[2], color
    );
}

/// Draw a heart outline (for empty hearts)
fn draw_heart_outline(x: f32, y: f32, size: f32, color: Color) {
    let s = size / 2.0;
    draw_circle_lines(x - s / 2.0, y - s / 2.0, s / 2.0, 2.0, color);
    draw_circle_lines(x + s / 2.0, y - s / 2.0, s / 2.0, 2.0, color);
    // Outline for triangle part
    draw_line(
        x - s, y - s / 4.0,
        x, y + s,
        2.0, color
    );
    draw_line(
        x + s, y - s / 4.0,
        x, y + s,
        2.0, color
    );
}