//! TUI rendering system using bevy_ratatui.

use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui::RatatuiContext;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};

use super::widgets::{HelpWidget, LogWidget, MapWidget, NarrativeWidget, StatsWidget};
use crate::game::{GameLog, GameMap, Health, Item, Monster, MoveIntent, Player, Position};

/// Plugin for rendering systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentNarrative>()
        .add_systems(Update, (handle_input, render_ui.after(handle_input)));
}

/// Currently displayed AI narrative.
#[derive(Resource, Default)]
pub struct CurrentNarrative {
    pub title: String,
    pub text: String,
    pub atmosphere: Option<String>,
    pub visible: bool,
}

/// Handle keyboard input.
fn handle_input(
    mut key_messages: MessageReader<KeyMessage>,
    mut move_messages: MessageWriter<MoveIntent>,
    player_query: Query<Entity, With<Player>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for key_message in key_messages.read() {
        let key = &key_message.0;

        // Check for Ctrl+C to quit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            exit.write(bevy::app::AppExit::Success);
            return;
        }

        match key.code {
            // Movement keys (vi-style and arrows)
            KeyCode::Char('h') | KeyCode::Left => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: -1,
                    dy: 0,
                });
            }
            KeyCode::Char('j') | KeyCode::Down => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: 0,
                    dy: 1,
                });
            }
            KeyCode::Char('k') | KeyCode::Up => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: 0,
                    dy: -1,
                });
            }
            KeyCode::Char('l') | KeyCode::Right => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: 1,
                    dy: 0,
                });
            }
            // Diagonal movement
            KeyCode::Char('y') => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: -1,
                    dy: -1,
                });
            }
            KeyCode::Char('u') => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: 1,
                    dy: -1,
                });
            }
            KeyCode::Char('b') => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: -1,
                    dy: 1,
                });
            }
            KeyCode::Char('n') => {
                move_messages.write(MoveIntent {
                    entity: player_entity,
                    dx: 1,
                    dy: 1,
                });
            }
            // Wait
            KeyCode::Char('.') => {
                // Do nothing, just pass time
            }
            // Quit
            KeyCode::Char('q') => {
                exit.write(bevy::app::AppExit::Success);
            }
            _ => {}
        }
    }
}

/// Render the game UI.
fn render_ui(
    mut context: ResMut<RatatuiContext>,
    map: Res<GameMap>,
    log: Res<GameLog>,
    narrative: Res<CurrentNarrative>,
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

            // Right sidebar: stats, narrative, help
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),  // Stats
                    Constraint::Min(8),     // Narrative
                    Constraint::Length(7),  // Help
                ])
                .split(main_chunks[1]);

            // Render map
            let map_widget = MapWidget::new(&map)
                .with_player(*player_pos)
                .with_entities(entities);
            frame.render_widget(map_widget, left_chunks[0]);

            // Render message log
            let log_widget = LogWidget::new(&log).with_max_lines(5);
            frame.render_widget(log_widget, left_chunks[1]);

            // Render stats
            let stats_widget = StatsWidget::new(player_health.clone(), map.current_floor, 0);
            frame.render_widget(stats_widget, right_chunks[0]);

            // Render narrative (if available)
            if narrative.visible {
                let narrative_widget = NarrativeWidget::new(&narrative.title, &narrative.text);
                let narrative_widget = if let Some(ref atmo) = narrative.atmosphere {
                    narrative_widget.with_atmosphere(atmo)
                } else {
                    narrative_widget
                };
                frame.render_widget(narrative_widget, right_chunks[1]);
            }

            // Render help
            frame.render_widget(HelpWidget, right_chunks[2]);
        })
        .expect("Failed to draw frame");
}
