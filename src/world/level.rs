// Dungeon Diver - Level & Tile System
use macroquad::prelude::*;
use crate::config::*;
use crate::world::{Chest, Enemy, EnemyKind, Item, ItemsAtlas, ItemType, TerrainAtlas, Torch, TorchDir, Vase};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Door,
    SolidWall,
    BottomCap,
    LeftFace,
    RightFace,
    SolidWallRight,
    SolidWallLeft,
    BottomCapRight,
    BottomCapLeft,
    SolidWallBottom,
    SolidWallTop,
    Spikes,
    Pit,
    Hazard,
    Water,
}

impl Tile {
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
                | Tile::Pit
                | Tile::Hazard
        )
    }

    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Door | Tile::Spikes)
    }

    pub fn sprite_type(&self) -> WallSprite {
        match self {
            Tile::Floor | Tile::Spikes | Tile::Pit | Tile::Hazard | Tile::Water => WallSprite::Floor,
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
            Tile::SolidWallTop => WallSprite::TopMid,
        }
    }
}

pub enum WallSprite {
    Floor,
    Door,
    Mid,
    TopMid,
    TopLeft,
    TopRight,
    Left,
    Right,
    BottomMid,
}

pub struct Level {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub exit_x: i32,
    pub exit_y: i32,
    pub palette: Palette,
    pub items: Vec<Item>,
    pub vases: Vec<Vase>,
    pub torches: Vec<Torch>,
    pub chests: Vec<Chest>,
    pub enemies: Vec<Enemy>,
    pub door_unlocked: bool,
}

impl Level {
    pub fn new(width: usize, height: usize, palette: Palette) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![Tile::Floor; width]; height],
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

    fn parse_layout(&mut self, layout: &[&str]) {
        use Tile::*;
        for (y, row) in layout.iter().enumerate() {
            if y >= self.height {
                break;
            }
            let chars: Vec<char> = row.chars().collect();
            let mut i = 0usize;
            let mut x = 0usize;
            while i < chars.len() && x < self.width {
                let ch = chars[i];
                let next = chars.get(i + 1).copied();
                let next2 = chars.get(i + 2).copied();
                let (tile, consumed) = match (ch, next, next2) {
                    ('S', Some('S'), Some('P')) => {
                        self.items.push(Item::new(x as i32, y as i32, ItemType::ShieldPotion));
                        (Floor, 3)
                    }
                    ('B', Some('S'), Some('P')) => {
                        self.items.push(Item::new(x as i32, y as i32, ItemType::BigShieldPotion));
                        (Floor, 3)
                    }
                    ('#', Some('|'), _) => (SolidWallRight, 2),
                    ('|', Some('#'), _) => (SolidWallLeft, 2),
                    ('#', Some('-'), _) => (SolidWallBottom, 2),
                    ('#', Some('+'), _) => (SolidWallTop, 2),
                    ('_', Some('|'), _) => (BottomCapRight, 2),
                    ('|', Some('_'), _) => (BottomCapLeft, 2),
                    ('B', Some('P'), _) => {
                        self.items.push(Item::new(x as i32, y as i32, ItemType::BigPotion));
                        (Floor, 2)
                    }
                    ('S', Some('P'), _) => {
                        self.items.push(Item::new(x as i32, y as i32, ItemType::SmallPotion));
                        (Floor, 2)
                    }
                    ('L', Some('T'), _) => {
                        self.torches.push(Torch::with_direction(x as i32, y as i32, TorchDir::Left));
                        (Floor, 2)
                    }
                    ('R', Some('T'), _) => {
                        self.torches.push(Torch::with_direction(x as i32, y as i32, TorchDir::Right));
                        (Floor, 2)
                    }
                    _ => match ch {
                        '#' => (SolidWall, 1),
                        '_' => (BottomCap, 1),
                        '|' => (RightFace, 1),
                        '.' => (Floor, 1),
                        '^' => (Spikes, 1),
                        'O' => (Pit, 1),
                        '@' => {
                            self.spawn_x = x as i32;
                            self.spawn_y = y as i32;
                            (Floor, 1)
                        }
                        'E' => {
                            self.exit_x = x as i32;
                            self.exit_y = y as i32;
                            (Door, 1)
                        }
                        'C' => {
                            self.items.push(Item::new(x as i32, y as i32, ItemType::Coin));
                            (Floor, 1)
                        }
                        'B' => {
                            self.items.push(Item::new(x as i32, y as i32, ItemType::BlueCoin));
                            (Floor, 1)
                        }
                        'G' => {
                            self.items.push(Item::new(x as i32, y as i32, ItemType::CoinBag));
                            (Floor, 1)
                        }
                        'P' => {
                            self.items.push(Item::new(x as i32, y as i32, ItemType::Potion));
                            (Floor, 1)
                        }
                        'V' => {
                            let contents = if (x + y) % 3 == 0 { Some(ItemType::Coin) } else { None };
                            self.vases.push(Vase::new(x as i32, y as i32, contents));
                            (Floor, 1)
                        }
                        'T' => {
                            self.torches.push(Torch::with_direction(x as i32, y as i32, TorchDir::Top));
                            (Floor, 1)
                        }
                        'H' => {
                            self.chests.push(Chest::new(x as i32, y as i32));
                            (Floor, 1)
                        }
                        'Z' => {
                            self.enemies.push(Enemy::new_with_kind(x as i32, y as i32, EnemyKind::Zombie));
                            (Floor, 1)
                        }
                        'W' => {
                            self.enemies.push(Enemy::new_with_kind(x as i32, y as i32, EnemyKind::BigZombie));
                            (Floor, 1)
                        }
                        'D' => {
                            self.enemies.push(Enemy::new_with_kind(x as i32, y as i32, EnemyKind::BigDemon));
                            (Floor, 1)
                        }
                        _ => (Floor, 1),
                    },
                };
                self.tiles[y][x] = tile;
                i += consumed;
                x += 1;
            }
        }
    }

    pub fn load_level_1() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",
            "#LT@...........RT#",
            "#.........BP...E#",
            "#......C......RT#",
            "#.....##-#-#-#.G..#",
            "#.....#-.H.#....#",
            "#.....T...#-#-#-#-#-#",
            "#.B....C....SP..#",
            "#.G........B...#",
            "#...C.......C..#",
            "#.........C...RT#",
            "#LT.............#",
            "#.......C......#",
            "#..C...........#",
            "#..........G...#",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_2() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",
            "#..............#",
            "#..####.####...#",
            "#..#Z#...H.#...E#",
            "#..####......###",
            "#....####..#...#",
            "#..............#",
            "#..####...#....#",
            "#@.....Z..#....#",
            "#..####...####.#",
            "#..............#",
            "#.......Z......#",
            "#......#####...#",
            "#..............#",
            "#...C......BP..#",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_3() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "##-#-#-#-#-#-#-#-#-#-#-#-#-#-#",
            "#.......BP...E..#",
            "#.##-##-.#-#-#-#-#-#-.##",
            "#.#Z#.........##",
            "#.###.SP.......##",
            "#.............#-#",
            "#..##-#-#-#-#-......#",
            "#..#Z.G...Z....#",
            "#..#.......H...#",
            "#..######......#",
            "#..............#",
            "#.##-#-#.........#",
            "#.#..#.....Z...#",
            "#.#-C.#-.....SP...#",
            "#@.............#",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_4() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "################",
            "#E..^.^....C..Z#",
            "#.####.####.####",
            "#...^....SSP...#",
            "#.####.####..#.#",
            "#....^....#..#.#",
            "#.######..#..#.#",
            "#......#..#..#.#",
            "#..H...#..#..#.#",
            "#......#..#..#.#",
            "#.######..#..#.#",
            "#....^....#..#.#",
            "#.####.####..#.#",
            "#..C.....G....Z#",
            "#@......^......#",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_5() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "################",
            "#@....O....C...#",
            "#.###.O.######.#",
            "#...#...#....#.#",
            "###.#.O.#.Z..#.#",
            "#...#...#.##.#.#",
            "#.O.###.#....#.#",
            "#...#...#.####.#",
            "#.###.O.#..H...#",
            "#...#...#.####.#",
            "#.O.#.###....#.#",
            "#...#...#..Z.#.#",
            "#.###.O.###..#.#",
            "#..SSP..BSP..#E#",
            "#....C....Z...##",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_6() -> Self {
        let mut level = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
        let layout = [
            "################",
            "#@....^..W....E#",
            "#.####O####O##.#",
            "#....^.....^...#",
            "#.##.#######.###",
            "#..Z....H.....W#",
            "#.##.#######.###",
            "#....^.....^...#",
            "#.####O####O##.#",
            "#..BSP....SSP..#",
            "#.##.#######.###",
            "#..Z.......Z...#",
            "#.##.#######.###",
            "#..C....P....G.#",
            "#....^....SP...#",
            "################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_7() -> Self {
        let mut level = Self::new(LEVEL_LARGE_W, LEVEL_LARGE_H, LEVEL1_PALETTE);
        let layout = [
            "####################",
            "#E......#....C.....#",
            "#.####..#.######.###",
            "#....#..#......#...#",
            "###..#..####O..###.#",
            "#....#....W.#......#",
            "#.######.##.#.###..#",
            "#..SSP...##.#...#..#",
            "#.######....###.#..#",
            "#.....##.H....#.#..#",
            "#.###.##.####.#.#..#",
            "#...#....#....#.#..#",
            "###.#.####.####.##.#",
            "#...#...Z...^....#.#",
            "#.#####.#####.##.#.#",
            "#..P....W....BP..#.#",
            "#.#####.#####.##.#.#",
            "#....Z......C....#.#",
            "#@...............D.#",
            "####################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_8() -> Self {
        let mut level = Self::new(LEVEL_LARGE_W, LEVEL_LARGE_H, LEVEL1_PALETTE);
        let layout = [
            "####################",
            "#.........E........#",
            "#.^^^^.######.^^^^.#",
            "#.^..^.##WW##.^..^.#",
            "#.^..^.#....#.^..^.#",
            "#.^^^^.#.H..#.^^^^.#",
            "#......#.DD.#......#",
            "####.###....###.####",
            "#....#..Z..Z..#....#",
            "#.C..#..W..W..#..C.#",
            "#....#..Z..D..#....#",
            "####.###....###.####",
            "#......#.BP........#",
            "#.^^^^.#....#.^^^^.#",
            "#.^..^.#....#.^..^.#",
            "#.^..^.##..##.^..^.#",
            "#.^^^^.######.^^^^.#",
            "#....SSP....G....P.#",
            "#.........@........#",
            "####################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_9() -> Self {
        let mut level = Self::new(LEVEL_LARGE_W, LEVEL_LARGE_H, LEVEL1_PALETTE);
        let layout = [
            "####################",
            "#@.................#",
            "#.################.#",
            "#......^....W....#.#",
            "#.#.###########..#.#",
            "#.#.#.........#..#.#",
            "#.#.#.#######.#..#.#",
            "#.#.#.#..D..#.#..#.#",
            "#.#.#.#.###.#.#..#.#",
            "#.#.#.#.#H..#.#..#.#",
            "#.#.#.#.###.#.#..#.#",
            "#.#.#....W..#.#..#.#",
            "#.#.#.#######.#..#.#",
            "#.#.#....O.......#.#",
            "#.#.###########..#.#",
            "#.#....^....D....#.#",
            "#.################.#",
            "#..BSP...G..SSP...#",
            "#...............E.#",
            "####################",
        ];
        level.parse_layout(&layout);
        level
    }

    pub fn load_level_10() -> Self {
        let mut level = Self::new(LEVEL_LARGE_W, LEVEL_LARGE_H, LEVEL1_PALETTE);
        let layout = [
            "####################",
            "#...BP....E.........#",
            "#.######.######.##.#",
            "#.#..D.#.^..^.#..##",
            "#.#.##BP#.####.#.##.#",
            "#.#.##.#BSP..BSP#.#.##.#",
            "#.#H...####.#.#....#",
            "#.####.#OO#.#.####.#",
            "#....#.#OO#.#.#....#",
            "####.#.####.#.#.####",
            "#....#.GWWG.#.#....#",
            "#.####.####.#.####.#",
            "#.#..Z..P...#....#.#",
            "#.#.###########.#.#.",
            "#.#.....BSP.....#.#.",
            "#.###.#######.###.#.",
            "#...#..D...D..#...#.",
            "#.P.#..W...W..#.G.#.",
            "#@..#..Z...D..#SSP#.",
            "####################",
        ];
        level.parse_layout(&layout);
        level
    }


    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Tile::SolidWall;
        }
        self.tiles[y as usize][x as usize]
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.tiles[y as usize][x as usize] = tile;
        }
    }

    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return false;
        }
        !self.tiles[y as usize][x as usize].is_solid()
    }

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

    pub fn draw(
        &self,
        camera_x: f32,
        camera_y: f32,
        terrain: Option<&TerrainAtlas>,
        items_atlas: Option<&ItemsAtlas>,
        spike_timer: f32,
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
                    Tile::Spikes => {
                        if let Some(ref t) = terrain {
                            let offset = ((x * 7 + y * 13) % 4) as f32 * 0.5;
                            let local_t = spike_timer + offset;
                            let cycle = 3.0_f32;
                            let phase_t = local_t % cycle;
                            let frame = if phase_t < 1.0 {
                                ((phase_t * 4.0) as usize).min(3)
                            } else {
                                3 - ((((phase_t - 1.0) / 2.0) * 4.0) as usize).min(3)
                            };
                            t.draw_spikes(screen_x, screen_y, frame);
                        } else {
                            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, Color::from_rgba(90, 50, 50, 255));
                        }
                    }
                    Tile::Pit => {
                        if let Some(ref t) = terrain {
                            t.draw_pit(screen_x, screen_y);
                        } else {
                            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, Color::from_rgba(10, 10, 15, 255));
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

    pub fn pixel_to_grid(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / TILE_SIZE).floor() as i32, (y / TILE_SIZE).floor() as i32)
    }

    pub fn grid_to_pixel(&self, x: i32, y: i32) -> (f32, f32) {
        (x as f32 * TILE_SIZE + TILE_SIZE / 2.0, y as f32 * TILE_SIZE + TILE_SIZE / 2.0)
    }
}