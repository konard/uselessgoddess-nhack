//! Map and dungeon generation.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for map systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<GameMap>()
        .register_type::<GameMap>()
        .register_type::<Room>();
}

/// Tile types for the dungeon map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Door,
    StairsDown,
    StairsUp,
}

impl TileType {
    pub fn glyph(&self) -> char {
        match self {
            TileType::Wall => '#',
            TileType::Floor => '.',
            TileType::Door => '+',
            TileType::StairsDown => '>',
            TileType::StairsUp => '<',
        }
    }

    pub fn is_walkable(&self) -> bool {
        matches!(
            self,
            TileType::Floor | TileType::Door | TileType::StairsDown | TileType::StairsUp
        )
    }

    pub fn blocks_sight(&self) -> bool {
        matches!(self, TileType::Wall | TileType::Door)
    }
}

/// A room in the dungeon.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub room_id: usize,
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32, room_id: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room_id,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }

    /// Check if a position is inside this room.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// The game map resource.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameMap {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Room>,
    pub current_floor: i32,
    pub revealed: Vec<bool>,
    pub visible: Vec<bool>,
}

impl Default for GameMap {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

impl GameMap {
    pub fn new(width: i32, height: i32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; size],
            rooms: Vec::new(),
            current_floor: 1,
            revealed: vec![false; size],
            visible: vec![false; size],
        }
    }

    /// Convert (x, y) to linear index.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Check if coordinates are within bounds.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Get tile at position.
    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        if self.in_bounds(x, y) {
            Some(self.tiles[self.xy_idx(x, y)])
        } else {
            None
        }
    }

    /// Set tile at position.
    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.tiles[idx] = tile;
        }
    }

    /// Check if a position is walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| t.is_walkable())
    }

    /// Generate a simple dungeon with rooms and corridors.
    pub fn generate_dungeon(&mut self, rng: &mut impl rand::Rng) {

        const MAX_ROOMS: usize = 10;
        const MIN_ROOM_SIZE: i32 = 4;
        const MAX_ROOM_SIZE: i32 = 10;

        self.rooms.clear();

        for room_id in 0..MAX_ROOMS {
            let w = rng.random_range(MIN_ROOM_SIZE..=MAX_ROOM_SIZE);
            let h = rng.random_range(MIN_ROOM_SIZE..=MAX_ROOM_SIZE);
            let x = rng.random_range(1..self.width - w - 1);
            let y = rng.random_range(1..self.height - h - 1);

            let new_room = Room::new(x, y, w, h, room_id);

            // Check for intersections with existing rooms
            let intersects = self.rooms.iter().any(|r| new_room.intersects(r));

            if !intersects {
                self.carve_room(&new_room);

                if !self.rooms.is_empty() {
                    // Connect to previous room
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms.last().unwrap().center();

                    if rng.random_bool(0.5) {
                        self.carve_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.carve_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.carve_vertical_tunnel(prev_y, new_y, prev_x);
                        self.carve_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);
            }
        }

        // Place stairs in the last room
        if let Some(last_room) = self.rooms.last() {
            let (cx, cy) = last_room.center();
            self.set_tile(cx, cy, TileType::StairsDown);
        }
    }

    fn carve_room(&mut self, room: &Room) {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                self.set_tile(x, y, TileType::Floor);
            }
        }
    }

    fn carve_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in start..=end {
            self.set_tile(x, y, TileType::Floor);
        }
    }

    fn carve_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in start..=end {
            self.set_tile(x, y, TileType::Floor);
        }
    }

    /// Find which room contains a given position.
    pub fn room_at(&self, x: i32, y: i32) -> Option<&Room> {
        self.rooms.iter().find(|r| r.contains(x, y))
    }

    /// Compute field of view from a position.
    pub fn compute_fov(&mut self, x: i32, y: i32, radius: i32) {
        // Reset visibility
        self.visible.fill(false);

        // Simple raycasting FOV
        for angle in 0..360 {
            let rad = (angle as f32).to_radians();
            let dx = rad.cos();
            let dy = rad.sin();

            let mut cx = x as f32 + 0.5;
            let mut cy = y as f32 + 0.5;

            for _ in 0..radius {
                let ix = cx as i32;
                let iy = cy as i32;

                if !self.in_bounds(ix, iy) {
                    break;
                }

                let idx = self.xy_idx(ix, iy);
                self.visible[idx] = true;
                self.revealed[idx] = true;

                if let Some(tile) = self.get_tile(ix, iy) {
                    if tile.blocks_sight() {
                        break;
                    }
                }

                cx += dx;
                cy += dy;
            }
        }
    }
}
