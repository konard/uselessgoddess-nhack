//! Theme module - visual styling constants.

use bevy::prelude::*;

/// Plugin for theme resources.
pub fn plugin(_app: &mut App) {
    // Theme is currently just constants, no runtime resources needed
}

/// Color palette for the horror theme.
pub mod colors {
    use ratatui::style::Color;

    /// Blood red for damage and enemies
    pub const BLOOD: Color = Color::Rgb(139, 0, 0);

    /// Sickly green for poison/corruption
    pub const POISON: Color = Color::Rgb(0, 100, 0);

    /// Ethereal blue for magic/spirits
    pub const ETHEREAL: Color = Color::Rgb(100, 149, 237);

    /// Gold for treasure/important items
    pub const GOLD: Color = Color::Rgb(255, 215, 0);

    /// Bone white for skeletons/death
    pub const BONE: Color = Color::Rgb(255, 250, 240);

    /// Shadow purple for darkness/void
    pub const SHADOW: Color = Color::Rgb(48, 0, 48);

    /// Rust orange for decay
    pub const RUST: Color = Color::Rgb(183, 65, 14);

    /// Eldritch cyan for otherworldly elements
    pub const ELDRITCH: Color = Color::Rgb(0, 255, 255);
}

/// ASCII art title for the game.
pub const TITLE_ART: &str = r#"
 _   _                      _   _            _
| \ | | ___ _   _ _ __ ___ | | | | __ _  ___| | __
|  \| |/ _ \ | | | '__/ _ \| |_| |/ _` |/ __| |/ /
| |\  |  __/ |_| | | | (_) |  _  | (_| | (__|   <
|_| \_|\___|\__,_|_|  \___/|_| |_|\__,_|\___|_|\_\
"#;

/// Horror-themed loading messages.
pub const LOADING_MESSAGES: &[&str] = &[
    "Awakening ancient evils...",
    "Summoning the darkness...",
    "Opening the gates...",
    "The dungeon hungers...",
    "Preparing your doom...",
    "The shadows gather...",
];
