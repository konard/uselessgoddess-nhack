//! Game screens and state management.

mod gameplay;

use bevy::prelude::*;

/// Plugin for screen management.
pub fn plugin(app: &mut App) {
    app.init_state::<GameScreen>()
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
