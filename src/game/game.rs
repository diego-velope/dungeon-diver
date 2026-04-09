// Dungeon Diver - Game State & Main Game Loop
// Handles game states, update loop, and rendering

use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound, play_sound, stop_sound, PlaySoundParams, set_sound_volume};
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
    /// Stone plate for pause menu (same asset family as main menu).
    pause_plate: Option<Texture2D>,
    // --- Audio and Settings ---
    /// Looped on title / main menu (`intro_music.mp3`).
    pub intro_music: Option<Sound>,
    /// Looped during gameplay (`background_music.wav`).
    pub gameplay_music: Option<Sound>,
    sfx_coin: Option<Sound>,
    sfx_blue_coin: Option<Sound>,
    sfx_coin_bag: Option<Sound>,
    sfx_use_potion: Option<Sound>,
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
            pause_plate: None,
            intro_music: None,
            gameplay_music: None,
            sfx_coin: None,
            sfx_blue_coin: None,
            sfx_coin_bag: None,
            sfx_use_potion: None,
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
        self.intro_music = load_sound("assets/audio/intro_music.mp3").await.ok();
        self.gameplay_music = load_sound("assets/audio/background_music.wav").await.ok();
        self.sfx_coin = load_sound("assets/audio/coin.wav").await.ok();
        self.sfx_blue_coin = load_sound("assets/audio/blue_coin.wav").await.ok();
        self.sfx_coin_bag = load_sound("assets/audio/coin_bag.wav").await.ok();
        self.sfx_use_potion = load_sound("assets/audio/use_potion.wav").await.ok();
    }

    /// Call after assets load. Loops intro menu music; stops gameplay music if any.
    pub fn enter_title_music(&mut self) {
        if let Some(s) = &self.gameplay_music {
            stop_sound(s);
        }
        if let Some(s) = &self.intro_music {
            stop_sound(s);
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: self.game_settings.effective_music_volume(),
                },
            );
        }
    }

    /// Stops intro and loops gameplay music (master × music volume).
    pub fn enter_gameplay_music(&mut self) {
        if let Some(s) = &self.intro_music {
            stop_sound(s);
        }
        if let Some(s) = &self.gameplay_music {
            stop_sound(s);
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: self.game_settings.effective_music_volume(),
                },
            );
        }
    }

    fn play_sfx(&self, sfx: &Option<Sound>) {
        if let Some(s) = sfx {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: self.game_settings.effective_effects_volume(),
                },
            );
        }
    }

    fn play_item_pickup_sfx(&self, kind: ItemType) {
        match kind {
            ItemType::Coin => self.play_sfx(&self.sfx_coin),
            ItemType::BlueCoin => self.play_sfx(&self.sfx_blue_coin),
            ItemType::CoinBag => self.play_sfx(&self.sfx_coin_bag),
            ItemType::Potion | ItemType::BigPotion | ItemType::SmallPotion => {
                self.play_sfx(&self.sfx_use_potion)
            }
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
        if let Ok(tex) = load_texture("assets/images/plate.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.pause_plate = Some(tex);
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

        self.enter_gameplay_music();
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

        if let Some(s) = &self.intro_music {
            set_sound_volume(s, self.game_settings.effective_music_volume());
        }
        if let Some(s) = &self.gameplay_music {
            set_sound_volume(s, self.game_settings.effective_music_volume());
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
        let mut pickup_sfx: Vec<ItemType> = Vec::new();
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
                    let kind = item.item_type;
                    let value = item.collect();
                    pickup_sfx.push(kind);
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

        for k in pickup_sfx {
            self.play_item_pickup_sfx(k);
        }
    }

    fn update_pause_menu(&mut self, actions: &[InputAction]) {
        if self.settings_open {
            if self.game_settings.handle_options_input(actions) {
                self.settings_open = false;
            }
            return;
        }

        const MENU_ITEMS: &[&str] = &["Return to game", "Inventory", "Settings", "Exit game"];

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
                        3 => {
                            self.state = GameState::Title;
                            self.enter_title_music();
                        }
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
            if matches!(
                action,
                InputAction::Pause
                    | InputAction::Cancel
                    | InputAction::Confirm
                    | InputAction::Attack
            ) {
                self.state = GameState::PauseMenu;
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
                    self.enter_title_music();
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
        let start_y = TITLE_MENU_START_Y;
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
            let heart_size = 32.0;
            let heart_spacing = 36.0;
            let max_hearts = player.max_hp / 2;
            let coin_font = 24_u16;
            let coin_label = format!("COINS  {}", self.coins);
            let coin_dims = measure_text(&coin_label, self.ui_font.as_ref(), coin_font, 1.0);
            let row_gap = 8.0;
            let outer_pad = 14.0;

            let hearts_row_w = if max_hearts > 0 {
                (max_hearts as f32 - 1.0) * heart_spacing + heart_size
            } else {
                0.0
            };
            let content_w = hearts_row_w.max(coin_dims.width);
            let content_h = heart_size + row_gap + coin_dims.height;

            let panel_w = content_w + outer_pad * 2.0;
            let panel_h = content_h + outer_pad * 2.0 + 10.0;
            let panel_x = 0.0;
            let panel_y = 0.0;

            if let Some(ref plate) = self.pause_plate {
                draw_texture_ex(
                    plate,
                    panel_x,
                    panel_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(panel_w, panel_h)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(
                    panel_x,
                    panel_y,
                    panel_w,
                    panel_h,
                    Color::from_rgba(12, 14, 22, 200),
                );
                draw_rectangle_lines(
                    panel_x,
                    panel_y,
                    panel_w,
                    panel_h,
                    2.0,
                    Color::from_rgba(200, 200, 220, 120),
                );
            }

            let content_left = panel_x + (panel_w - content_w) / 2.0;
            let block_top = panel_y + outer_pad;

            // Hearts UI: health is tracked in half-hearts.
            let half_hp = player.hp.clamp(0, player.max_hp);
            let full_hearts = half_hp / 2;
            let has_half = (half_hp % 2) == 1;

            let atlas = self.items_atlas.as_ref();
            let hearts_left = content_left + (content_w - hearts_row_w) / 2.0;
            let hx0 = hearts_left + heart_size / 2.0;
            let hy = block_top + heart_size / 2.0;
            for i in 0..max_hearts {
                let i_f = i as f32;
                let x_center = hx0 + i_f * heart_spacing;
                let y_center = hy;
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

            let coin_x = content_left + (content_w - coin_dims.width) / 2.0;
            let coin_y = block_top + heart_size + row_gap + coin_dims.offset_y;
            draw_shadowed_text(
                &coin_label,
                coin_x,
                coin_y,
                coin_font,
                self.ui_font.as_ref(),
                Color::from_rgba(255, 220, 80, 255),
                Color::from_rgba(30, 20, 8, 255),
            );

            let level_text = format!("LEVEL {}", self.current_level);
            let level_font = 30_u16;
            let font = self.ui_font.as_ref();
            let tdims = measure_text(&level_text, font, level_font, 1.0);
            let badge_pad_x = 32.0;
            let badge_pad_y = 18.0;
            let badge_w = (tdims.width + badge_pad_x * 2.0).max(168.0);
            let badge_h = tdims.height + badge_pad_y * 2.0;
            let badge_x = SCREEN_W / 2.0 - badge_w / 2.0;
            let badge_y = 12.0;
            let badge_cx = SCREEN_W / 2.0;
            let badge_cy = badge_y + badge_h / 2.0;

            if let Some(ref plate) = self.pause_plate {
                draw_texture_ex(
                    plate,
                    badge_x,
                    badge_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(badge_w, badge_h)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(
                    badge_x,
                    badge_y,
                    badge_w,
                    badge_h,
                    Color::from_rgba(12, 14, 22, 200),
                );
                draw_rectangle_lines(
                    badge_x,
                    badge_y,
                    badge_w,
                    badge_h,
                    2.0,
                    Color::from_rgba(200, 200, 220, 120),
                );
            }
            draw_shadowed_text_centered(
                &level_text,
                badge_cx,
                badge_cy + 10.0,
                level_font,
                font,
                Color::from_rgba(245, 245, 252, 255),
                Color::from_rgba(20, 20, 30, 255),
            );
        }
    }

    fn draw_pause_menu(&self) {
        const MENU_ITEMS: &[&str] = &[
            "RETURN TO GAME",
            "INVENTORY",
            "SETTINGS",
            "EXIT GAME",
        ];

        draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            SCREEN_H,
            Color::from_rgba(0, 0, 0, 140),
        );

        let panel_side = PAUSE_MENU_PANEL_SIDE;
        let panel_x = (SCREEN_W - panel_side) / 2.0;
        let panel_y = (SCREEN_H - panel_side) / 2.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_side,
            panel_side,
            Color::from_rgba(14, 12, 22, 250),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_side,
            panel_side,
            3.0,
            Color::from_rgba(55, 50, 75, 220),
        );

        let title_font = 40_u16;
        let font = self.ui_font.as_ref();
        let title_dims = measure_text("PAUSED", font, title_font, 1.0);
        let title_cy = panel_y + 52.0;
        draw_shadowed_text_centered(
            "PAUSED",
            SCREEN_W / 2.0,
            title_cy,
            title_font,
            font,
            Color::from_rgba(255, 245, 230, 255),
            Color::from_rgba(25, 25, 35, 255),
        );
        let content_top = title_cy + title_dims.height / 2.0 + 32.0;

        let btn_w = PAUSE_MENU_BUTTON_W;
        let btn_h = PAUSE_MENU_BUTTON_H;
        let gap = PAUSE_MENU_BUTTON_GAP;
        let btn_x = (SCREEN_W - btn_w) / 2.0;

        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let y = content_top + i as f32 * (btn_h + gap);
            self.draw_menu_button(
                item,
                btn_x,
                y,
                btn_w,
                btn_h,
                i == self.pause_selection,
                false,
            );
        }
    }

    fn draw_settings_overlay(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::from_rgba(0, 0, 0, 170));
        let box_w = SCREEN_W * 0.64;
        let box_h = SCREEN_H * 0.78;
        let box_x = (SCREEN_W - box_w) / 2.0;
        // Shift panel up so content is not visually bottom-heavy.
        let box_y = ((SCREEN_H - box_h) / 2.0 - 36.0).max(12.0);

        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 50, 235));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 4.0, Color::from_rgba(255, 200, 50, 200));

        draw_shadowed_text_centered(
            "SETTINGS",
            SCREEN_W / 2.0,
            box_y + 48.0,
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
        let start_y = box_y + 118.0;

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
            const SETTINGS_VALUE_FONT: u16 = 28;
            const GAP_BAR_TO_VALUE: f32 = 10.0;
            let value_dims = measure_text(
                &value_text,
                self.ui_font.as_ref(),
                SETTINGS_VALUE_FONT,
                1.0,
            );
            // Macroquad: bbox top is at (text_y - offset_y) for draw_text_ex(..., text_y, ...).
            let value_text_y = bar_y + bar_h + GAP_BAR_TO_VALUE + value_dims.offset_y;
            let value_text_x = SCREEN_W / 2.0 - value_dims.width / 2.0;
            draw_shadowed_text(
                &value_text,
                value_text_x,
                value_text_y,
                SETTINGS_VALUE_FONT,
                self.ui_font.as_ref(),
                WHITE,
                Color::from_rgba(20, 20, 30, 255),
            );
        }

        draw_shadowed_text_centered(
            "UP / DOWN TO NAVIGATE - LEFT / RIGHT TO ADJUST - BACK TO CLOSE",
            SCREEN_W / 2.0,
            box_y + box_h - 28.0,
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

        // Macroquad places glyphs so the bbox top is at (text_y - dims.offset_y) for draw_text_ex(text, text_x, text_y, ...).
        // Vertical center of that bbox: text_y - offset_y + height/2 = y + h/2  =>  text_y = y + h/2 + offset_y - height/2.
        let font = self.ui_font.as_ref();
        let font_size = TITLE_MENU_FONT_SIZE;
        let dims = measure_text(label, font, font_size, 1.0);
        let text_x = x + w / 2.0 - dims.width / 2.0;
        let text_y = y + h / 2.0 + dims.offset_y - dims.height / 2.0;
        draw_shadowed_text(
            label,
            text_x,
            text_y,
            font_size,
            font,
            Color::from_rgba(240, 245, 255, 255),
            Color::from_rgba(20, 20, 30, 255),
        );
    }

    fn draw_inventory(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::from_rgba(0, 0, 0, 165));
        let box_w = SCREEN_W * 0.68;
        let box_h = SCREEN_H * 0.5;
        let box_x = (SCREEN_W - box_w) / 2.0;
        let box_y = (SCREEN_H - box_h) / 2.0;
        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 50, 235));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 4.0, Color::from_rgba(255, 200, 50, 200));
        draw_shadowed_text_centered(
            "INVENTORY",
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

    fn draw_game_over(&self) {
        draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.8 });

        draw_shadowed_text_centered(
            "GAME OVER",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0 - 36.0,
            TEXT_TITLE,
            self.ui_font.as_ref(),
            Color::from_rgba(255, 80, 80, 255),
            Color::from_rgba(40, 10, 10, 255),
        );

        draw_shadowed_text_centered(
            "PRESS OK TO RETURN TO MENU",
            SCREEN_W / 2.0,
            SCREEN_H / 2.0 + 32.0,
            TEXT_MEDIUM,
            self.ui_font.as_ref(),
            Color::from_rgba(240, 240, 250, 255),
            Color::from_rgba(20, 20, 30, 255),
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