//! Tooltip system for displaying entity information on hover.

use bevy::prelude::*;

use super::mouse::{GridHoverMessage, MouseState};
use crate::dnd::{AbilityScores, ArmorClass};
use crate::dnd::components::HitPoints;
use crate::dnd::monsters::MonsterTemplate;
use crate::game::{Monster, Player};

/// Plugin for tooltip systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<TooltipState>()
        .add_systems(Update, (update_tooltip_on_hover, clear_tooltip_on_empty));
}

/// Current tooltip state.
#[derive(Resource, Debug, Default)]
pub struct TooltipState {
    /// Whether the tooltip is visible.
    pub visible: bool,
    /// Title of the tooltip (entity name).
    pub title: String,
    /// Lines of tooltip content.
    pub lines: Vec<TooltipLine>,
    /// Screen position for the tooltip (near mouse).
    pub screen_position: (u16, u16),
}

/// A line in the tooltip with optional styling.
#[derive(Debug, Clone)]
pub struct TooltipLine {
    pub text: String,
    pub style: TooltipStyle,
}

/// Style for tooltip lines.
#[derive(Debug, Clone, Copy, Default)]
pub enum TooltipStyle {
    #[default]
    Normal,
    Header,
    Stat,
    Warning,
    Good,
}

impl TooltipLine {
    pub fn normal(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TooltipStyle::Normal,
        }
    }

    pub fn header(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TooltipStyle::Header,
        }
    }

    pub fn stat(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TooltipStyle::Stat,
        }
    }
}

/// Update tooltip when hovering over an entity.
fn update_tooltip_on_hover(
    mut hover_messages: MessageReader<GridHoverMessage>,
    mut tooltip: ResMut<TooltipState>,
    mouse_state: Res<MouseState>,
    // Query for different entity types
    monster_query: Query<
        (
            &Name,
            Option<&MonsterTemplate>,
            Option<&HitPoints>,
            Option<&ArmorClass>,
            Option<&AbilityScores>,
        ),
        With<Monster>,
    >,
    player_query: Query<
        (&Name, Option<&HitPoints>, Option<&ArmorClass>),
        With<Player>,
    >,
) {
    for hover_msg in hover_messages.read() {
        // Clear tooltip if no entity
        let Some(entity) = hover_msg.entity else {
            tooltip.visible = false;
            return;
        };

        // Try to get entity info
        if let Ok((name, template, hp, ac, abilities)) = monster_query.get(entity) {
            tooltip.visible = true;
            tooltip.title = if let Some(t) = template {
                t.display_name()
            } else {
                name.to_string()
            };

            tooltip.lines.clear();

            // Add monster type info
            if let Some(t) = template {
                let data = t.data();
                tooltip.lines.push(TooltipLine::normal(format!(
                    "CR {} {}",
                    data.challenge_rating.display(),
                    format!("{:?}", data.creature_type).to_lowercase()
                )));
            }

            // Add HP info
            if let Some(hp) = hp {
                let hp_pct = hp.percentage();
                let hp_desc = if hp_pct >= 1.0 {
                    "Uninjured"
                } else if hp_pct >= 0.75 {
                    "Lightly wounded"
                } else if hp_pct >= 0.5 {
                    "Wounded"
                } else if hp_pct >= 0.25 {
                    "Heavily wounded"
                } else {
                    "Near death"
                };
                tooltip.lines.push(TooltipLine::stat(hp_desc));
            }

            // Add AC info
            if let Some(ac) = ac {
                tooltip.lines.push(TooltipLine::stat(format!("AC: {}", ac.total())));
            }

            // Add ability scores (abbreviated)
            if let Some(ab) = abilities {
                tooltip.lines.push(TooltipLine::stat(format!(
                    "STR {} DEX {} CON {}",
                    ab.strength, ab.dexterity, ab.constitution
                )));
            }

            // Add flavor description if available
            if let Some(t) = template {
                if let Some(ref desc) = t.visual_description {
                    tooltip.lines.push(TooltipLine::normal(desc.clone()));
                }
            }
        } else if let Ok((name, hp, ac)) = player_query.get(entity) {
            // It's the player
            tooltip.visible = true;
            tooltip.title = name.to_string();
            tooltip.lines.clear();

            if let Some(hp) = hp {
                tooltip.lines.push(TooltipLine::stat(format!(
                    "HP: {}/{}",
                    hp.current, hp.max
                )));
            }

            if let Some(ac) = ac {
                tooltip.lines.push(TooltipLine::stat(format!("AC: {}", ac.total())));
            }
        } else {
            tooltip.visible = false;
        }

        // Update screen position
        if let Some((x, y)) = mouse_state.screen_position {
            tooltip.screen_position = (x + 2, y);
        }
    }
}

/// Clear tooltip when mouse moves to empty space.
fn clear_tooltip_on_empty(
    mouse_state: Res<MouseState>,
    mut tooltip: ResMut<TooltipState>,
) {
    // If no grid position or no entity, hide tooltip
    if mouse_state.grid_position.is_none() || mouse_state.hovered_entity.is_none() {
        if tooltip.visible {
            tooltip.visible = false;
        }
    }
}
