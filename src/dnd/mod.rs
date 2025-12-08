//! D&D 5e SRD rules implementation.
//!
//! This module implements core D&D 5e mechanics including:
//! - Ability scores and modifiers
//! - Dice rolling system
//! - Armor Class calculations
//! - Combat mechanics (attack rolls, damage)
//! - Equipment (weapons, armor)
//! - Character data (races, classes)

pub mod ability;
pub mod armor;
pub mod combat;
pub mod components;
pub mod dice;
pub mod monsters;
pub mod weapons;

use bevy::prelude::*;

/// Plugin for D&D 5e rules systems.
pub fn plugin(app: &mut App) {
    app.add_plugins((
        ability::plugin,
        components::plugin,
        dice::plugin,
        weapons::plugin,
        armor::plugin,
        combat::plugin,
        monsters::plugin,
    ));
}

// Re-export commonly used types
pub use ability::{Ability, AbilityScores};
pub use armor::{Armor, ArmorCategory, ArmorType};
pub use combat::AttackResult;
pub use weapons::DamageType;
pub use components::{ArmorClass, CharacterSheet, Equipment, Proficiencies, Skills};
pub use dice::{Dice, DiceRoll, DiceType};
pub use monsters::{ChallengeRating, MonsterTemplate};
pub use weapons::{Weapon, WeaponCategory, WeaponProperty, WeaponType};
