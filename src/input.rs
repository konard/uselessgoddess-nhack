//! Input handling system with action cooldowns and unified cursor.
//!
//! This module provides robust input handling that:
//! - Uses an `InputTimer` to prevent input jitter when holding keys
//! - Provides a unified `Cursor` resource for mouse and keyboard inspection
//! - Integrates with the game's state system for mode-specific input handling

use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::game::{MoveIntent, Player, Position};
use crate::screens::{GameMode, GameScreen};

/// Plugin for input handling systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<InputTimer>()
        .init_resource::<Cursor>()
        .init_resource::<InputState>()
        .add_systems(
            Update,
            (
                update_input_timer,
                handle_input.after(update_input_timer),
                snap_cursor_to_player.after(handle_input),
            )
                .run_if(in_state(GameScreen::Playing)),
        );
}

/// Resource for managing input cooldowns to prevent jitter when holding keys.
///
/// When a player holds down a movement key, the terminal sends key repeat events
/// faster than the game's logical tick rate. This timer throttles input processing
/// to provide consistent movement speed.
#[derive(Resource)]
pub struct InputTimer {
    /// Timer that controls how often movement/actions can be processed.
    pub timer: Timer,
    /// Whether an action was processed this frame.
    pub action_taken: bool,
}

impl Default for InputTimer {
    fn default() -> Self {
        Self {
            // 100ms cooldown between actions for responsive but controlled movement
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            action_taken: false,
        }
    }
}

impl InputTimer {
    /// Check if an action can be performed and mark it as taken.
    pub fn try_action(&mut self) -> bool {
        if self.timer.is_finished() && !self.action_taken {
            self.action_taken = true;
            true
        } else {
            false
        }
    }

    /// Reset the action taken flag (called at the start of each frame).
    pub fn reset_action(&mut self) {
        self.action_taken = false;
    }
}

/// Unified cursor resource for inspecting entities on the map.
///
/// The cursor can be controlled by:
/// - Mouse movement (updates position directly)
/// - Keyboard in Look/Targeting mode (HJKL/arrows move cursor)
/// - Snaps to player position when in Exploring mode (unless in Look mode)
#[derive(Resource, Debug, Clone)]
pub struct Cursor {
    /// Current grid position of the cursor.
    pub position: Position,
    /// Whether the cursor is currently visible.
    pub visible: bool,
    /// Entity at the cursor position (if any).
    pub target_entity: Option<Entity>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: Position::new(0, 0),
            visible: false,
            target_entity: None,
        }
    }
}

impl Cursor {
    /// Create a new cursor at the given position.
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            position: Position::new(x, y),
            visible: false,
            target_entity: None,
        }
    }

    /// Move the cursor by a delta.
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    /// Set the cursor position directly.
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position.x = x;
        self.position.y = y;
    }

    /// Snap the cursor to a specific position (usually the player).
    pub fn snap_to(&mut self, pos: &Position) {
        self.position = *pos;
    }
}

/// Tracks the current input state and modifiers.
#[derive(Resource, Default)]
pub struct InputState {
    /// Whether shift is held (for running/fast movement).
    pub shift_held: bool,
    /// Whether control is held (for special commands).
    pub ctrl_held: bool,
    /// Last direction pressed (for repeated movement).
    pub last_direction: Option<(i32, i32)>,
}

/// Update the input timer each frame.
fn update_input_timer(time: Res<Time>, mut input_timer: ResMut<InputTimer>) {
    input_timer.timer.tick(time.delta());
    if input_timer.timer.just_finished() {
        input_timer.reset_action();
    }
}

/// Main input handling system.
fn handle_input(
    mut key_messages: MessageReader<KeyMessage>,
    mut move_messages: MessageWriter<MoveIntent>,
    mut input_timer: ResMut<InputTimer>,
    mut cursor: ResMut<Cursor>,
    mut input_state: ResMut<InputState>,
    mut next_mode: ResMut<NextState<GameMode>>,
    current_mode: Res<State<GameMode>>,
    player_query: Query<Entity, With<Player>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for key_message in key_messages.read() {
        let key = &key_message.0;

        // Track modifier state
        input_state.shift_held = key.modifiers.contains(KeyModifiers::SHIFT);
        input_state.ctrl_held = key.modifiers.contains(KeyModifiers::CONTROL);

        // Global controls (work in any mode)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            exit.write(bevy::app::AppExit::Success);
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                exit.write(bevy::app::AppExit::Success);
                return;
            }
            KeyCode::Esc => {
                // Return to exploring mode from any mode
                if *current_mode.get() != GameMode::Exploring {
                    next_mode.set(GameMode::Exploring);
                    cursor.visible = false;
                }
                continue;
            }
            _ => {}
        }

        // Mode-specific input handling
        match current_mode.get() {
            GameMode::Exploring => {
                handle_exploring_input(
                    key,
                    &mut move_messages,
                    &mut input_timer,
                    &mut next_mode,
                    &mut cursor,
                    player_entity,
                );
            }
            GameMode::Targeting => {
                handle_targeting_input(
                    key,
                    &mut cursor,
                    &mut next_mode,
                    &mut input_timer,
                );
            }
            GameMode::Inventory => {
                handle_inventory_input(key, &mut next_mode);
            }
        }
    }
}

/// Handle input in Exploring mode (normal gameplay).
fn handle_exploring_input(
    key: &crossterm::event::KeyEvent,
    move_messages: &mut MessageWriter<MoveIntent>,
    input_timer: &mut InputTimer,
    next_mode: &mut NextState<GameMode>,
    cursor: &mut Cursor,
    player_entity: Entity,
) {
    // Movement keys
    let movement = match key.code {
        KeyCode::Char('h') | KeyCode::Left => Some((-1, 0)),
        KeyCode::Char('j') | KeyCode::Down => Some((0, 1)),
        KeyCode::Char('k') | KeyCode::Up => Some((0, -1)),
        KeyCode::Char('l') | KeyCode::Right => Some((1, 0)),
        // Diagonal movement
        KeyCode::Char('y') => Some((-1, -1)),
        KeyCode::Char('u') => Some((1, -1)),
        KeyCode::Char('b') => Some((-1, 1)),
        KeyCode::Char('n') => Some((1, 1)),
        _ => None,
    };

    if let Some((dx, dy)) = movement {
        // Only process movement if the input timer allows
        if input_timer.try_action() {
            move_messages.write(MoveIntent {
                entity: player_entity,
                dx,
                dy,
            });
        }
        return;
    }

    // Non-movement keys (no cooldown needed)
    match key.code {
        // Wait
        KeyCode::Char('.') => {
            // Pass time (no action needed)
        }
        // Enter targeting/look mode
        KeyCode::Char('x') => {
            next_mode.set(GameMode::Targeting);
            cursor.visible = true;
        }
        // Open inventory
        KeyCode::Char('i') => {
            next_mode.set(GameMode::Inventory);
        }
        _ => {}
    }
}

/// Handle input in Targeting mode (cursor movement for looking/targeting).
fn handle_targeting_input(
    key: &crossterm::event::KeyEvent,
    cursor: &mut Cursor,
    next_mode: &mut NextState<GameMode>,
    input_timer: &mut InputTimer,
) {
    // Cursor movement (uses same keys as player movement)
    let movement = match key.code {
        KeyCode::Char('h') | KeyCode::Left => Some((-1, 0)),
        KeyCode::Char('j') | KeyCode::Down => Some((0, 1)),
        KeyCode::Char('k') | KeyCode::Up => Some((0, -1)),
        KeyCode::Char('l') | KeyCode::Right => Some((1, 0)),
        // Diagonal movement
        KeyCode::Char('y') => Some((-1, -1)),
        KeyCode::Char('u') => Some((1, -1)),
        KeyCode::Char('b') => Some((-1, 1)),
        KeyCode::Char('n') => Some((1, 1)),
        _ => None,
    };

    if let Some((dx, dy)) = movement {
        if input_timer.try_action() {
            cursor.move_by(dx, dy);
        }
        return;
    }

    match key.code {
        // Confirm target selection
        KeyCode::Enter | KeyCode::Char(' ') => {
            // TODO: Implement target confirmation (spell casting, etc.)
            next_mode.set(GameMode::Exploring);
            cursor.visible = false;
        }
        // Cancel and return to exploring
        KeyCode::Char('x') => {
            next_mode.set(GameMode::Exploring);
            cursor.visible = false;
        }
        _ => {}
    }
}

/// Handle input in Inventory mode.
fn handle_inventory_input(key: &crossterm::event::KeyEvent, next_mode: &mut NextState<GameMode>) {
    match key.code {
        // Close inventory
        KeyCode::Char('i') => {
            next_mode.set(GameMode::Exploring);
        }
        // TODO: Implement inventory navigation and item usage
        _ => {}
    }
}

/// Snap cursor to player position when in Exploring mode.
fn snap_cursor_to_player(
    current_mode: Res<State<GameMode>>,
    player_query: Query<&Position, With<Player>>,
    mut cursor: ResMut<Cursor>,
) {
    // Only snap when exploring (not when targeting)
    if *current_mode.get() == GameMode::Exploring {
        if let Ok(player_pos) = player_query.single() {
            cursor.snap_to(player_pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_timer_cooldown() {
        let mut timer = InputTimer::default();

        // Simulate time passing to make timer finished
        timer.timer.tick(std::time::Duration::from_millis(101));
        assert!(timer.timer.is_finished());

        // First action should succeed
        assert!(timer.try_action());

        // Second action should fail (already taken this frame)
        assert!(!timer.try_action());

        // After reset, timer needs to tick again
        timer.reset_action();
        // Timer is in repeating mode, so it needs another tick
        timer.timer.tick(std::time::Duration::from_millis(101));
        assert!(timer.try_action());
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new(5, 5);

        cursor.move_by(1, 0);
        assert_eq!(cursor.position.x, 6);
        assert_eq!(cursor.position.y, 5);

        cursor.move_by(-2, 3);
        assert_eq!(cursor.position.x, 4);
        assert_eq!(cursor.position.y, 8);
    }

    #[test]
    fn test_cursor_snap() {
        let mut cursor = Cursor::new(0, 0);
        let pos = Position::new(10, 20);

        cursor.snap_to(&pos);
        assert_eq!(cursor.position.x, 10);
        assert_eq!(cursor.position.y, 20);
    }
}
