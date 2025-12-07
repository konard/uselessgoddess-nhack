//! TUI module - Terminal user interface with ratatui.

mod render;
mod widgets;

pub use render::CurrentNarrative;

use bevy::prelude::*;

/// Plugin for TUI rendering.
pub fn plugin(app: &mut App) {
    app.add_plugins(render::plugin);
}
