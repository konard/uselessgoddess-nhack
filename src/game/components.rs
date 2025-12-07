//! ECS Components for game entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for registering game components.
pub fn plugin(app: &mut App) {
    app.register_type::<Position>()
        .register_type::<Player>()
        .register_type::<Monster>()
        .register_type::<Item>()
        .register_type::<Health>()
        .register_type::<CombatStats>()
        .register_type::<Description>()
        .register_type::<Dialogue>()
        .register_type::<Renderable>();
}

/// Grid position in the game world.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position.
    pub fn distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Marker component for the player entity.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Marker component for monster entities.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Monster {
    pub kind: MonsterKind,
}

/// Types of monsters in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum MonsterKind {
    Orc,
    Goblin,
    Troll,
    Wraith,
    Skeleton,
    Demon,
}

impl MonsterKind {
    pub fn glyph(&self) -> char {
        match self {
            MonsterKind::Orc => 'o',
            MonsterKind::Goblin => 'g',
            MonsterKind::Troll => 'T',
            MonsterKind::Wraith => 'W',
            MonsterKind::Skeleton => 's',
            MonsterKind::Demon => 'D',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MonsterKind::Orc => "Orc",
            MonsterKind::Goblin => "Goblin",
            MonsterKind::Troll => "Troll",
            MonsterKind::Wraith => "Wraith",
            MonsterKind::Skeleton => "Skeleton",
            MonsterKind::Demon => "Demon",
        }
    }
}

/// Marker component for item entities.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Item {
    pub kind: ItemKind,
}

/// Types of items in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum ItemKind {
    HealthPotion,
    Sword,
    Shield,
    Gold,
    Key,
    Scroll,
    Chest,
}

impl ItemKind {
    pub fn glyph(&self) -> char {
        match self {
            ItemKind::HealthPotion => '!',
            ItemKind::Sword => '/',
            ItemKind::Shield => '[',
            ItemKind::Gold => '$',
            ItemKind::Key => '%',
            ItemKind::Scroll => '?',
            ItemKind::Chest => '=',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ItemKind::HealthPotion => "Health Potion",
            ItemKind::Sword => "Sword",
            ItemKind::Shield => "Shield",
            ItemKind::Gold => "Gold",
            ItemKind::Key => "Key",
            ItemKind::Scroll => "Scroll",
            ItemKind::Chest => "Chest",
        }
    }
}

/// Health points for entities that can take damage.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    pub fn percentage(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

/// Combat statistics for entities.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CombatStats {
    pub attack: i32,
    pub defense: i32,
}

impl CombatStats {
    pub fn new(attack: i32, defense: i32) -> Self {
        Self { attack, defense }
    }
}

/// AI-generated description component.
/// Populated by the AI Game Master.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Description {
    /// Short narrative description.
    pub text: String,
    /// Horror-themed atmospheric flavor.
    pub atmosphere: String,
    /// Any special hints or lore.
    pub lore: Option<String>,
}

/// AI-generated dialogue component for NPCs.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Dialogue {
    /// Current dialogue line.
    pub current_line: String,
    /// Available dialogue options.
    pub options: Vec<String>,
    /// NPC's mood/disposition.
    pub mood: String,
}

/// Visual representation in the TUI.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Renderable {
    pub glyph: char,
    pub fg_color: RenderColor,
    pub bg_color: RenderColor,
}

/// Color representation for TUI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum RenderColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    Reset,
}

impl Default for RenderColor {
    fn default() -> Self {
        Self::Reset
    }
}

impl From<RenderColor> for ratatui::style::Color {
    fn from(color: RenderColor) -> Self {
        match color {
            RenderColor::Black => ratatui::style::Color::Black,
            RenderColor::Red => ratatui::style::Color::Red,
            RenderColor::Green => ratatui::style::Color::Green,
            RenderColor::Yellow => ratatui::style::Color::Yellow,
            RenderColor::Blue => ratatui::style::Color::Blue,
            RenderColor::Magenta => ratatui::style::Color::Magenta,
            RenderColor::Cyan => ratatui::style::Color::Cyan,
            RenderColor::White => ratatui::style::Color::White,
            RenderColor::DarkGray => ratatui::style::Color::DarkGray,
            RenderColor::LightRed => ratatui::style::Color::LightRed,
            RenderColor::LightGreen => ratatui::style::Color::LightGreen,
            RenderColor::LightYellow => ratatui::style::Color::LightYellow,
            RenderColor::LightBlue => ratatui::style::Color::LightBlue,
            RenderColor::LightMagenta => ratatui::style::Color::LightMagenta,
            RenderColor::LightCyan => ratatui::style::Color::LightCyan,
            RenderColor::Reset => ratatui::style::Color::Reset,
        }
    }
}
