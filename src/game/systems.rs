//! Core game systems for movement, combat, etc.

use bevy::prelude::*;

use super::{CombatStats, GameMap, Health, Monster, Player, Position};

/// Plugin for game systems.
pub fn plugin(app: &mut App) {
    app.add_message::<MoveIntent>()
        .add_message::<CombatEvent>()
        .add_message::<GameLogEvent>()
        .init_resource::<GameLog>()
        .add_systems(
            Update,
            (
                process_movement.run_if(on_message::<MoveIntent>),
                process_combat.run_if(on_message::<CombatEvent>),
                cleanup_dead_entities,
                update_fov,
            )
                .chain(),
        );
}

/// Message for movement intentions.
#[derive(Message, Debug)]
pub struct MoveIntent {
    pub entity: Entity,
    pub dx: i32,
    pub dy: i32,
}

/// Message for combat actions.
#[derive(Message, Debug)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub defender: Entity,
}

/// Message for adding messages to the game log.
#[derive(Message, Debug)]
#[allow(dead_code)]
pub struct GameLogEvent {
    pub message: String,
}

/// Resource for storing game log messages.
#[derive(Resource, Debug, Default)]
pub struct GameLog {
    pub messages: Vec<String>,
    pub max_messages: usize,
}

impl GameLog {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
        }
    }

    pub fn add(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn recent(&self, count: usize) -> impl Iterator<Item = &String> {
        self.messages.iter().rev().take(count)
    }
}

/// Process movement intentions.
fn process_movement(
    mut messages: MessageReader<MoveIntent>,
    mut positions: Query<&mut Position, Without<Monster>>,
    monsters: Query<(Entity, &Position), With<Monster>>,
    map: Res<GameMap>,
    mut combat_messages: MessageWriter<CombatEvent>,
    mut log: ResMut<GameLog>,
) {
    for message in messages.read() {
        if let Ok(mut pos) = positions.get_mut(message.entity) {
            let new_x = pos.x + message.dx;
            let new_y = pos.y + message.dy;

            // Check for monster at destination
            let monster_at_dest = monsters
                .iter()
                .find(|(_, monster_pos)| monster_pos.x == new_x && monster_pos.y == new_y);

            if let Some((monster_entity, _)) = monster_at_dest {
                // Initiate combat instead of moving
                combat_messages.write(CombatEvent {
                    attacker: message.entity,
                    defender: monster_entity,
                });
            } else if map.is_walkable(new_x, new_y) {
                pos.x = new_x;
                pos.y = new_y;
            } else {
                log.add("You bump into a wall.");
            }
        }
    }
}

/// Process combat messages.
fn process_combat(
    mut messages: MessageReader<CombatEvent>,
    mut combatants: Query<(&CombatStats, &mut Health, Option<&Player>, Option<&Monster>)>,
    names: Query<Option<&Name>>,
    mut log: ResMut<GameLog>,
) {
    for message in messages.read() {
        let Ok([(attacker_stats, _, attacker_player, attacker_monster), (defender_stats, mut defender_health, defender_player, defender_monster)]) =
            combatants.get_many_mut([message.attacker, message.defender]) else {
                continue;
            };

        let attacker_name = if attacker_player.is_some() {
            "You".to_string()
        } else if attacker_monster.is_some() {
            names
                .get(message.attacker)
                .ok()
                .flatten()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "Something".to_string())
        } else {
            "Something".to_string()
        };

        let defender_name = if defender_player.is_some() {
            "you".to_string()
        } else if defender_monster.is_some() {
            names
                .get(message.defender)
                .ok()
                .flatten()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "something".to_string())
        } else {
            "something".to_string()
        };

        let damage = (attacker_stats.attack - defender_stats.defense).max(0);
        defender_health.take_damage(damage);

        if damage > 0 {
            log.add(format!(
                "{} hits {} for {} damage!",
                attacker_name, defender_name, damage
            ));
        } else {
            log.add(format!(
                "{} attacks {} but does no damage.",
                attacker_name, defender_name
            ));
        }

        if defender_health.is_dead() {
            log.add(format!("{} is slain!", defender_name));
        }
    }
}

/// Remove dead entities from the game.
fn cleanup_dead_entities(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Without<Player>, Changed<Health>)>,
) {
    for (entity, health) in &query {
        if health.is_dead() {
            commands.entity(entity).despawn();
        }
    }
}

/// Update field of view when player moves.
fn update_fov(
    player_query: Query<&Position, (With<Player>, Changed<Position>)>,
    mut map: ResMut<GameMap>,
) {
    if let Ok(player_pos) = player_query.single() {
        map.compute_fov(player_pos.x, player_pos.y, 8);
    }
}
