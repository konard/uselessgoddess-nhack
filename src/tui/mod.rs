//! TUI module - Terminal user interface with ratatui.

mod mouse;
mod render;
mod tooltip;
mod widgets;

pub use mouse::{GridClickMessage, GridHoverMessage, GridViewport, MouseState};
pub use render::CurrentNarrative;
pub use tooltip::TooltipState;

use bevy::prelude::*;

/// Plugin for TUI rendering.
pub fn plugin(app: &mut App) {
    app.add_plugins((render::plugin, mouse::plugin, tooltip::plugin));
}
