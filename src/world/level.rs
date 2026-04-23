// Dungeon Diver - Level & Tile System
use macroquad::prelude::*;
use crate::config::*;
use crate::world::tiled_visual::{TiledVisualMap, TiledVisualRaw};
use crate::world::{Chest, Enemy, EnemyKind, Item, ItemsAtlas, ItemType, TerrainAtlas, Torch, TorchDir, Vase};

/// Screen-space rect for the door leaf overlay: bottom aligns with the bottom of the door anchor cell
/// (see TMX `door` object; same framing as [`Level::draw_exit_door_unlock_overlay`]).
fn compute_door_leaf_screen_rect(
    door_x: i32,
    door_y: i32,
    camera_x: f32,
    camera_y: f32,
) -> (f32, f32, f32, f32) {
    let sx = door_x as f32 * TILE_SIZE - camera_x;
    let sy = door_y as f32 * TILE_SIZE - camera_y;
    let door_h = TILE_SIZE * (DUNGEON_TILESET_II_DOOR_LEAF_PX / DUNGEON_TILESET_II_CELL_PX);
    let door_w = EXIT_DOOR_LEAF_TILE_SPAN_W as f32 * TILE_SIZE;
    let top = sy + TILE_SIZE - door_h;
    (sx, top, door_w, door_h)
}

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

/// Gate driven by game events (key, button). Visuals crossfade `doors_leaf_closed` → `doors_leaf_open`
/// from dungeon_tileset_ii, not Tiled’s automatic tile animation clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateState {
    Closed,
    /// Elapsed time in the opening animation (seconds).
    Opening(f32),
    Open,
}

pub struct GateEntity {
    pub grid_x: i32,
    pub grid_y: i32,
    pub state: GateState,
}

impl GateEntity {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x,
            grid_y,
            state: GateState::Closed,
        }
    }

    pub fn blocks_movement(&self) -> bool {
        !matches!(self.state, GateState::Open)
    }

    pub fn begin_open(&mut self) {
        if matches!(self.state, GateState::Closed) {
            self.state = GateState::Opening(0.0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let GateState::Opening(t) = self.state {
            let nt = t + dt;
            if nt >= crate::config::GATE_OPEN_ANIM_DURATION {
                self.state = GateState::Open;
            } else {
                self.state = GateState::Opening(nt);
            }
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, atlas: Option<&ItemsAtlas>) {
        let sx = self.grid_x as f32 * TILE_SIZE - camera_x;
        let sy = self.grid_y as f32 * TILE_SIZE - camera_y;

        let Some(a) = atlas else {
            let dest = TILE_SIZE * (crate::config::DUNGEON_TILESET_II_DOOR_LEAF_PX / crate::config::DUNGEON_TILESET_II_CELL_PX);
            let dy = sy + TILE_SIZE - dest;
            let c = if matches!(self.state, GateState::Open) {
                Color::from_rgba(90, 140, 120, 220)
            } else {
                Color::from_rgba(125, 145, 170, 245)
            };
            draw_rectangle(sx, dy, dest, dest, c);
            draw_rectangle_lines(sx, dy, dest, dest, 2.0, Color::from_rgba(35, 45, 55, 255));
            return;
        };

        let t_open: f32 = match self.state {
            GateState::Closed => 0.0,
            GateState::Opening(t) => (t / crate::config::GATE_OPEN_ANIM_DURATION).clamp(0.0, 1.0),
            GateState::Open => 1.0,
        };
        // PNGs are 32×32; Tiled uses 16×16 cells → 2×2 game tiles footprint, bottom-aligned.
        let cell_px = crate::config::DUNGEON_TILESET_II_CELL_PX;
        let door_px = crate::config::DUNGEON_TILESET_II_DOOR_LEAF_PX;
        let dest = TILE_SIZE * (door_px / cell_px);
        let draw_y = sy + TILE_SIZE - dest;
        let params = DrawTextureParams {
            dest_size: Some(vec2(dest, dest)),
            ..Default::default()
        };
        draw_texture_ex(
            &a.gate_closed_tex,
            sx,
            draw_y,
            Color::new(1.0, 1.0, 1.0, 1.0 - t_open),
            params.clone(),
        );
        draw_texture_ex(
            &a.gate_open_tex,
            sx,
            draw_y,
            Color::new(1.0, 1.0, 1.0, t_open),
            params,
        );
    }
}

/// Floor switch: stepping on the tile once triggers `Level::open_all_gates()` (same as key).
#[derive(Debug, Clone)]
pub struct FloorButton {
    pub grid_x: i32,
    pub grid_y: i32,
    pub triggered: bool,
}

impl FloorButton {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x,
            grid_y,
            triggered: false,
        }
    }
}

pub struct Level {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub spawn_x: i32,
    pub spawn_y: i32,
    /// Doorway grid rect (TMX `door` object): `Tile::Door` conversion, open overlay, skip closed leaf art.
    pub door_x: i32,
    pub door_y: i32,
    pub door_w: i32,
    pub door_h: i32,
    /// Level-exit trigger rect (TMX `exit` object): `Player::at_exit`, exit marker, level complete.
    pub exit_x: i32,
    pub exit_y: i32,
    pub exit_w: i32,
    pub exit_h: i32,
    pub palette: Palette,
    pub items: Vec<Item>,
    pub vases: Vec<Vase>,
    pub torches: Vec<Torch>,
    pub chests: Vec<Chest>,
    pub gates: Vec<GateEntity>,
    pub buttons: Vec<FloorButton>,
    pub enemies: Vec<Enemy>,
    pub door_unlocked: bool,
    /// Raw GID grid extracted from the TMX map. Populated by `tmx_loader`;
    /// consumed by `Game::load_tiled_textures` to build `tiled_visual`.
    pub tiled_visual_raw: Option<TiledVisualRaw>,
    /// GPU-ready Tiled visual map. When `Some`, `draw()` renders Tiled layers
    /// instead of the generic `TerrainAtlas`.
    pub tiled_visual: Option<TiledVisualMap>,
}

impl Level {
    pub fn new(width: usize, height: usize, palette: Palette) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![Tile::Floor; width]; height],
            spawn_x: 1,
            spawn_y: 1,
            door_x: width as i32 - 2,
            door_y: 1,
            door_w: 1,
            door_h: 1,
            exit_x: width as i32 - 2,
            exit_y: 1,
            exit_w: 1,
            exit_h: 1,
            palette,
            items: Vec::new(),
            vases: Vec::new(),
            torches: Vec::new(),
            chests: Vec::new(),
            gates: Vec::new(),
            buttons: Vec::new(),
            enemies: Vec::new(),
            door_unlocked: false,
            tiled_visual_raw: None,
            tiled_visual: None,
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
                            let ex = x as i32;
                            let ey = y as i32;
                            self.door_x = ex;
                            self.door_y = ey;
                            self.door_w = 1;
                            self.door_h = 1;
                            self.exit_x = ex;
                            self.exit_y = ey;
                            self.exit_w = 1;
                            self.exit_h = 1;
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
                            self.vases.push(Vase::new(x as i32, y as i32, false));
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

    pub fn load_level_1_tmx() -> Self {
        match crate::world::tmx_loader::load_level_from_tmx("assets/levels/level1.tmx") {
            Ok(level) => level,
            Err(err) => {
                error!("L1 TMX load failed: {}", err);
                // Keep startup resilient even if the TMX file is missing/corrupt.
                let mut fallback = Self::new(LEVEL1_W, LEVEL1_H, LEVEL1_PALETTE);
                for y in 0..LEVEL1_H as i32 {
                    for x in 0..LEVEL1_W as i32 {
                        if x == 0 || y == 0 || x == LEVEL1_W as i32 - 1 || y == LEVEL1_H as i32 - 1 {
                            fallback.set_tile(x, y, Tile::SolidWall);
                        } else {
                            fallback.set_tile(x, y, Tile::Floor);
                        }
                    }
                }
                fallback.set_tile(LEVEL1_W as i32 - 2, 1, Tile::Door);
                fallback.spawn_x = 1;
                fallback.spawn_y = 1;
                let dx = LEVEL1_W as i32 - 2;
                let dy = 1;
                fallback.door_x = dx;
                fallback.door_y = dy;
                fallback.door_w = 1;
                fallback.door_h = 1;
                fallback.exit_x = dx;
                fallback.exit_y = dy;
                fallback.exit_w = 1;
                fallback.exit_h = 1;
                fallback
            }
        }
    }

    pub fn load_level_1() -> Self {
        Self::load_level_1_tmx()
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
        if self.gates.iter().any(|g| g.grid_x == x && g.grid_y == y && g.blocks_movement()) {
            return false;
        }
        let t = self.tiles[y as usize][x as usize];
        if t == Tile::Door && !self.door_unlocked {
            return false;
        }
        if self
            .vases
            .iter()
            .any(|v| !v.broken && v.grid_x == x && v.grid_y == y)
        {
            return false;
        }
        !t.is_solid()
    }

    pub fn update(&mut self, dt: f32) {
        for gate in &mut self.gates {
            gate.update(dt);
        }
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
        if let Some(ref tiled) = self.tiled_visual {
            // ─── Tiled visual path ────────────────────────────────────────────────
            // Render all Tiled tile layers (floor, walls, decoration) from the TMX
            // map. Then draw gameplay-logic overlays on top.
            let broken_tiled_vases: Vec<(i32, i32)> = self
                .vases
                .iter()
                .filter(|v| v.tiled_sprite && v.broken)
                .map(|v| (v.grid_x, v.grid_y))
                .collect();
            tiled.draw(camera_x, camera_y, &broken_tiled_vases);
            // Object-layer `torch` entities (and legacy torches not on a tile layer): not part of Tiled cells.
            if let Some(atlas) = items_atlas {
                for torch in &self.torches {
                    torch.draw_with_atlas(camera_x, camera_y, atlas);
                }
            } else {
                for torch in &self.torches {
                    torch.draw(camera_x, camera_y);
                }
            }
            // Exit marker / glow: drawn after foreground door art in `Game::draw_playing`
            // (`draw_exit_tile_marker`) so it does not sit under transparent door pixels.
        } else {
            // ─── Fallback / generic path ──────────────────────────────────────────
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
                        Tile::SolidWall | Tile::BottomCap | Tile::LeftFace | Tile::RightFace
                        | Tile::SolidWallRight | Tile::SolidWallLeft
                        | Tile::BottomCapRight | Tile::BottomCapLeft
                        | Tile::SolidWallBottom | Tile::SolidWallTop => {
                            if let Some(ref t) = terrain {
                                t.draw_wall(tile.sprite_type(), screen_x, screen_y);
                            } else {
                                draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, self.palette.wall_top);
                            }
                        }
                        Tile::Floor => {
                            if let Some(ref t) = terrain {
                                t.draw_floor(x, y, screen_x, screen_y);
                            } else {
                                let color = if (x + y) % 2 == 0 { self.palette.floor } else { self.palette.floor_alt };
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

            // Torches (behind items).
            if let Some(atlas) = items_atlas {
                for torch in &self.torches {
                    torch.draw_with_atlas(camera_x, camera_y, atlas);
                }
            } else {
                for torch in &self.torches {
                    torch.draw(camera_x, camera_y);
                }
            }
        }

        // Vases: ASCII draws placeholder; TMX `vase_shine_anim` draws intact in Tiled — only shards / placeholder here.
        for vase in &self.vases {
            vase.draw(camera_x, camera_y);
        }

        // Gates always render from gameplay state (independent from TMX visuals).
        for gate in &self.gates {
            gate.draw(camera_x, camera_y, items_atlas);
        }

        // ─── Always draw interactive gameplay entities ─────────────────────────
        // These are drawn regardless of Tiled visual mode.

        // Chests.
        if let Some(atlas) = items_atlas {
            for chest in &self.chests {
                chest.draw(camera_x, camera_y, atlas);
            }
        }

        // Items.
        for item in &self.items {
            item.draw(camera_x, camera_y, items_atlas);
        }
    }

    pub fn draw_foreground_before_player(
        &self,
        camera_x: f32,
        camera_y: f32,
        player_grid_x: i32,
        player_grid_y: i32,
    ) {
        if let Some(ref tiled) = self.tiled_visual {
            let broken_tiled_vases: Vec<(i32, i32)> = self
                .vases
                .iter()
                .filter(|v| v.tiled_sprite && v.broken)
                .map(|v| (v.grid_x, v.grid_y))
                .collect();
            tiled.draw_foreground_before_player(
                camera_x,
                camera_y,
                player_grid_x,
                player_grid_y,
                self.door_unlocked,
                self.door_x,
                self.door_y,
                &broken_tiled_vases,
            );
        }
    }

    pub fn draw_foreground_after_player(
        &self,
        camera_x: f32,
        camera_y: f32,
        player_grid_x: i32,
        player_grid_y: i32,
    ) {
        if let Some(ref tiled) = self.tiled_visual {
            let broken_tiled_vases: Vec<(i32, i32)> = self
                .vases
                .iter()
                .filter(|v| v.tiled_sprite && v.broken)
                .map(|v| (v.grid_x, v.grid_y))
                .collect();
            tiled.draw_foreground_after_player(
                camera_x,
                camera_y,
                player_grid_x,
                player_grid_y,
                self.door_unlocked,
                self.door_x,
                self.door_y,
                &broken_tiled_vases,
            );
        }
    }

    /// Wall sconces on non-`torches` layers (paths containing `decoration/torch/`) so tall `column` art
    /// does not cover them; see [`TiledVisualMap::draw_sconce_overlay`].
    pub fn draw_tiled_sconce_overlay(&self, camera_x: f32, camera_y: f32) {
        if let Some(ref tiled) = self.tiled_visual {
            let broken_tiled_vases: Vec<(i32, i32)> = self
                .vases
                .iter()
                .filter(|v| v.tiled_sprite && v.broken)
                .map(|v| (v.grid_x, v.grid_y))
                .collect();
            tiled.draw_sconce_overlay(camera_x, camera_y, &broken_tiled_vases);
        }
    }

    /// Draw the open door leaf after Tiled layers when unlocked (anchored to the `door` object).
    /// TMX tiles stay on the closed art; this overlay matches `door_unlocked`.
    pub fn draw_exit_door_unlock_overlay(&self, camera_x: f32, camera_y: f32, atlas: Option<&ItemsAtlas>) {
        if self.tiled_visual.is_none() || !self.door_unlocked {
            return;
        }
        let Some(a) = atlas else { return };
        let (sx, top, door_w, door_h) =
            compute_door_leaf_screen_rect(self.door_x, self.door_y, camera_x, camera_y);
        let dest = door_h.min(door_w);
        draw_texture_ex(
            &a.gate_open_tex,
            sx,
            top,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest, dest)),
                ..Default::default()
            },
        );
    }

    /// `(screen_x, screen_y, width, height)` for the door leaf overlay (anchored to `door`).
    pub fn door_leaf_screen_rect(&self, camera_x: f32, camera_y: f32) -> (f32, f32, f32, f32) {
        compute_door_leaf_screen_rect(self.door_x, self.door_y, camera_x, camera_y)
    }

    /// Full exit trigger area in screen space (for markers / level-complete pulse).
    pub fn exit_zone_screen_rect(&self, camera_x: f32, camera_y: f32) -> (f32, f32, f32, f32) {
        let sx = self.exit_x as f32 * TILE_SIZE - camera_x;
        let sy = self.exit_y as f32 * TILE_SIZE - camera_y;
        let w = self.exit_w as f32 * TILE_SIZE;
        let h = self.exit_h as f32 * TILE_SIZE;
        (sx, sy, w, h)
    }

    /// Exit hint / glow, drawn after door sprites so semi-transparent door pixels are not tinted
    /// by a rectangle drawn underneath.
    pub fn draw_exit_tile_marker(&self, camera_x: f32, camera_y: f32) {
        if self.tiled_visual.is_none() {
            return;
        }
        let (sx, sy, zw, zh) = self.exit_zone_screen_rect(camera_x, camera_y);

        if self.door_unlocked {
            let glow_time = (get_time() * 2.0) as f32;
            let glow_alpha = 0.22 + glow_time.sin() * 0.10;
            draw_rectangle(
                sx - 3.0,
                sy - 3.0,
                zw + 6.0,
                zh + 6.0,
                Color { r: 0.35, g: 0.88, b: 1.0, a: glow_alpha },
            );
        } else {
            let pulse_t = (get_time() as f32 * 1.5).sin() * 0.5 + 0.5;
            let marker_a = 0.35 + pulse_t * 0.25;
            let inset = 4.0_f32;
            draw_rectangle_lines(
                sx + inset,
                sy + inset,
                zw - 2.0 * inset,
                zh - 2.0 * inset,
                3.0,
                Color { r: 0.45, g: 0.72, b: 1.0, a: marker_a },
            );
        }
    }

    pub fn open_all_gates(&mut self) {
        for gate in &mut self.gates {
            gate.begin_open();
        }
    }

    pub fn pixel_to_grid(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / TILE_SIZE).floor() as i32, (y / TILE_SIZE).floor() as i32)
    }

    pub fn grid_to_pixel(&self, x: i32, y: i32) -> (f32, f32) {
        (x as f32 * TILE_SIZE + TILE_SIZE / 2.0, y as f32 * TILE_SIZE + TILE_SIZE / 2.0)
    }
}