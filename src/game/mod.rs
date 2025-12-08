//! Game module - core roguelike mechanics and ECS components.

mod components;
mod map;
mod systems;

pub use components::*;
pub use map::*;
pub use systems::*;

use bevy::prelude::*;

/// Plugin for core game mechanics.
pub fn plugin(app: &mut App) {
    app.add_plugins((components::plugin, map::plugin, systems::plugin));
}
