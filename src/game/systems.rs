//! Core game systems for movement, combat, etc.

use bevy::prelude::*;

use crate::dnd::ability::AbilityScores;
use crate::dnd::combat::{make_attack, AttackModifier, AttackResult};
use crate::dnd::components::{Level, Proficiencies};
use crate::dnd::dice::DiceRoller;
use crate::dnd::weapons::{Weapon, WeaponType};

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

/// Process combat messages using the D&D combat system.
///
/// This system bridges the simple CombatStats component with the full D&D
/// combat mechanics via `dnd::combat::make_attack`. If entities have D&D
/// components (AbilityScores, Proficiencies, Level), those are used.
/// Otherwise, defaults based on CombatStats are generated.
fn process_combat(
    mut messages: MessageReader<CombatEvent>,
    mut combatants: Query<(&CombatStats, &mut Health, Option<&Player>, Option<&Monster>)>,
    dnd_query: Query<(Option<&AbilityScores>, Option<&Proficiencies>, Option<&Level>)>,
    names: Query<Option<&Name>>,
    mut dice_roller: ResMut<DiceRoller>,
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

        // Try to use D&D components if available, otherwise derive from CombatStats
        let (attacker_ability, attacker_profs, attacker_level) =
            get_dnd_stats(&dnd_query, message.attacker, attacker_stats);

        // Derive AC from defender's defense stat (base 10 + defense bonus)
        let target_ac = 10 + defender_stats.defense;

        // Create a weapon based on the attacker's attack stat
        // Higher attack stat = slightly better weapon
        let weapon = derive_weapon_from_attack(attacker_stats.attack);

        // Perform the D&D attack using make_attack
        let result = make_attack(
            &mut dice_roller,
            &attacker_ability,
            &attacker_profs,
            &attacker_level,
            &weapon,
            target_ac,
            AttackModifier::Normal,
        );

        // Log the result using D&D description
        log_attack_result(&mut log, &attacker_name, &defender_name, &result);

        // Apply damage if hit
        if result.hit {
            defender_health.take_damage(result.damage_total);

            if defender_health.is_dead() {
                log.add(format!("{} is slain!", defender_name));
            }
        }
    }
}

/// Get D&D stats for an entity, creating defaults if not present.
fn get_dnd_stats(
    dnd_query: &Query<(Option<&AbilityScores>, Option<&Proficiencies>, Option<&Level>)>,
    entity: Entity,
    combat_stats: &CombatStats,
) -> (AbilityScores, Proficiencies, Level) {
    if let Ok((maybe_ability, maybe_profs, maybe_level)) = dnd_query.get(entity) {
        let ability = maybe_ability.cloned().unwrap_or_else(|| {
            // Derive ability scores from attack stat
            // Attack 5 → STR 14, Attack 10 → STR 20
            let str_score = 10 + (combat_stats.attack.clamp(0, 10));
            AbilityScores {
                strength: str_score,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            }
        });

        let profs = maybe_profs.cloned().unwrap_or_else(|| {
            // Default to proficient with simple weapons
            let mut p = Proficiencies::default();
            p.simple_weapons = true;
            p
        });

        let level = maybe_level.cloned().unwrap_or_else(|| {
            // Derive level from combined stats
            let total = combat_stats.attack + combat_stats.defense;
            Level { value: (total / 4).clamp(1, 20) }
        });

        (ability, profs, level)
    } else {
        // Complete defaults based on combat stats
        let str_score = 10 + combat_stats.attack.clamp(0, 10);
        let ability = AbilityScores {
            strength: str_score,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };

        let mut profs = Proficiencies::default();
        profs.simple_weapons = true;

        let total = combat_stats.attack + combat_stats.defense;
        let level = Level { value: (total / 4).clamp(1, 20) };

        (ability, profs, level)
    }
}

/// Derive a weapon from the attack stat.
fn derive_weapon_from_attack(attack: i32) -> Weapon {
    // Higher attack stat = better weapon type
    let weapon_type = if attack >= 8 {
        WeaponType::Longsword // 1d8 martial weapon
    } else if attack >= 5 {
        WeaponType::Mace // 1d6 simple weapon
    } else if attack >= 3 {
        WeaponType::Dagger // 1d4 simple weapon
    } else {
        WeaponType::Unarmed // 1+STR
    };

    Weapon::new(weapon_type).equipped(true)
}

/// Log the attack result to the game log.
fn log_attack_result(log: &mut GameLog, attacker: &str, defender: &str, result: &AttackResult) {
    if result.fumble {
        log.add(format!(
            "{} swings wildly at {} but completely misses!",
            attacker, defender
        ));
    } else if result.critical {
        log.add(format!(
            "{} lands a CRITICAL HIT on {}! {} {} damage!",
            attacker,
            defender,
            result.damage_total,
            result.damage_type.name()
        ));
    } else if result.hit {
        log.add(format!(
            "{} hits {} for {} {} damage.",
            attacker,
            defender,
            result.damage_total,
            result.damage_type.name()
        ));
    } else {
        log.add(format!(
            "{} attacks {} but misses. ({} vs AC {})",
            attacker, defender, result.total, result.target_ac
        ));
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
