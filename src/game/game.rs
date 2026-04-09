// Dungeon Diver - Game State & Main Game Loop
// Handles game states, update loop, and rendering

use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound, play_sound, PlaySoundParams, set_sound_volume};
use crate::config::*;
use crate::input::*;
use crate::rendering::Camera;
use crate::entities::Player;
use crate::world::*;
use crate::game::{GameSettings, ShutdownFlow};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuSubScreen {
    None,
    HowToPlay,
    Settings,
    QuitConfirm,
}

/// Main game structure
pub struct Game {
    pub state: GameState,
    pub level: Option<Level>,
    pub player: Option<Player>,
    pub camera: Camera,
    pub input: InputHandler,
    pub pause_selection: usize,
    menu_selection: usize,
    menu_sub_screen: MenuSubScreen,
    quit_confirm_close_focused: bool,
    menu_click_index: Option<usize>,
    menu_click_timer: f32,
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
    /// UI menu font
    pub ui_font: Option<Font>,
    /// Title screen background
    title_background: Option<Texture2D>,
    title_btn_focused: Option<Texture2D>,
    title_btn_unfocused: Option<Texture2D>,
    title_btn_clicked: Option<Texture2D>,
    // --- Audio and Settings ---
    pub bgm: Option<Sound>,
    pub settings_open: bool,
    pub game_settings: GameSettings,
    pub shutdown_flow: ShutdownFlow,
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
            menu_selection: 0,
            menu_sub_screen: MenuSubScreen::None,
            quit_confirm_close_focused: false,
            menu_click_index: None,
            menu_click_timer: 0.0,
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
            ui_font: None,
            title_background: None,
            title_btn_focused: None,
            title_btn_unfocused: None,
            title_btn_clicked: None,
            bgm: None,
            settings_open: false,
            game_settings: GameSettings::default(),
            shutdown_flow: ShutdownFlow::default(),
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
        if let Ok(ui_font_data) = load_file("assets/fonts/ThaleahFat.ttf").await {
            self.ui_font = load_ttf_font_from_bytes(&ui_font_data).ok();
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
        if let Ok(tex) = load_texture("assets/images/focused.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_focused = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/unfocused.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_unfocused = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/clicked.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_clicked = Some(tex);
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
                    volume: self.game_settings.effective_music_volume(),
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
        if self.menu_click_timer > 0.0 {
            self.menu_click_timer -= dt;
            if self.menu_click_timer <= 0.0 {
                self.menu_click_timer = 0.0;
                self.menu_click_index = None;
            }
        }

        let gameplay_dt = dt * self.game_settings.speed_multiplier();

        // State machine
        match self.state {
            GameState::Title => {
                self.update_title(&actions);
            }
            GameState::Playing => {
                self.update_playing(&actions, gameplay_dt);
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
                self.update_level_complete(&actions, gameplay_dt);
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

        if let Some(sound) = &self.bgm {
            set_sound_volume(sound, self.game_settings.effective_music_volume());
        }
    }

    fn update_title(&mut self, actions: &[InputAction]) {
        const MENU_ITEMS: [&str; 4] = ["PLAY", "HOW TO PLAY", "SETTINGS", "EXIT GAME"];

        match self.menu_sub_screen {
            MenuSubScreen::None => {
                for &action in actions {
                    match action {
                        InputAction::MoveDown => {
                            self.menu_selection = (self.menu_selection + 1) % MENU_ITEMS.len();
                        }
                        InputAction::MoveUp => {
                            self.menu_selection = if self.menu_selection == 0 {
                                MENU_ITEMS.len() - 1
                            } else {
                                self.menu_selection - 1
                            };
                        }
                        InputAction::Confirm | InputAction::Attack => {
                            self.menu_click_index = Some(self.menu_selection);
                            self.menu_click_timer = 0.12;
                            match self.menu_selection {
                                0 => self.start(),
                                1 => self.menu_sub_screen = MenuSubScreen::HowToPlay,
                                2 => {
                                    self.game_settings.focused_row = 0;
                                    self.menu_sub_screen = MenuSubScreen::Settings;
                                }
                                3 => {
                                    self.quit_confirm_close_focused = true;
                                    self.menu_sub_screen = MenuSubScreen::QuitConfirm;
                                }
                                _ => {}
                            }
                        }
                        InputAction::Cancel | InputAction::Pause => {
                            self.quit_confirm_close_focused = false;
                            self.menu_sub_screen = MenuSubScreen::QuitConfirm;
                        }
                        _ => {}
                    }
                }
            }
            MenuSubScreen::HowToPlay => {
                for &action in actions {
                    if matches!(
                        action,
                        InputAction::Cancel
                            | InputAction::Pause
                            | InputAction::Confirm
                            | InputAction::Attack
                    ) {
                        self.menu_sub_screen = MenuSubScreen::None;
                    }
                }
            }
            MenuSubScreen::Settings => {
                if self.game_settings.handle_options_input(actions) {
                    self.menu_sub_screen = MenuSubScreen::None;
                }
            }
            MenuSubScreen::QuitConfirm => {
                for &action in actions {
                    match action {
                        InputAction::Cancel | InputAction::Pause => {
                            self.menu_sub_screen = MenuSubScreen::None;
                        }
                        InputAction::MoveLeft
                        | InputAction::MoveRight
                        | InputAction::MoveUp
                        | InputAction::MoveDown => {
                            self.quit_confirm_close_focused = !self.quit_confirm_close_focused;
                        }
                        InputAction::Confirm | InputAction::Attack => {
                            if self.quit_confirm_close_focused {
                                self.shutdown_flow.request_close();
                            } else {
                                self.menu_sub_screen = MenuSubScreen::None;
                            }
                        }
                        _ => {}
                    }
                }
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
                        level.items.push(crate::world::Item::new(vase.grid_x, vase.grid_y, item_type));
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
            level.enemies.retain(|e| e.state != crate::world::EnemyState::Dead);

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
            if self.game_settings.handle_options_input(actions) {
                self.settings_open = false;
            }
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
                            self.game_settings.focused_row = 0;
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

        const MENU_ITEMS: [&str; 4] = ["PLAY", "HOW TO PLAY", "SETTINGS", "EXIT GAME"];
        let button_w = TITLE_MENU_BUTTON_W;
        let button_h = TITLE_MENU_BUTTON_H;
        let spacing = TITLE_MENU_BUTTON_GAP;
        let start_y = 250.0;
        let x = (SCREEN_W - button_w) / 2.0;

        for (i, label) in MENU_ITEMS.iter().enumerate() {
            let y = start_y + i as f32 * (button_h + spacing);
            let clicked = self.menu_click_index == Some(i) && self.menu_click_timer > 0.0;
            self.draw_menu_button(*label, x, y, button_w, button_h, i == self.menu_selection, clicked);
        }

        match self.menu_sub_screen {
            MenuSubScreen::None => {}
            MenuSubScreen::HowToPlay => self.draw_how_to_play_overlay(),
            MenuSubScreen::Settings => self.draw_settings_overlay(),
            MenuSubScreen::QuitConfirm => self.draw_quit_confirm_overlay(),
        }
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
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::from_rgba(0, 0, 0, 170));
        let box_w = SCREEN_W * 0.64;
        let box_h = SCREEN_H * 0.78;
        let box_x = (SCREEN_W - box_w) / 2.0;
        let box_y = (SCREEN_H - box_h) / 2.0;

        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 50, 235));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 4.0, Color::from_rgba(255, 200, 50, 200));

        draw_shadowed_text_centered(
            "SETTINGS",
            SCREEN_W / 2.0,
            box_y + 60.0,
            50,
            self.ui_font.as_ref(),
            Color::from_rgba(255, 245, 230, 255),
            Color::from_rgba(25, 25, 35, 255),
        );

        let labels = ["Master Volume", "Music Volume", "FX Volume", "Game Speed"];
        let values = [
            self.game_settings.master_volume,
            self.game_settings.music_volume,
            self.game_settings.effects_volume,
            self.game_settings.game_speed,
        ];
        let start_y = box_y + 140.0;

        for (i, (label, val)) in labels.iter().zip(values.iter()).enumerate() {
            let y = start_y + i as f32 * 105.0;
            let focused = self.game_settings.focused_row == i;
            let label_color = if focused {
                Color::from_rgba(85, 230, 120, 255)
            } else {
                Color::from_rgba(235, 235, 245, 255)
            };

            draw_shadowed_text_centered(
                label,
                SCREEN_W / 2.0,
                y,
                34,
                self.ui_font.as_ref(),
                label_color,
                Color::from_rgba(20, 20, 30, 255),
            );

            let bar_w = box_w * 0.6;
            let bar_h = 20.0;
            let bar_x = (SCREEN_W - bar_w) / 2.0;
            let bar_y = y + 18.0;

            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_rgba(60, 60, 80, 255));
            draw_rectangle(
                bar_x,
                bar_y,
                bar_w * (*val as f32 / 10.0),
                bar_h,
                if focused {
                    Color::from_rgba(85, 230, 120, 255)
                } else {
                    Color::from_rgba(90, 140, 205, 255)
                },
            );
            draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 2.0, Color::from_rgba(200, 200, 220, 180));

            let value_text = if i == 3 {
                format!("{}%", val * 10)
            } else {
                format!("{}/10", val)
            };
            draw_shadowed_text_centered(
                &value_text,
                SCREEN_W / 2.0,
                bar_y + 40.0,
                28,
                self.ui_font.as_ref(),
                WHITE,
                Color::from_rgba(20, 20, 30, 255),
            );
        }

        draw_shadowed_text_centered(
            "ARROWS TO ADJUST - BACK TO CLOSE",
            SCREEN_W / 2.0,
            box_y + box_h - 22.0,
            24,
            self.ui_font.as_ref(),
            Color::from_rgba(200, 200, 220, 255),
            Color::from_rgba(20, 20, 30, 255),
        );
    }

    fn draw_how_to_play_overlay(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::from_rgba(0, 0, 0, 165));
        let box_w = SCREEN_W * 0.68;
        let box_h = SCREEN_H * 0.5;
        let box_x = (SCREEN_W - box_w) / 2.0;
        let box_y = (SCREEN_H - box_h) / 2.0;
        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 50, 235));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 4.0, Color::from_rgba(255, 200, 50, 200));
        draw_shadowed_text_centered(
            "HOW TO PLAY",
            SCREEN_W / 2.0,
            box_y + 70.0,
            48,
            self.ui_font.as_ref(),
            WHITE,
            Color::from_rgba(20, 20, 30, 255),
        );
        draw_shadowed_text_centered(
            "COMING SOON",
            SCREEN_W / 2.0,
            box_y + box_h / 2.0,
            42,
            self.ui_font.as_ref(),
            Color::from_rgba(220, 220, 235, 255),
            Color::from_rgba(20, 20, 30, 255),
        );
        draw_shadowed_text_centered(
            "PRESS OK OR BACK",
            SCREEN_W / 2.0,
            box_y + box_h - 24.0,
            24,
            self.ui_font.as_ref(),
            Color::from_rgba(200, 200, 220, 255),
            Color::from_rgba(20, 20, 30, 255),
        );
    }

    fn draw_quit_confirm_overlay(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::from_rgba(0, 0, 0, 180));
        let box_w = SCREEN_W * 0.55;
        let box_h = SCREEN_H * 0.34;
        let box_x = (SCREEN_W - box_w) / 2.0;
        let box_y = (SCREEN_H - box_h) / 2.0;
        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 50, 235));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 4.0, Color::from_rgba(255, 200, 50, 200));

        draw_shadowed_text_centered(
            "EXIT THE GAME?",
            SCREEN_W / 2.0,
            box_y + 70.0,
            46,
            self.ui_font.as_ref(),
            WHITE,
            Color::from_rgba(20, 20, 30, 255),
        );

        let btn_w = box_w * 0.36;
        let btn_h = 72.0;
        let gap = box_w * 0.08;
        let start_x = box_x + (box_w - (btn_w * 2.0 + gap)) / 2.0;
        let y = box_y + box_h - 110.0;

        self.draw_menu_button("EXIT GAME", start_x, y, btn_w, btn_h, self.quit_confirm_close_focused, false);
        self.draw_menu_button(
            "CONTINUE",
            start_x + btn_w + gap,
            y,
            btn_w,
            btn_h,
            !self.quit_confirm_close_focused,
            false,
        );
    }

    fn draw_menu_button(
        &self,
        label: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        is_selected: bool,
        is_pressed: bool,
    ) {
        let tex = if is_pressed {
            self.title_btn_clicked.as_ref()
        } else if is_selected {
            self.title_btn_focused.as_ref()
        } else {
            self.title_btn_unfocused.as_ref()
        };

        if let Some(texture) = tex {
            draw_texture_ex(
                texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(
                x,
                y,
                w,
                h,
                if is_selected { UI_HIGHLIGHT } else { UI_BG },
            );
            draw_rectangle_lines(x, y, w, h, 2.0, UI_BORDER);
        }

        draw_shadowed_text_centered(
            label,
            x + w / 2.0,
            y + h / 2.0 + 4.0,
            34,
            self.ui_font.as_ref(),
            Color::from_rgba(240, 245, 255, 255),
            Color::from_rgba(20, 20, 30, 255),
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
    let dims = measure_text(text, params.font, params.font_size, 1.0);
    let final_params = TextParams {
        font_size: params.font_size,
        font: params.font,
        color: params.color,
        ..Default::default()
    };
    draw_text_ex(text, x - dims.width / 2.0, y - dims.height / 2.0, final_params);
}

fn draw_shadowed_text(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    font: Option<&Font>,
    color: Color,
    shadow_color: Color,
) {
    draw_text_ex(
        text,
        x + 2.0,
        y + 2.0,
        TextParams {
            font_size,
            font,
            color: shadow_color,
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size,
            font,
            color,
            ..Default::default()
        },
    );
}

fn draw_shadowed_text_centered(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    font: Option<&Font>,
    color: Color,
    shadow_color: Color,
) {
    let dims = measure_text(text, font, font_size, 1.0);
    let left = x - dims.width / 2.0;
    let top = y - dims.height / 2.0;
    draw_shadowed_text(text, left, top, font_size, font, color, shadow_color);
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