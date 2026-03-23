// Dungeon Diver - Level & Tile System
// Grid-based tile map - tile size is configurable via TILE_SIZE constant
// Default: 64x64 pixels (set in src/constants.rs)

use macroquad::prelude::*;
use crate::constants::*;
use crate::items::{Chest, Item, ItemsAtlas, ItemType, Vase, Torch};
use crate::terrain::TerrainAtlas;
use crate::enemy::Enemy;

/// Tile types in the level
///
/// ═══════════════════════════════════════════════════════════════════════════════
/// TILE COMBINATION GRAMMAR
/// ═══════════════════════════════════════════════════════════════════════════════
/// Level layout uses 1-2 characters per cell to specify exact wall appearance:
///
/// SINGLE CHAR TILES:
///   .  = Floor
///   @  = Spawn (becomes Floor)
///   E  = Exit door
///   #  = SolidWall (solid brick on all sides)
///
/// DOUBLE CHAR TILES (wall combinations, order matters):
///   _  = BottomCap  (horizontal top/bottom face)
///   |  = LeftFace   (vertical side face, shows left side)
///   |  = RightFace  (vertical side face, shows right side)
///
///   #| = SolidWallRight (solid + right side visible)
///   |# = SolidWallLeft  (solid + left side visible)
///   _| = BottomCapRight (bottom + right side)
///   |_ = BottomCapLeft  (bottom + left side)
///
/// ═══════════════════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Door,

    // ═════════════════════════════════════════════════════════════════════════
    // EXPLICIT WALL TYPES - no autotiling needed
    // ═════════════════════════════════════════════════════════════════════════
    /// Solid brick on all sides - no transparency
    SolidWall,
    /// Bottom/top cap - horizontal face visible (stone top)
    BottomCap,
    /// Vertical side face showing LEFT side
    LeftFace,
    /// Vertical side face showing RIGHT side
    RightFace,
    /// Solid wall + right side visible
    SolidWallRight,
    /// Solid wall + left side visible
    SolidWallLeft,
    /// Bottom cap + right side
    BottomCapRight,
    /// Bottom cap + left side
    BottomCapLeft,
    /// Solid wall + bottom cap (the #- combination)
    SolidWallBottom,
    /// Solid wall + top cap (the #+ combination)
    SolidWallTop,

    // Future types
    Hazard,
    Water,
}

impl Tile {
    /// Check if this tile blocks movement
    pub fn is_solid(&self) -> bool {
        matches!(
            self,
            Tile::SolidWall
                | Tile::BottomCap
                | Tile::LeftFace
                | Tile::RightFace
                | Tile::SolidWallRight
                | Tile::SolidWallLeft
                | Tile::BottomCapRight
                | Tile::BottomCapLeft
                | Tile::SolidWallBottom
                | Tile::SolidWallTop
                | Tile::Hazard
        )
    }

    /// Check if this tile is walkable
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Door)
    }

    /// Get the sprite type for rendering (bypasses autotiling)
    pub fn sprite_type(&self) -> WallSprite {
        match self {
            Tile::Floor => WallSprite::Floor,
            Tile::Door => WallSprite::Door,
            Tile::SolidWall => WallSprite::Mid,
            Tile::BottomCap => WallSprite::TopMid,
            Tile::LeftFace => WallSprite::Left,
            Tile::RightFace => WallSprite::Right,
            Tile::SolidWallRight => WallSprite::TopRight,
            Tile::SolidWallLeft => WallSprite::TopLeft,
            Tile::BottomCapRight => WallSprite::TopRight,
            Tile::BottomCapLeft => WallSprite::TopLeft,
            Tile::SolidWallBottom => WallSprite::BottomMid,
            Tile::Hazard | Tile::Water => WallSprite::Floor,
            Tile::SolidWallTop => WallSprite::TopMid,
        }
    }
}

/// Which wall sprite to use (direct mapping, no autotiling)
pub enum WallSprite {
    Floor,
    Door,
    Mid,         // wall_mid - solid brick at 0°
    TopMid,      // wall_top_mid at 0° - horizontal cap
    TopLeft,     // wall_top_mid at 0° + 270° - corner with left side
    TopRight,    // wall_top_mid at 0° + 90° - corner with right side
    Left,        // wall_mid + wall_top_mid at 270° - left side face
    Right,       // wall_mid + wall_top_mid at 90° - right side face
    BottomMid,   // wall_mid at 0° + wall_top_mid at 0° - solid + bottom cap (#- combo)
}

/// Level structure with tile grid
pub struct Level {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub exit_x: i32,
    pub exit_y: i32,
    pub palette: Palette,
    // Level objects
    pub items: Vec<Item>,
    pub vases: Vec<Vase>,
    pub torches: Vec<Torch>,
    pub chests: Vec<Chest>,
    pub enemies: Vec<Enemy>,
    /// Updated by chest/key logic; used by door rendering in later phases.
    pub door_unlocked: bool,
}

impl Level {
    /// Create a new empty level
    pub fn new(width: usize, height: usize, palette: Palette) -> Self {
        let tiles = vec![vec![Tile::Floor; width]; height];
        Self {
            width,
            height,
            tiles,
            spawn_x: 1,
            spawn_y: 1,
            exit_x: width as i32 - 2,
            exit_y: 1,
            palette,
            items: Vec::new(),
            vases: Vec::new(),
            torches: Vec::new(),
            chests: Vec::new(),
            enemies: Vec::new(),
            door_unlocked: false,
        }
    }

    /// Load Level 1: Tutorial Dungeon
    /// 12x12 tiles - compact level for testing with larger tiles
    pub fn load_level_1() -> Self {
        use Tile::*;

        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);

        // ═══════════════════════════════════════════════════════════════════════════════
        // TILE COMBINATION GRAMMAR - 1-2 characters per cell
        // ═══════════════════════════════════════════════════════════════════════════════
        // KEY COMBINATIONS (each row must be exactly 16 chars!)
        // ═══════════════════════════════════════════════════════════════════════════════
        //   #  = SolidWall      (just brick face)
        //   #| = SolidWallRight (brick + RIGHT side face, flip_x=true)
        //   |# = SolidWallLeft  (brick + LEFT side face, flip_x=false)
        //   #- = SolidWallBottom (brick + top cap, like your 3rd image)
        //   _  = BottomCap       (just top cap, lighter stone)
        // ═══════════════════════════════════════════════════════════════════════════════
        // Legend: @=spawn  E=exit  C=coin  B=blue coin  P=potion  V=vase  H=chest  G=coin bag
        // ═══════════════════════════════════════════════════════════════════════════════

        let layout = [
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
        ];

        // Parse the layout - each cell is 1-2 characters
        for (y, row) in layout.iter().enumerate() {
            if y >= level.height {
                break;
            }
            let mut x = 0;

            let chars: Vec<char> = row.chars().collect();
            let mut i = 0;

            while i < chars.len() && x < level.width {
                let ch = chars[i];
                let next_ch = if i + 1 < chars.len() { Some(chars[i + 1]) } else { None };

                // Check for 2-character combinations first (order matters!)
                let (tile_type, consumed) = match (ch, next_ch) {
                    // Wall combinations
                    ('#', Some('|')) => (SolidWallRight, 2),
                    ('|', Some('#')) => (SolidWallLeft, 2),
                    ('#', Some('-')) => (SolidWallBottom, 2),  // #- = solid + bottom cap
                    ('#', Some('+')) => (SolidWallTop, 2),  // #+ = solid + top cap
                    ('_', Some('|')) => (BottomCapRight, 2),
                    ('|', Some('_')) => (BottomCapLeft, 2),

                    // Item combinations (2-char codes)
                    ('B', Some('P')) => {
                        level.items.push(Item::new(x as i32, y as i32, ItemType::BigPotion));
                        (Floor, 2)
                    }
                    ('S', Some('P')) => {
                        level.items.push(Item::new(x as i32, y as i32, ItemType::SmallPotion));
                        (Floor, 2)
                    }
                    ('L', Some('T')) => {
                        level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Left));
                        (Floor, 2)
                    }
                    ('R', Some('T')) => {
                        level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Right));
                        (Floor, 2)
                    }

                    // Single char tiles (not part of a combo)
                    _ => {
                        match ch {
                            '#' => (SolidWall, 1),
                            '_' => (BottomCap, 1),
                            '|' => (RightFace, 1),  // default to right face for single |
                            '.' => (Floor, 1),
                            '@' => {
                                level.spawn_x = x as i32;
                                level.spawn_y = y as i32;
                                (Floor, 1)
                            }
                            'E' => {
                                level.exit_x = x as i32;
                                level.exit_y = y as i32;
                                (Door, 1)
                            }
                            'C' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::Coin));
                                (Floor, 1)
                            }
                            'B' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::BlueCoin));
                                (Floor, 1)
                            }
                            'G' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::CoinBag));
                                (Floor, 1)
                            }
                            'P' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::Potion));
                                (Floor, 1)
                            }
                            'V' => {
                                let contents = if (x + y) % 3 == 0 { Some(ItemType::Coin) } else { None };
                                level.vases.push(Vase::new(x as i32, y as i32, contents));
                                (Floor, 1)
                            }
                            'T' => {
                                level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Top));
                                (Floor, 1)
                            }
                            'H' => {
                                level.chests.push(Chest::new(x as i32, y as i32));
                                (Floor, 1)
                            }
                            'Z' => {
                                level.enemies.push(Enemy::new(x as i32, y as i32));
                                (Floor, 1)
                            }
                            _ => (Floor, 1),  // Unknown char → floor
                        }
                    }
                };

                level.tiles[y][x] = tile_type;
                x += 1;
                i += consumed;
            }
        }

        level
    }

    /// Load Level 2: Zombie Crypt
    /// 16x16 tiles - similar size to Level 1 but different layout
    /// Features: Walls in middle, different chest/door positions, zombie enemies
    pub fn load_level_2() -> Self {
        use Tile::*;

        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);

        // ═══════════════════════════════════════════════════════════════════════════════
        // LEVEL 2 LAYOUT - Zombie Crypt
        // ═══════════════════════════════════════════════════════════════════════════════
        // Key differences from Level 1:
        //   - Walls create a more maze-like layout in the middle
        //   - Chest is in a different location (bottom right area)
        //   - Exit is on the left side instead of right
        //   - Multiple zombie spawn points (Z)
        // ═══════════════════════════════════════════════════════════════════════════════
        // Legend: @=spawn  E=exit  C=coin  B=blue coin  P=potion  H=chest  Z=zombie
        // ═══════════════════════════════════════════════════════════════════════════════

        let layout = [
            "##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",  // row 0
            "#..............#",  // row 1
            "#..####.####...#",  // row 2 - walls in middle
            "#..#Z#...H.#...E#",  // row 3 - zombie, chest, exit
            "#..####......###",  // row 4
            "#....####..#...#",  // row 5
            "#..............#",  // row 6
            "#..####...#....#",  // row 7 - walls
            "#@.....Z..#....#",  // row 8 - spawn, zombie
            "#..####...####.#",  // row 9
            "#..............#",  // row 10
            "#.......Z......#",  // row 11 - zombie
            "#......#####...#",  // row 12 - wall section
            "#..............#",  // row 13
            "#...C......BP..#",  // row 14 - coin, big potion
            "################",  // row 15
        ];

        // Parse the layout - same parser as Level 1
        for (y, row) in layout.iter().enumerate() {
            if y >= level.height {
                break;
            }
            let mut x = 0;

            let chars: Vec<char> = row.chars().collect();
            let mut i = 0;

            while i < chars.len() && x < level.width {
                let ch = chars[i];
                let next_ch = if i + 1 < chars.len() { Some(chars[i + 1]) } else { None };

                // Check for 2-character combinations first (order matters!)
                let (tile_type, consumed) = match (ch, next_ch) {
                    // Wall combinations
                    ('#', Some('|')) => (SolidWallRight, 2),
                    ('|', Some('#')) => (SolidWallLeft, 2),
                    ('#', Some('-')) => (SolidWallBottom, 2),
                    ('#', Some('+')) => (SolidWallTop, 2),
                    ('_', Some('|')) => (BottomCapRight, 2),
                    ('|', Some('_')) => (BottomCapLeft, 2),

                    // Item combinations (2-char codes)
                    ('B', Some('P')) => {
                        level.items.push(Item::new(x as i32, y as i32, ItemType::BigPotion));
                        (Floor, 2)
                    }
                    ('S', Some('P')) => {
                        level.items.push(Item::new(x as i32, y as i32, ItemType::SmallPotion));
                        (Floor, 2)
                    }
                    ('L', Some('T')) => {
                        level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Left));
                        (Floor, 2)
                    }
                    ('R', Some('T')) => {
                        level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Right));
                        (Floor, 2)
                    }

                    // Single char tiles (not part of a combo)
                    _ => {
                        match ch {
                            '#' => (SolidWall, 1),
                            '_' => (BottomCap, 1),
                            '|' => (RightFace, 1),
                            '.' => (Floor, 1),
                            '@' => {
                                level.spawn_x = x as i32;
                                level.spawn_y = y as i32;
                                (Floor, 1)
                            }
                            'E' => {
                                level.exit_x = x as i32;
                                level.exit_y = y as i32;
                                (Door, 1)
                            }
                            'C' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::Coin));
                                (Floor, 1)
                            }
                            'B' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::BlueCoin));
                                (Floor, 1)
                            }
                            'G' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::CoinBag));
                                (Floor, 1)
                            }
                            'P' => {
                                level.items.push(Item::new(x as i32, y as i32, ItemType::Potion));
                                (Floor, 1)
                            }
                            'V' => {
                                let contents = if (x + y) % 3 == 0 { Some(ItemType::Coin) } else { None };
                                level.vases.push(Vase::new(x as i32, y as i32, contents));
                                (Floor, 1)
                            }
                            'T' => {
                                level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Top));
                                (Floor, 1)
                            }
                            'H' => {
                                level.chests.push(Chest::new(x as i32, y as i32));
                                (Floor, 1)
                            }
                            'Z' => {
                                level.enemies.push(Enemy::new(x as i32, y as i32));
                                (Floor, 1)
                            }
                            _ => (Floor, 1),
                        }
                    }
                };

                level.tiles[y][x] = tile_type;
                x += 1;
                i += consumed;
            }
        }

        level
    }


    /// Level 3: The Bone Pit
    /// Harder than Level 2 — 4 zombies, split corridors, exit top-left.
    /// Spawn: bottom-left (@). Exit: top-left (E).
    /// 16×16 tiles, all single-char layout.
    pub fn load_level_3() -> Self {
        use Tile::*;

        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);

        // ═══════════════════════════════════════════════════════════════════════════════
        // LEVEL 3 LAYOUT - The Bone Pit
        // ═══════════════════════════════════════════════════════════════════════════════
        // Key differences from Level 2:
        //   - 4 zombies (up from 3)
        //   - Exit is top-left, spawn is bottom-left (reverse traversal)
        //   - Central horizontal barrier splits level into upper/lower halves
        //   - Left side has a narrow corridor + guarded mini-room
        //   - Chest guarded by two flanking zombies (row 7)
        // ═══════════════════════════════════════════════════════════════════════════════
        // Legend: @=spawn  E=exit  C=coin  P=potion  H=chest  Z=zombie
        // ═══════════════════════════════════════════════════════════════════════════════

        let layout = [
            "##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",  // row  0  top border
            "#E......BP......#",  // row  1  exit top-left
            "#.##-##-.#-#-#-#-#-#-.##",  // row  2  thick wall forces movement right
            "#.#Z#.........##",  // row  3  guarded mini-room
            "#.###.SP.......##",  // row  4
            "#.............#-#",  // row  5  open corridor along right wall
            "#..##-#-#-#-#-......#",  // row  6  horizontal barrier (gap on right)
            "#..#Z.G...Z....#",  // row  7  two zombies flanking the chest corridor
            "#..#.......H...#",  // row  8  chest behind zombie line
            "#..######......#",  // row  9  mirror barrier closes the chamber
            "#..............#",  // row 10  open bottom section
            "#.##-#-#.........#",  // row 11  left side obstacle
            "#.#..#.....Z...#",  // row 12  zombie bottom-right
            "#.#-C.#-.....SP...#",  // row 13  coin + potion bottom reward
            "#@.............#",  // row 14  spawn bottom-left
            "################",  // row 15  bottom border
        ];

        // Parse the layout — same 2-char combo parser as Level 1/2
        for (y, row) in layout.iter().enumerate() {
            if y >= level.height { break; }
            let mut x = 0;
            let chars: Vec<char> = row.chars().collect();
            let mut i = 0;

            while i < chars.len() && x < level.width {
                let ch = chars[i];
                let next_ch = if i + 1 < chars.len() { Some(chars[i + 1]) } else { None };

                let (tile_type, consumed) = match (ch, next_ch) {
                    ('#', Some('|')) => (SolidWallRight, 2),
                    ('|', Some('#')) => (SolidWallLeft, 2),
                    ('#', Some('-')) => (SolidWallBottom, 2),
                    ('#', Some('+')) => (SolidWallTop, 2),
                    ('_', Some('|')) => (BottomCapRight, 2),
                    ('|', Some('_')) => (BottomCapLeft, 2),
                    ('B', Some('P')) => { level.items.push(Item::new(x as i32, y as i32, ItemType::BigPotion)); (Floor, 2) }
                    ('S', Some('P')) => { level.items.push(Item::new(x as i32, y as i32, ItemType::SmallPotion)); (Floor, 2) }
                    ('L', Some('T')) => { level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Left)); (Floor, 2) }
                    ('R', Some('T')) => { level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Right)); (Floor, 2) }
                    _ => match ch {
                        '#' => (SolidWall, 1),
                        '_' => (BottomCap, 1),
                        '|' => (RightFace, 1),
                        '.' => (Floor, 1),
                        '@' => { level.spawn_x = x as i32; level.spawn_y = y as i32; (Floor, 1) }
                        'E' => { level.exit_x = x as i32; level.exit_y = y as i32; (Door, 1) }
                        'C' => { level.items.push(Item::new(x as i32, y as i32, ItemType::Coin)); (Floor, 1) }
                        'B' => { level.items.push(Item::new(x as i32, y as i32, ItemType::BlueCoin)); (Floor, 1) }
                        'G' => { level.items.push(Item::new(x as i32, y as i32, ItemType::CoinBag)); (Floor, 1) }
                        'P' => { level.items.push(Item::new(x as i32, y as i32, ItemType::Potion)); (Floor, 1) }
                        'V' => { let c = if (x + y) % 3 == 0 { Some(ItemType::Coin) } else { None }; level.vases.push(Vase::new(x as i32, y as i32, c)); (Floor, 1) }
                        'T' => { level.torches.push(Torch::with_direction(x as i32, y as i32, crate::items::TorchDir::Top)); (Floor, 1) }
                        'H' => { level.chests.push(Chest::new(x as i32, y as i32)); (Floor, 1) }
                        'Z' => { level.enemies.push(Enemy::new(x as i32, y as i32)); (Floor, 1) }
                        _ => (Floor, 1),
                    },
                };

                level.tiles[y][x] = tile_type;
                x += 1;
                i += consumed;
            }
        }

        level
    }


    /// Get tile at grid position
    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Tile::SolidWall;
        }
        self.tiles[y as usize][x as usize]
    }

    /// Set tile at grid position
    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.tiles[y as usize][x as usize] = tile;
        }
    }

    /// Check if position is valid (within bounds and not a wall)
    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return false;
        }
        !self.tiles[y as usize][x as usize].is_solid()
    }

    /// Update all level objects (torches, items)
    pub fn update(&mut self, dt: f32) {
        for torch in &mut self.torches {
            torch.update(dt);
        }
        for item in &mut self.items {
            item.update(dt);
        }
        for chest in &mut self.chests {
            chest.update(dt);
        }
    }

    /// Draw the visible portion of the level.
    /// When `terrain` is `Some`, floors/walls use Gathering Set atlases (16→32 px scaling).
    /// When `items_atlas` is `Some`, chests/keys use 0x72 sprites.
    pub fn draw(
        &self,
        camera_x: f32,
        camera_y: f32,
        terrain: Option<&TerrainAtlas>,
        items_atlas: Option<&ItemsAtlas>,
    ) {
        let start_x = (camera_x / TILE_SIZE).floor() as i32 - 1;
        let start_y = (camera_y / TILE_SIZE).floor() as i32 - 1;
        let end_x = start_x + SCREEN_TILES_W + 2;
        let end_y = start_y + SCREEN_TILES_H + 2;

        for y in start_y..end_y {
            for x in start_x..end_x {
                if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                    continue;
                }

                let tile = self.tiles[y as usize][x as usize];
                let screen_x = x as f32 * TILE_SIZE - camera_x;
                let screen_y = y as f32 * TILE_SIZE - camera_y;

                match tile {
                    // ═══════════════════════════════════════════════════════════════════════
                    // WALL TILES - use explicit sprite type, no autotiling
                    // ═══════════════════════════════════════════════════════════════════════
                    Tile::SolidWall | Tile::BottomCap | Tile::LeftFace | Tile::RightFace
                    | Tile::SolidWallRight | Tile::SolidWallLeft
                    | Tile::BottomCapRight | Tile::BottomCapLeft
                    | Tile::SolidWallBottom | Tile::SolidWallTop => {
                        if let Some(ref t) = terrain {
                            t.draw_wall(tile.sprite_type(), screen_x, screen_y);
                        } else {
                            // Fallback: solid rectangle
                            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, self.palette.wall_top);
                        }
                    }
                    Tile::Floor => {
                        if let Some(ref t) = terrain {
                            t.draw_floor(x, y, screen_x, screen_y);
                        } else {
                            let color = if (x + y) % 2 == 0 {
                                self.palette.floor
                            } else {
                                self.palette.floor_alt
                            };
                            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, color);
                        }
                    }
                    Tile::Door => {
                        if let Some(ref t) = terrain {
                            t.draw_door(screen_x, screen_y, self.door_unlocked);
                        } else {
                            // Fallback: glowing blue rectangle
                            let glow_time = (get_time() * 2.0) as f32;
                            let glow_alpha = 0.3 + (glow_time).sin() * 0.15;
                            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, self.palette.accent);
                            draw_rectangle(
                                screen_x - 2.0, screen_y - 2.0,
                                TILE_SIZE + 4.0, TILE_SIZE + 4.0,
                                Color { r: 0.4, g: 0.8, b: 1.0, a: glow_alpha },
                            );
                        }
                    }
                    Tile::Hazard => {
                        draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, RED);
                    }
                    Tile::Water => {
                        draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, BLUE);
                    }
                }
            }
        }

        // Draw torches (behind items)
        if let Some(atlas) = items_atlas {
            for torch in &self.torches {
                torch.draw_with_atlas(camera_x, camera_y, atlas);
            }
        } else {
            for torch in &self.torches {
                torch.draw(camera_x, camera_y);
            }
        }

        // Draw vases
        for vase in &self.vases {
            vase.draw(camera_x, camera_y);
        }

        // Draw chests (interactive tiles)
        if let Some(atlas) = items_atlas {
            for chest in &self.chests {
                chest.draw(camera_x, camera_y, atlas);
            }
        }

        // Draw items
        for item in &self.items {
            item.draw(camera_x, camera_y, items_atlas);
        }
    }

    /// Convert pixel position to grid position
    pub fn pixel_to_grid(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / TILE_SIZE).floor() as i32, (y / TILE_SIZE).floor() as i32)
    }

    /// Convert grid position to pixel position (center of tile)
    pub fn grid_to_pixel(&self, x: i32, y: i32) -> (f32, f32) {
        (x as f32 * TILE_SIZE + TILE_SIZE / 2.0, y as f32 * TILE_SIZE + TILE_SIZE / 2.0)
    }
}