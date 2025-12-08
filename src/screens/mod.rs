//! Game screens and state management.

mod gameplay;

use bevy::prelude::*;

/// Plugin for screen management.
pub fn plugin(app: &mut App) {
    app.init_state::<GameScreen>()
        .init_state::<GameMode>()
        .add_plugins(gameplay::plugin);
}

/// The main game screen states.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameScreen {
    /// Initial loading/title screen
    #[default]
    Loading,
    /// Main gameplay
    Playing,
    /// Player has died
    GameOver,
    /// Paused game
    #[allow(dead_code)]
    Paused,
}

/// Game mode states for in-game UI modes.
///
/// These represent different interaction modes while playing:
/// - Exploring: Standard movement and interaction
/// - Targeting: Cursor mode for looking at things or selecting targets
/// - Inventory: Viewing and managing inventory
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    /// Standard movement and exploration.
    #[default]
    Exploring,
    /// Cursor/look mode for inspecting entities or selecting targets.
    Targeting,
    /// Inventory management screen.
    Inventory,
}
