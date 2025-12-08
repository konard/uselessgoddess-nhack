//! Mouse input handling for TUI.
//!
//! Captures mouse events from crossterm and translates them into
//! world grid coordinates for game interaction.

use bevy::prelude::*;
use bevy_ratatui::event::MouseMessage;
use crossterm::event::{MouseButton, MouseEventKind};

use crate::game::Position;

/// Plugin for mouse input systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<MouseState>()
        .add_message::<GridClickMessage>()
        .add_message::<GridHoverMessage>()
        .add_systems(Update, process_mouse_input);
}

/// Current mouse state.
#[derive(Resource, Debug, Default)]
pub struct MouseState {
    /// Current mouse position on screen (column, row).
    pub screen_position: Option<(u16, u16)>,
    /// Current mouse position in world grid (x, y).
    pub grid_position: Option<Position>,
    /// Entity currently being hovered over.
    pub hovered_entity: Option<Entity>,
    /// Whether the left mouse button is pressed.
    pub left_pressed: bool,
    /// Whether the right mouse button is pressed.
    pub right_pressed: bool,
}

/// Message sent when player clicks on a grid tile.
#[derive(Message, Debug, Clone)]
pub struct GridClickMessage {
    /// World grid position that was clicked.
    pub position: Position,
    /// Which mouse button was used.
    pub button: MouseButton,
    /// Entity at this position, if any.
    pub entity: Option<Entity>,
}

/// Message sent when mouse hovers over a new grid tile.
#[derive(Message, Debug, Clone)]
pub struct GridHoverMessage {
    /// World grid position being hovered.
    pub position: Position,
    /// Entity at this position, if any.
    pub entity: Option<Entity>,
    /// Previous position (for detecting changes).
    pub previous_position: Option<Position>,
}

/// Configuration for converting screen coordinates to grid coordinates.
#[derive(Resource, Debug, Clone)]
pub struct GridViewport {
    /// Offset of the map widget in screen coordinates.
    pub map_offset: (u16, u16),
    /// Size of the map widget in screen coordinates.
    pub map_size: (u16, u16),
    /// Center of the viewport in world coordinates (typically player position).
    pub world_center: Position,
}

impl Default for GridViewport {
    fn default() -> Self {
        Self {
            // These will be updated by the render system
            map_offset: (1, 1), // Account for border
            map_size: (40, 20),
            world_center: Position::new(0, 0),
        }
    }
}

impl GridViewport {
    /// Convert screen coordinates to world grid coordinates.
    pub fn screen_to_grid(&self, screen_x: u16, screen_y: u16) -> Option<Position> {
        // Check if within map bounds
        if screen_x < self.map_offset.0
            || screen_y < self.map_offset.1
            || screen_x >= self.map_offset.0 + self.map_size.0
            || screen_y >= self.map_offset.1 + self.map_size.1
        {
            return None;
        }

        // Calculate relative position within map widget
        let rel_x = (screen_x - self.map_offset.0) as i32;
        let rel_y = (screen_y - self.map_offset.1) as i32;

        // Calculate world position (centered on player)
        let center_x = self.map_size.0 as i32 / 2;
        let center_y = self.map_size.1 as i32 / 2;

        let world_x = self.world_center.x + (rel_x - center_x);
        let world_y = self.world_center.y + (rel_y - center_y);

        Some(Position::new(world_x, world_y))
    }
}

/// Process mouse input and convert to grid coordinates.
fn process_mouse_input(
    mut mouse_messages: MessageReader<MouseMessage>,
    mut state: ResMut<MouseState>,
    viewport: Option<Res<GridViewport>>,
    mut click_messages: MessageWriter<GridClickMessage>,
    mut hover_messages: MessageWriter<GridHoverMessage>,
    entities_query: Query<(Entity, &Position)>,
) {
    let Some(viewport) = viewport else {
        return;
    };

    for mouse_msg in mouse_messages.read() {
        let event = &mouse_msg.0;

        // Update screen position
        state.screen_position = Some((event.column, event.row));

        // Convert to grid position
        let new_grid_pos = viewport.screen_to_grid(event.column, event.row);
        let previous_position = state.grid_position;

        // Find entity at this position
        let entity_at_pos = new_grid_pos.and_then(|pos| {
            entities_query
                .iter()
                .find(|(_, entity_pos)| **entity_pos == pos)
                .map(|(entity, _)| entity)
        });

        match event.kind {
            MouseEventKind::Down(button) => {
                match button {
                    MouseButton::Left => state.left_pressed = true,
                    MouseButton::Right => state.right_pressed = true,
                    _ => {}
                }

                // Send click message if on grid
                if let Some(pos) = new_grid_pos {
                    click_messages.write(GridClickMessage {
                        position: pos,
                        button,
                        entity: entity_at_pos,
                    });
                }
            }
            MouseEventKind::Up(button) => {
                match button {
                    MouseButton::Left => state.left_pressed = false,
                    MouseButton::Right => state.right_pressed = false,
                    _ => {}
                }
            }
            MouseEventKind::Moved => {
                // Check if grid position changed
                if new_grid_pos != state.grid_position {
                    state.grid_position = new_grid_pos;
                    state.hovered_entity = entity_at_pos;

                    // Send hover message if on grid
                    if let Some(pos) = new_grid_pos {
                        hover_messages.write(GridHoverMessage {
                            position: pos,
                            entity: entity_at_pos,
                            previous_position,
                        });
                    }
                }
            }
            MouseEventKind::Drag(_button) => {
                // Update position during drag
                if new_grid_pos != state.grid_position {
                    state.grid_position = new_grid_pos;
                    state.hovered_entity = entity_at_pos;
                }
            }
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => {
                // Could be used for scrolling message log, etc.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_to_grid_conversion() {
        let viewport = GridViewport {
            map_offset: (1, 1),
            map_size: (40, 20),
            world_center: Position::new(50, 50),
        };

        // Center of viewport should be player position
        let center_screen_x = 1 + 20; // offset + half width
        let center_screen_y = 1 + 10; // offset + half height
        let grid_pos = viewport.screen_to_grid(center_screen_x, center_screen_y);
        assert_eq!(grid_pos, Some(Position::new(50, 50)));

        // Top-left of viewport
        let grid_pos = viewport.screen_to_grid(1, 1);
        assert_eq!(grid_pos, Some(Position::new(30, 40))); // 50-20, 50-10

        // Outside map bounds
        let grid_pos = viewport.screen_to_grid(0, 0);
        assert_eq!(grid_pos, None);
    }
}
