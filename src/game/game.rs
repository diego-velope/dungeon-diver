// Dungeon Diver - Game State & Main Game Loop
// Handles game states, update loop, and rendering

use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound, play_sound, stop_sound, PlaySoundParams, set_sound_volume};
use crate::config::*;
use crate::input::*;
use crate::rendering::Camera;
use crate::entities::Player;
use crate::world::tiled_visual::TiledVisualMap;
use crate::world::*;
use crate::game::{GameSettings, ShutdownFlow};
use crate::game::hit_vfx::{HitVfxAtlas, HitVfxInstance, HitVfxKind};

#[derive(Debug, Clone, Copy)]
enum CombatSfxEvent {
    EnemyPunch,
    PlayerSwingMiss,
    PlayerSwingHit,
    PlayerSwingFinisher,
}

/// Game states for state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Title,
    Playing,
    PauseMenu,
    Inventory,
    GameOver,
    LevelComplete,
    /// Beat level 10 — full-screen win + stats.
    Victory,
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
    /// Red vignette on real damage; decays with `screen_flash` in `update()`.
    damage_flash: f32,
    pub coins: i32,
    current_level: u8,
    /// Run stats (reset when starting a new game from the title / Play again).
    run_enemies_killed: i32,
    run_healing_potions: i32,
    run_shield_potions: i32,
    win_menu_selection: usize,

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
    /// Full-screen art after beating level 10.
    winner_screen: Option<Texture2D>,
    title_btn_focused: Option<Texture2D>,
    title_btn_unfocused: Option<Texture2D>,
    title_btn_clicked: Option<Texture2D>,
    /// Stone plate for pause menu (same asset family as main menu).
    pause_plate: Option<Texture2D>,
    /// Frame used by mini attack cooldown bar in HUD.
    hud_loading_bar: Option<Texture2D>,
    /// Combat hit effects.
    hit_vfx_atlas: Option<HitVfxAtlas>,
    hit_vfx_instances: Vec<HitVfxInstance>,
    spike_timer: f32,
    /// Tileset PNGs referenced by TMX levels (L1, L2, … pre-scanned in `load_tiled_textures`).
    tiled_textures: HashMap<String, Texture2D>,
    // --- Audio and Settings ---
    /// Looped on title / main menu (`intro_music.mp3`).
    pub intro_music: Option<Sound>,
    /// Looped during gameplay (`background_music.wav`).
    pub gameplay_music: Option<Sound>,
    sfx_coin: Option<Sound>,
    sfx_blue_coin: Option<Sound>,
    sfx_coin_bag: Option<Sound>,
    sfx_use_potion: Option<Sound>,
    sfx_punch_1: Option<Sound>,
    sfx_punch_2: Option<Sound>,
    sfx_sword_slash_1: Option<Sound>,
    sfx_sword_slash_2: Option<Sound>,
    sfx_sword_slash_3: Option<Sound>,
    sfx_sword_slash_4: Option<Sound>,
    sfx_sword_slash_finisher: Option<Sound>,
    pub settings_open: bool,
    pub game_settings: GameSettings,
    pub shutdown_flow: ShutdownFlow,
}

impl Game {
    /// Camera shake + red flash (enemy hit or spikes).
    fn trigger_hurt_reaction(&mut self) {
        self.camera.shake();
        self.damage_flash = HURT_REACTION_FLASH;
    }

    fn spawn_hit_vfx(&mut self, kind: HitVfxKind, tile_x: i32, tile_y: i32, facing: Direction) {
        if self.hit_vfx_atlas.is_some() {
            self.hit_vfx_instances.push(HitVfxInstance::spawn(kind, tile_x, tile_y, facing));
        }
    }

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
            damage_flash: 0.0,
            coins: 0,
            current_level: 1,
            run_enemies_killed: 0,
            run_healing_potions: 0,
            run_shield_potions: 0,
            win_menu_selection: 0,
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
            winner_screen: None,
            title_btn_focused: None,
            title_btn_unfocused: None,
            title_btn_clicked: None,
            pause_plate: None,
            hud_loading_bar: None,
            hit_vfx_atlas: None,
            hit_vfx_instances: Vec::new(),
            spike_timer: 0.0,
            tiled_textures: HashMap::new(),
            intro_music: None,
            gameplay_music: None,
            sfx_coin: None,
            sfx_blue_coin: None,
            sfx_coin_bag: None,
            sfx_use_potion: None,
            sfx_punch_1: None,
            sfx_punch_2: None,
            sfx_sword_slash_1: None,
            sfx_sword_slash_2: None,
            sfx_sword_slash_3: None,
            sfx_sword_slash_4: None,
            sfx_sword_slash_finisher: None,
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

    pub async fn load_hit_vfx(&mut self) {
        self.hit_vfx_atlas = HitVfxAtlas::load().await;
    }

    /// Parse TMX level files (preloaded on WASM) to collect tileset PNG paths, then load textures.
    /// Must be called after `preload_level_tmx_for_wasm` and before `start()`.
    pub async fn load_tiled_textures(&mut self) {
        let mut paths: HashSet<String> = HashSet::new();
        for tmx in [
            "assets/levels/level1.tmx",
            "assets/levels/level2.tmx",
            "assets/levels/level3.tmx",
        ] {
            match crate::world::tmx_loader::load_level_from_tmx(tmx) {
                Ok(level) => {
                    if let Some(raw) = level.tiled_visual_raw {
                        for p in raw.image_paths() {
                            paths.insert(p);
                        }
                    } else {
                        error!("load_tiled_textures: no TiledVisualRaw in {tmx}");
                    }
                }
                Err(e) => {
                    error!("load_tiled_textures: failed to parse {tmx} — {e}");
                }
            }
        }

        if paths.is_empty() {
            return;
        }
        let path_vec: Vec<String> = paths.into_iter().collect();
        info!("Tiled: loading {} unique tileset PNG(s): {:?}", path_vec.len(), path_vec);

        for path in &path_vec {
            if self.tiled_textures.contains_key(path) {
                continue;
            }
            match load_texture(path).await {
                Ok(tex) => {
                    tex.set_filter(FilterMode::Nearest);
                    self.tiled_textures.insert(path.clone(), tex);
                }
                Err(e) => {
                    error!("Tiled texture load failed for '{path}': {e}");
                }
            }
        }

        info!("Tiled textures loaded: {}/{}", self.tiled_textures.len(), path_vec.len());
    }

    /// Rebuilds [`Level::tiled_visual`] from the current level’s TMX raw + loaded tileset textures.
    fn refresh_level_tiled_visual(&mut self) {
        if let Some(level) = self.level.as_mut() {
            if let Some(raw) = level.tiled_visual_raw.clone() {
                if !self.tiled_textures.is_empty() {
                    level.tiled_visual = Some(TiledVisualMap::build(raw, self.tiled_textures.clone()));
                    return;
                }
            }
            level.tiled_visual = None;
        }
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
        // quad-snd can panic on unsupported formats on some desktop targets
        // (observed on macOS with MP3), so intro music uses a WAV copy.
        self.intro_music = load_sound("assets/audio/intro_music.wav").await.ok();
        self.gameplay_music = load_sound("assets/audio/background_music.wav").await.ok();
        self.sfx_coin = load_sound("assets/audio/coin.wav").await.ok();
        self.sfx_blue_coin = load_sound("assets/audio/blue_coin.wav").await.ok();
        self.sfx_coin_bag = load_sound("assets/audio/coin_bag.wav").await.ok();
        self.sfx_use_potion = load_sound("assets/audio/use_potion.wav").await.ok();
        self.sfx_punch_1 = load_sound("assets/audio/punch_1.wav").await.ok();
        self.sfx_punch_2 = load_sound("assets/audio/punch_2.wav").await.ok();
        self.sfx_sword_slash_1 = load_sound("assets/audio/sword_slash_1.wav").await.ok();
        self.sfx_sword_slash_2 = load_sound("assets/audio/sword_slash_2.wav").await.ok();
        self.sfx_sword_slash_3 = load_sound("assets/audio/sword_slash_3.wav").await.ok();
        self.sfx_sword_slash_4 = load_sound("assets/audio/sword_slash_4.wav").await.ok();
        self.sfx_sword_slash_finisher = load_sound("assets/audio/sword_slash_finisher.wav").await.ok();
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
            ItemType::Potion
            | ItemType::BigPotion
            | ItemType::SmallPotion
            | ItemType::ShieldPotion
            | ItemType::BigShieldPotion => {
                self.play_sfx(&self.sfx_use_potion)
            }
            ItemType::Key => self.play_sfx(&self.sfx_coin),
        }
    }

    fn play_combat_sfx(&self, event: CombatSfxEvent) {
        match event {
            CombatSfxEvent::EnemyPunch => {
                if macroquad::rand::gen_range(0, 2) == 0 {
                    self.play_sfx(&self.sfx_punch_1);
                } else {
                    self.play_sfx(&self.sfx_punch_2);
                }
            }
            CombatSfxEvent::PlayerSwingMiss => match macroquad::rand::gen_range(0, 3) {
                0 => self.play_sfx(&self.sfx_sword_slash_1),
                1 => self.play_sfx(&self.sfx_sword_slash_2),
                _ => self.play_sfx(&self.sfx_sword_slash_4),
            },
            CombatSfxEvent::PlayerSwingHit => self.play_sfx(&self.sfx_sword_slash_3),
            CombatSfxEvent::PlayerSwingFinisher => self.play_sfx(&self.sfx_sword_slash_finisher),
        }
    }

    pub async fn load_title_background(&mut self) {
        if let Ok(tex) = load_texture("assets/images/backgrounds/background.png").await {
            tex.set_filter(FilterMode::Linear);
            self.title_background = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/ui/focused.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_focused = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/ui/unfocused.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_unfocused = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/ui/clicked.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.title_btn_clicked = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/ui/plate.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.pause_plate = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/ui/loadingBar.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.hud_loading_bar = Some(tex);
        }
        if let Ok(tex) = load_texture("assets/images/backgrounds/winner_screen.png").await {
            tex.set_filter(FilterMode::Linear);
            self.winner_screen = Some(tex);
        }
    }

    /// Load player sprites (call before starting game)
    pub async fn load_player_sprites(&mut self) {
        if let Ok(tex) = load_texture("assets/sprites/player/bk-idle-spritesheet.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.player_idle_tex = Some(tex);
        }

        if let Ok(tex) = load_texture("assets/sprites/player/bk-run-spritesheet.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.player_run_tex = Some(tex);
        }
    }

    /// Start a new game
    pub fn start(&mut self) {
        self.run_enemies_killed = 0;
        self.run_healing_potions = 0;
        self.run_shield_potions = 0;
        self.win_menu_selection = 0;

        self.level = Some(Level::load_level_1());
        self.refresh_level_tiled_visual();
        if self.level.as_ref().is_some_and(|l| l.tiled_visual.is_some()) {
            info!("TiledVisualMap attached to current level");
        }

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
        self.spike_timer = 0.0;

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
            GameState::Victory => {
                self.update_victory(&actions);
            }
        }

        // Update camera to follow player
        if let Some(ref player) = self.player {
            self.camera.set_target(player.x, player.y);
        }
        self.camera.update(dt);
        self.spike_timer += dt;
        if let Some(ref level) = self.level {
            let level_w = level.width as f32 * TILE_SIZE;
            let level_h = level.height as f32 * TILE_SIZE;
            self.camera.clamp_to_level(level_w, level_h);
        }

        // Update screen flash
        if self.screen_flash > 0.0 {
            self.screen_flash -= dt;
        }
        if self.damage_flash > 0.0 {
            self.damage_flash -= dt;
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

        if let Some(ref atlas) = self.hit_vfx_atlas {
            for fx in &mut self.hit_vfx_instances {
                fx.update(dt, atlas);
            }
            self.hit_vfx_instances.retain(HitVfxInstance::is_active);
        }

        // Update player and check collisions
        let mut pickup_sfx: Vec<ItemType> = Vec::new();
        let mut queued_vfx: Vec<(HitVfxKind, i32, i32, Direction)> = Vec::new();
        let mut combat_sfx_events: Vec<CombatSfxEvent> = Vec::new();
        let mut should_open_gates = false;
        let mut want_hurt_reaction = false;
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
                level.open_all_gates();
            }

            // Check item collection
            for item in &mut level.items {
                if !item.collected && item.grid_x == player.grid_x && item.grid_y == player.grid_y {
                    let kind = item.item_type;
                    let value = item.collect();
                    pickup_sfx.push(kind);
                    match kind {
                        ItemType::Coin | ItemType::BlueCoin | ItemType::CoinBag => {
                            if value > 0 {
                                self.coins += value;
                                self.screen_flash = 0.2; // Small flash on pickup
                            }
                        }
                        ItemType::Potion | ItemType::BigPotion | ItemType::SmallPotion => {
                            if value > 0 {
                                player.heal(value);
                                self.run_healing_potions += 1;
                            }
                        }
                        ItemType::ShieldPotion | ItemType::BigShieldPotion => {
                            if value > 0 {
                                player.add_shield(value);
                                self.run_shield_potions += 1;
                            }
                        }
                        ItemType::Key => {
                            player.has_key = true;
                            level.door_unlocked = true;
                            should_open_gates = true;
                        }
                    }
                }
            }

            if should_open_gates {
                level.open_all_gates();
            }

            // Floor buttons: opens portcullises only (does not grant exit key unless you add a key pickup).
            let mut gate_from_button = false;
            for btn in &mut level.buttons {
                if !btn.triggered && btn.grid_x == player.grid_x && btn.grid_y == player.grid_y {
                    btn.triggered = true;
                    gate_from_button = true;
                }
            }
            if gate_from_button {
                level.open_all_gates();
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

            // ═══════════════════════════════════════════════════════════════════════════════
            // COMBAT: Player attack → Enemy damage (face-to-face melee)
            // ═══════════════════════════════════════════════════════════════════════════════
            // One resolve per swing via `melee_hit_pending` — do not rely on `is_attacking`,
            // which can clear in the same frame as `start_attack` if `anim_timer` was stale.
            if player.consume_melee_hit_pending() {
                let (attack_x, attack_y) = player.get_attack_position();
                queued_vfx.push((HitVfxKind::PlayerHit, attack_x, attack_y, player.facing));
                let mut hit_enemy = false;

                for enemy in &mut level.enemies {
                    if !enemy.is_alive() {
                        continue;
                    }
                    if enemy.grid_x == attack_x
                        && enemy.grid_y == attack_y
                        && enemy.facing == player.facing.opposite()
                    {
                        hit_enemy = true;
                        let will_kill = enemy.hp <= 1;
                        enemy.take_damage(1);
                        if will_kill {
                            self.run_enemies_killed += 1;
                            combat_sfx_events.push(CombatSfxEvent::PlayerSwingFinisher);
                        } else {
                            combat_sfx_events.push(CombatSfxEvent::PlayerSwingHit);
                        }
                        break;
                    }
                }

                if !hit_enemy {
                    let mut hit_vase = false;
                    for vase in &mut level.vases {
                        if vase.grid_x != attack_x || vase.grid_y != attack_y {
                            continue;
                        }
                        if vase.broken {
                            break;
                        }
                        if vase.shatter_timer > 0.0 {
                            // Already flickering from a prior swing: still a “hit” (no whiff SFX).
                            hit_vase = true;
                            break;
                        }
                        if vase.start_shatter_windup() {
                            hit_vase = true;
                            combat_sfx_events.push(CombatSfxEvent::PlayerSwingHit);
                        }
                        break;
                    }
                    if !hit_vase {
                        combat_sfx_events.push(CombatSfxEvent::PlayerSwingMiss);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // COMBAT: Enemy → Player damage (face-to-face melee + cooldown)
            // ═══════════════════════════════════════════════════════════════════════════════
            for enemy in &mut level.enemies {
                if !enemy.is_alive() || enemy.attack_cooldown > 0.0 {
                    continue;
                }
                let (attack_x, attack_y) = enemy.get_attack_position();
                if attack_x == player.grid_x
                    && attack_y == player.grid_y
                    && player.facing == enemy.facing.opposite()
                {
                    if player.invincible_time <= 0.0 {
                        if player.take_damage(enemy.damage_for()) {
                            want_hurt_reaction = true;
                        }
                    }
                    enemy.attack_cooldown = enemy.attack_cooldown_for();
                    queued_vfx.push((HitVfxKind::EnemyHit, attack_x, attack_y, enemy.facing));
                    combat_sfx_events.push(CombatSfxEvent::EnemyPunch);
                }
            }

            let spike_offset = ((player.grid_x * 7 + player.grid_y * 13) % 4) as f32 * 0.5;
            let spike_local = (self.spike_timer + spike_offset) % 3.0;
            let spikes_active = spike_local < 1.0;
            if spikes_active && level.get_tile(player.grid_x, player.grid_y) == Tile::Spikes {
                if player.take_damage(1) {
                    want_hurt_reaction = true;
                }
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

        if want_hurt_reaction {
            self.trigger_hurt_reaction();
        }

        for k in pickup_sfx {
            self.play_item_pickup_sfx(k);
        }
        for (kind, x, y, facing) in queued_vfx {
            self.spawn_hit_vfx(kind, x, y, facing);
        }
        for event in combat_sfx_events {
            self.play_combat_sfx(event);
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
            if self.current_level == 10 {
                self.state = GameState::Victory;
                self.win_menu_selection = 0;
                self.enter_title_music();
                return;
            }
            // Load next level; keep coin score persistent.
            self.current_level = self.current_level.saturating_add(1);
            let next = match self.current_level {
                2 => Level::load_level_2(),
                3 => Level::load_level_3(),
                4 => Level::load_level_4(),
                5 => Level::load_level_5(),
                6 => Level::load_level_6(),
                7 => Level::load_level_7(),
                8 => Level::load_level_8(),
                9 => Level::load_level_9(),
                10 => Level::load_level_10(),
                _ => {
                    self.state = GameState::Title;
                    self.enter_title_music();
                    return;
                }
            };
            self.load_level_and_spawn_player(next);
        }
    }

    fn update_victory(&mut self, actions: &[InputAction]) {
        const MENU_ITEMS: [&str; 2] = ["PLAY AGAIN", "MAIN MENU"];
        for &action in actions {
            match action {
                InputAction::MoveDown => {
                    self.win_menu_selection = (self.win_menu_selection + 1) % MENU_ITEMS.len();
                }
                InputAction::MoveUp => {
                    self.win_menu_selection = if self.win_menu_selection == 0 {
                        MENU_ITEMS.len() - 1
                    } else {
                        self.win_menu_selection - 1
                    };
                }
                InputAction::Confirm | InputAction::Attack => {
                    self.menu_click_index = Some(self.win_menu_selection);
                    self.menu_click_timer = 0.12;
                    match self.win_menu_selection {
                        0 => self.start(),
                        1 => {
                            self.state = GameState::Title;
                            self.player = None;
                            self.level = None;
                            self.enter_title_music();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn load_level_and_spawn_player(&mut self, level: Level) {
        let spawn_x = level.spawn_x;
        let spawn_y = level.spawn_y;

        let prev_hp = self.player.as_ref().map(|p| p.hp);
        let prev_max_hp = self.player.as_ref().map(|p| p.max_hp);
        let prev_shields = self.player.as_ref().map(|p| p.shield_charges);

        self.level = Some(level);
        self.refresh_level_tiled_visual();
        let mut player = Player::new(spawn_x, spawn_y);

        if let Some(hp) = prev_hp {
            player.hp = hp;
        }
        if let Some(max_hp) = prev_max_hp {
            player.max_hp = max_hp;
        }
        if let Some(shields) = prev_shields {
            player.shield_charges = shields;
        }

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
            GameState::Victory => {
                self.draw_victory();
            }
        }

        // Draw screen flash
        if self.screen_flash > 0.0 {
            draw_rectangle(
                0.0, 0.0, SCREEN_W, SCREEN_H,
                Color { r: 1.0, g: 1.0, b: 1.0, a: self.screen_flash }
            );
        }
        if self.damage_flash > 0.0 {
            let a = (self.damage_flash * 1.15).min(0.4);
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color { r: 0.9, g: 0.12, b: 0.1, a },
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
            level.draw(cam_x, cam_y, self.terrain.as_ref(), self.items_atlas.as_ref(), self.spike_timer);

            // Draw enemies (behind player)
            if let Some(ref atlas) = self.enemy_atlas {
                for enemy in &level.enemies {
                    if enemy.is_alive() {
                        enemy.draw(cam_x, cam_y, atlas);
                    }
                }
            }

            // Tall Tiled sprites (door leaves, decor, etc.): Y-sort vs player — except the `columns`
            // layer, which always draws under the player (16×48 column art). Bases north-west of the
            // player draw before; south-east draw after for those layers.
            if let Some(ref player) = self.player {
                let player_depth_x = (player.x / TILE_SIZE).floor() as i32;
                let player_depth_y = (player.y / TILE_SIZE).floor() as i32;
                level.draw_foreground_before_player(cam_x, cam_y, player_depth_x, player_depth_y);
                player.draw(cam_x, cam_y);
                level.draw_foreground_after_player(cam_x, cam_y, player_depth_x, player_depth_y);
            } else {
                // No player: draw every tall tile in the “before” bucket (sentinel sorts all bases south of it).
                level.draw_foreground_before_player(cam_x, cam_y, i32::MAX, i32::MAX);
            }
            // 16px sconce art deferred from `Level::draw` (same paths as `is_deferred_sconce_path`).
            level.draw_tiled_sconce_overlay(cam_x, cam_y);
            // Exit door: TMX keeps closed tiles; overlay open leaf when `door_unlocked` (key/chest).
            level.draw_exit_door_unlock_overlay(cam_x, cam_y, self.items_atlas.as_ref());

            if let Some(ref atlas) = self.hit_vfx_atlas {
                for fx in &self.hit_vfx_instances {
                    fx.draw(cam_x, cam_y, atlas);
                }
            }

            // Draw HUD (hearts, coins, level label)
            self.draw_hud();
        }
    }

    fn draw_hud(&self) {
        if let Some(ref player) = self.player {
            let font = self.ui_font.as_ref();
            let atlas = self.items_atlas.as_ref();

            // ── TOP-LEFT: compact panel with hearts (row 1) + shield pills (row 2) ──
            let heart_size = 32.0;
            let heart_gap = 6.0;
            let max_hearts = player.max_hp / 2;
            let half_hp = player.hp.clamp(0, player.max_hp);
            let full_hearts = half_hp / 2;
            let has_half = (half_hp % 2) == 1;

            let shield_bar_w = 46.0;
            let shield_bar_h = 18.0;
            let shield_gap = 8.0;
            let shield_count = MAX_SHIELD_CHARGES as usize;

            let pad = 12.0;
            let inner_row_w_hearts = max_hearts as f32 * heart_size + (max_hearts - 1).max(0) as f32 * heart_gap;
            let inner_row_w_shields = shield_count as f32 * shield_bar_w + (shield_count - 1).max(0) as f32 * shield_gap;
            let inner_w = inner_row_w_hearts.max(inner_row_w_shields);
            let row_gap = 4.0;
            let inner_h = heart_size + row_gap + shield_bar_h;

            let panel_w = inner_w + pad * 2.0;
            let panel_h = inner_h + pad * 2.0;
            let panel_x = 4.0;
            let panel_y = 4.0;

            if let Some(ref plate) = self.pause_plate {
                draw_texture_ex(plate, panel_x, panel_y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(panel_w, panel_h)),
                    ..Default::default()
                });
            } else {
                draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_rgba(12, 14, 22, 200));
                draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::from_rgba(200, 200, 220, 120));
            }

            let hearts_x0 = panel_x + (panel_w - inner_row_w_hearts) / 2.0;
            let hearts_y = panel_y + pad;
            for i in 0..max_hearts {
                let hx = hearts_x0 + i as f32 * (heart_size + heart_gap);
                if let Some(a) = atlas {
                    let tex = if i < full_hearts {
                        &a.heart_full
                    } else if i == full_hearts && has_half {
                        &a.heart_half
                    } else {
                        &a.heart_empty
                    };
                    draw_texture_ex(tex, hx, hearts_y, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(heart_size, heart_size)),
                        ..Default::default()
                    });
                } else {
                    let cx = hx + heart_size / 2.0;
                    let cy = hearts_y + heart_size / 2.0;
                    if i < full_hearts {
                        draw_heart(cx, cy, heart_size, LEVEL1_PALETTE.accent);
                    } else if i == full_hearts && has_half {
                        draw_heart(cx, cy, heart_size, Color { r: LEVEL1_PALETTE.accent.r, g: LEVEL1_PALETTE.accent.g, b: LEVEL1_PALETTE.accent.b, a: 0.6 });
                    } else {
                        draw_heart_outline(cx, cy, heart_size, UI_BORDER);
                    }
                }
            }

            let shields_x0 = panel_x + (panel_w - inner_row_w_shields) / 2.0;
            let shields_y = hearts_y + heart_size + row_gap;
            for idx in 0..shield_count {
                let sx = shields_x0 + idx as f32 * (shield_bar_w + shield_gap);
                let active = (idx as i32) < player.shield_charges.max(0);
                let fill = if active {
                    Color::from_rgba(48, 130, 255, 245)
                } else {
                    Color::from_rgba(120, 125, 135, 220)
                };
                let r = shield_bar_h / 2.0;
                draw_rectangle(sx + r, shields_y, shield_bar_w - r * 2.0, shield_bar_h, fill);
                draw_circle(sx + r, shields_y + r, r, fill);
                draw_circle(sx + shield_bar_w - r, shields_y + r, r, fill);
                if active {
                    let hi = Color::from_rgba(130, 200, 255, 120);
                    draw_rectangle(sx + r, shields_y, shield_bar_w - r * 2.0, shield_bar_h * 0.35, hi);
                }
            }

            // ── BOTTOM-LEFT: coins text + attack bar + label ─────────────
            let bl_x = 14.0;
            let coin_font_sz = 26_u16;
            let coin_label = format!("COINS: {}", self.coins);
            let coin_dims = measure_text(&coin_label, font, coin_font_sz, 1.0);

            let atk_bar_w = 220.0;
            let atk_bar_h = 40.0;
            let atk_label_sz = 18_u16;
            let atk_ready = player.attack_ready_percent();
            let atk_label = if atk_ready >= 0.999 { "ATK READY" } else { "ATK RECHARGE" };
            let atk_label_dims = measure_text(atk_label, font, atk_label_sz, 1.0);

            let bottom_block_h = coin_dims.height + 6.0 + atk_bar_h + 4.0;
            let bl_y = SCREEN_H - bottom_block_h - 14.0;

            draw_shadowed_text(
                &coin_label,
                bl_x,
                bl_y + coin_dims.offset_y,
                coin_font_sz,
                font,
                Color::from_rgba(255, 220, 80, 255),
                Color::from_rgba(30, 20, 8, 255),
            );

            let bar_y = bl_y + coin_dims.height + 6.0;
            if let Some(ref frame) = self.hud_loading_bar {
                draw_texture_ex(frame, bl_x, bar_y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(atk_bar_w, atk_bar_h)),
                    ..Default::default()
                });
            } else {
                draw_rectangle(bl_x, bar_y, atk_bar_w, atk_bar_h, Color::from_rgba(18, 20, 28, 220));
                draw_rectangle_lines(bl_x, bar_y, atk_bar_w, atk_bar_h, 2.0, Color::from_rgba(180, 190, 220, 180));
            }
            let fill_inset_x = 14.0;
            let fill_inset_y = 10.0;
            let fill_w = (atk_bar_w - fill_inset_x * 2.0) * atk_ready;
            let fill_h = atk_bar_h - fill_inset_y * 2.0;
            draw_rectangle(bl_x + fill_inset_x, bar_y + fill_inset_y, fill_w, fill_h, Color::from_rgba(48, 130, 255, 245));
            draw_rectangle(bl_x + fill_inset_x, bar_y + fill_inset_y, fill_w, fill_h * 0.33, Color::from_rgba(136, 190, 255, 190));

            draw_shadowed_text(
                atk_label,
                bl_x + atk_bar_w + 10.0,
                bar_y + (atk_bar_h / 2.0) + atk_label_dims.offset_y - atk_label_dims.height / 2.0,
                atk_label_sz,
                font,
                Color::from_rgba(200, 224, 255, 255),
                Color::from_rgba(22, 30, 56, 255),
            );

            // ── TOP-CENTER: level badge ──────────────────────────────────
            let level_text = format!("LEVEL {}", self.current_level);
            let level_font = 30_u16;
            let tdims = measure_text(&level_text, font, level_font, 1.0);
            let badge_pad_x = 32.0;
            let badge_pad_y = 18.0;
            let badge_w = (tdims.width + badge_pad_x * 2.0).max(168.0);
            let badge_h = tdims.height + badge_pad_y * 2.0;
            let badge_x = SCREEN_W / 2.0 - badge_w / 2.0;
            let badge_y = 12.0;

            if let Some(ref plate) = self.pause_plate {
                draw_texture_ex(plate, badge_x, badge_y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(badge_w, badge_h)),
                    ..Default::default()
                });
            } else {
                draw_rectangle(badge_x, badge_y, badge_w, badge_h, Color::from_rgba(12, 14, 22, 200));
                draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 2.0, Color::from_rgba(200, 200, 220, 120));
            }
            draw_shadowed_text_centered(
                &level_text,
                SCREEN_W / 2.0,
                badge_y + badge_h / 2.0 + 10.0,
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
            let (door_x, door_y, door_w, door_h) = level.exit_zone_screen_rect(cam_x, cam_y);

            // Pulse decreases as timer runs out.
            let t = (self.level_complete_timer / self.level_complete_duration).clamp(0.0, 1.0);
            let pulse = 0.15 + (t * std::f32::consts::PI * 2.0).sin().abs() * 0.20;

            // Outer glow
            draw_rectangle(
                door_x - 4.0,
                door_y - 4.0,
                door_w + 8.0,
                door_h + 8.0,
                Color { r: 0.4, g: 0.8, b: 1.0, a: pulse },
            );
            // Inner accent
            draw_rectangle(
                door_x,
                door_y,
                door_w,
                door_h,
                Color { r: LEVEL1_PALETTE.accent.r, g: LEVEL1_PALETTE.accent.g, b: LEVEL1_PALETTE.accent.b, a: pulse * 0.7 },
            );
        }
    }

    /// Single-line text on a `plate.png` panel (same style as the in-game level badge).
    fn draw_plate_line_centered(&self, cx: f32, y_top: f32, text: &str, font_size: u16) -> f32 {
        let font = self.ui_font.as_ref();
        let tdims = measure_text(text, font, font_size, 1.0);
        let pad_x = 32.0;
        let pad_y = 18.0;
        let badge_w = (tdims.width + pad_x * 2.0).max(200.0);
        let badge_h = tdims.height + pad_y * 2.0;
        let badge_x = cx - badge_w / 2.0;
        if let Some(ref plate) = self.pause_plate {
            draw_texture_ex(
                plate,
                badge_x,
                y_top,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(badge_w, badge_h)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(badge_x, y_top, badge_w, badge_h, Color::from_rgba(12, 14, 22, 200));
            draw_rectangle_lines(badge_x, y_top, badge_w, badge_h, 2.0, Color::from_rgba(200, 200, 220, 120));
        }
        // Optical vertical center in plate (font bbox sits slightly high at raw geometric center).
        let text_cy = y_top + badge_h / 2.0 + tdims.offset_y * 0.35 + 6.0;
        draw_shadowed_text_centered(
            text,
            cx,
            text_cy,
            font_size,
            font,
            Color::from_rgba(245, 245, 252, 255),
            Color::from_rgba(20, 20, 30, 255),
        );
        y_top + badge_h + 16.0
    }

    fn plate_stats_badge_height(&self, lines: &[String], font_size: u16) -> f32 {
        let font = self.ui_font.as_ref();
        let line_spacing = 10.0;
        let mut content_h = 0.0_f32;
        for (i, line) in lines.iter().enumerate() {
            let d = measure_text(line, font, font_size, 1.0);
            content_h += d.height;
            if i + 1 < lines.len() {
                content_h += line_spacing;
            }
        }
        let pad_y = 22.0;
        content_h + pad_y * 2.0
    }

    fn draw_plate_stats_centered(&self, cx: f32, y_top: f32, lines: &[String], font_size: u16) -> f32 {
        let font = self.ui_font.as_ref();
        let line_spacing = 10.0;
        let mut max_w = 0.0_f32;
        let mut heights: Vec<f32> = Vec::new();
        for line in lines {
            let d = measure_text(line, font, font_size, 1.0);
            max_w = max_w.max(d.width);
            heights.push(d.height);
        }
        let content_h: f32 = heights.iter().sum::<f32>() + line_spacing * (lines.len().saturating_sub(1)) as f32;
        let pad_x = 36.0;
        let pad_y = 22.0;
        let badge_w = (max_w + pad_x * 2.0).max(280.0);
        let badge_h = content_h + pad_y * 2.0;
        let badge_x = cx - badge_w / 2.0;
        if let Some(ref plate) = self.pause_plate {
            draw_texture_ex(
                plate,
                badge_x,
                y_top,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(badge_w, badge_h)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(badge_x, y_top, badge_w, badge_h, Color::from_rgba(12, 14, 22, 200));
            draw_rectangle_lines(badge_x, y_top, badge_w, badge_h, 2.0, Color::from_rgba(200, 200, 220, 120));
        }
        // measure_text heights sit a bit above true glyph extent; nudge down for optical centering in the plate.
        let stats_block_nudge_y = 8.0_f32;
        let block_top = y_top + (badge_h - content_h) / 2.0 + stats_block_nudge_y;
        let mut y = block_top;
        for line in lines {
            let d = measure_text(line, font, font_size, 1.0);
            let line_center = y + d.height / 2.0;
            draw_shadowed_text_centered(
                line,
                cx,
                line_center,
                font_size,
                font,
                Color::from_rgba(235, 238, 248, 255),
                Color::from_rgba(20, 20, 30, 255),
            );
            y += d.height + line_spacing;
        }
        y_top + badge_h + 16.0
    }

    fn draw_victory(&self) {
        if let Some(ref tex) = self.winner_screen {
            draw_texture_ex(
                tex,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(SCREEN_W, SCREEN_H)),
                    ..Default::default()
                },
            );
        } else {
            clear_background(Color::from_rgba(20, 16, 28, 255));
        }

        let cx = SCREEN_W / 2.0;

        let button_w = TITLE_MENU_BUTTON_W;
        let button_h = TITLE_MENU_BUTTON_H;
        let spacing = TITLE_MENU_BUTTON_GAP;
        let bottom_margin = 28.0;
        let btn_x = (SCREEN_W - button_w) / 2.0;
        let first_button_y = SCREEN_H - bottom_margin - 2.0 * button_h - spacing;

        let stat_lines = vec![
            format!("ENEMIES KILLED: {}", self.run_enemies_killed),
            format!("HEALING POTIONS TAKEN: {}", self.run_healing_potions),
            format!("SHIELD POTIONS TAKEN: {}", self.run_shield_potions),
            format!("TOTAL COINS: {}", self.coins),
        ];
        let stats_font = 22_u16;
        let stats_h = self.plate_stats_badge_height(&stat_lines, stats_font);
        let stats_gap = 20.0;
        let stats_y_top = first_button_y - stats_gap - stats_h;

        let mut y = 32.0;
        y = self.draw_plate_line_centered(cx, y, "Thanks for playing!", 26);
        self.draw_plate_line_centered(cx, y, "You have found the treasure!", 24);

        self.draw_plate_stats_centered(cx, stats_y_top, &stat_lines, stats_font);

        const LABELS: [&str; 2] = ["PLAY AGAIN", "MAIN MENU"];
        for (i, label) in LABELS.iter().enumerate() {
            let by = first_button_y + i as f32 * (button_h + spacing);
            let clicked = self.menu_click_index == Some(i) && self.menu_click_timer > 0.0;
            self.draw_menu_button(
                label,
                btn_x,
                by,
                button_w,
                button_h,
                i == self.win_menu_selection,
                clicked,
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