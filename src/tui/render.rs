//! TUI rendering system using bevy_ratatui.

use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;
use ratatui::layout::{Constraint, Direction, Layout};

use super::widgets::{
    CursorWidget, HelpWidget, InventoryWidget, LogWidget, MapWidget, NarrativeWidget, StatsWidget,
};
use crate::game::{GameLog, GameMap, Health, Item, Monster, Player, Position};
use crate::input::Cursor;
use crate::screens::{GameMode, GameScreen};

/// Plugin for rendering systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentNarrative>()
        .add_systems(Update, render_ui.run_if(in_state(GameScreen::Playing)));
}

/// Currently displayed AI narrative.
#[derive(Resource, Default)]
pub struct CurrentNarrative {
    pub title: String,
    pub text: String,
    pub atmosphere: Option<String>,
    pub visible: bool,
}

/// Render the game UI based on current game mode.
fn render_ui(
    mut context: ResMut<RatatuiContext>,
    map: Res<GameMap>,
    log: Res<GameLog>,
    narrative: Res<CurrentNarrative>,
    cursor: Res<Cursor>,
    game_mode: Res<State<GameMode>>,
    player_query: Query<(&Position, &Health), With<Player>>,
    monster_query: Query<(&Position, &Monster)>,
    item_query: Query<(&Position, &Item)>,
) {
    let Ok((player_pos, player_health)) = player_query.single() else {
        return;
    };

    // Collect entities for rendering
    let mut entities = Vec::new();

    // Add monsters
    for (pos, monster) in &monster_query {
        let glyph = monster.kind.glyph();
        let color = ratatui::style::Color::Red;
        entities.push((*pos, glyph, color));
    }

    // Add items
    for (pos, item) in &item_query {
        let glyph = item.kind.glyph();
        let color = ratatui::style::Color::Blue;
        entities.push((*pos, glyph, color));
    }

    context
        .draw(|frame| {
            let size = frame.area();

            match game_mode.get() {
                GameMode::Inventory => {
                    // Full-screen inventory view
                    render_inventory_mode(frame, size, player_health, &map, &log);
                }
                GameMode::Exploring | GameMode::Targeting => {
                    // Standard map view (with optional cursor highlight in targeting mode)
                    render_exploration_mode(
                        frame,
                        size,
                        &map,
                        &log,
                        &narrative,
                        &cursor,
                        player_pos,
                        player_health,
                        entities,
                        *game_mode.get() == GameMode::Targeting,
                    );
                }
            }
        })
        .expect("Failed to draw frame");
}

/// Render the standard exploration/targeting mode UI.
fn render_exploration_mode(
    frame: &mut ratatui::Frame,
    size: ratatui::layout::Rect,
    map: &GameMap,
    log: &GameLog,
    narrative: &CurrentNarrative,
    cursor: &Cursor,
    player_pos: &Position,
    player_health: &Health,
    entities: Vec<(Position, char, ratatui::style::Color)>,
    targeting_mode: bool,
) {
    // Main layout: map on left, sidebar on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(25)])
        .split(size);

    // Left side: map and messages
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(7)])
        .split(main_chunks[0]);

    // Right sidebar: stats, narrative/target info, help
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Stats
            Constraint::Min(8),    // Narrative or target info
            Constraint::Length(9), // Help (slightly larger for more controls)
        ])
        .split(main_chunks[1]);

    // Render map with cursor if in targeting mode
    let mut map_widget = MapWidget::new(map)
        .with_player(*player_pos)
        .with_entities(entities);

    if targeting_mode && cursor.visible {
        map_widget = map_widget.with_cursor(cursor.position);
    }

    frame.render_widget(map_widget, left_chunks[0]);

    // Render message log
    let log_widget = LogWidget::new(log).with_max_lines(5);
    frame.render_widget(log_widget, left_chunks[1]);

    // Render stats
    let stats_widget = StatsWidget::new(player_health.clone(), map.current_floor, 0);
    frame.render_widget(stats_widget, right_chunks[0]);

    // Render narrative or cursor info
    if targeting_mode && cursor.visible {
        // Show cursor position info
        let cursor_widget = CursorWidget::new(&cursor.position, map);
        frame.render_widget(cursor_widget, right_chunks[1]);
    } else if narrative.visible {
        let narrative_widget = NarrativeWidget::new(&narrative.title, &narrative.text);
        let narrative_widget = if let Some(ref atmo) = narrative.atmosphere {
            narrative_widget.with_atmosphere(atmo)
        } else {
            narrative_widget
        };
        frame.render_widget(narrative_widget, right_chunks[1]);
    }

    // Render help with mode-specific controls
    let help_widget = HelpWidget::new(targeting_mode);
    frame.render_widget(help_widget, right_chunks[2]);
}

/// Render the inventory mode UI.
fn render_inventory_mode(
    frame: &mut ratatui::Frame,
    size: ratatui::layout::Rect,
    player_health: &Health,
    map: &GameMap,
    log: &GameLog,
) {
    // Layout: inventory on left, stats and log on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(30)])
        .split(size);

    // Left side: inventory list
    let inventory_widget = InventoryWidget::new();
    frame.render_widget(inventory_widget, main_chunks[0]);

    // Right side: stats and recent messages
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(main_chunks[1]);

    let stats_widget = StatsWidget::new(player_health.clone(), map.current_floor, 0);
    frame.render_widget(stats_widget, right_chunks[0]);

    let log_widget = LogWidget::new(log).with_max_lines(10);
    frame.render_widget(log_widget, right_chunks[1]);
}
