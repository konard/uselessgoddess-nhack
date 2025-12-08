//! D&D 5e SRD rules implementation.
//!
//! This module implements core D&D 5e mechanics including:
//! - Ability scores and modifiers
//! - Dice rolling system
//! - Armor Class calculations
//! - Combat mechanics (attack rolls, damage)
//! - Equipment (weapons, armor)
//! - Status effects and conditions
//! - Spellcasting system

pub mod ability;
pub mod armor;
pub mod combat;
pub mod components;
pub mod dice;
pub mod effects;
pub mod monsters;
pub mod spells;
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
        effects::plugin,
        spells::plugin,
    ));
}

// Re-export key types for external use.
// Most types should be accessed via their submodule (e.g., dnd::ability::Ability).
pub use ability::AbilityScores;
pub use components::ArmorClass;
pub use dice::DiceRoller;
pub use effects::{DamageModifiers, StatusEffects};
pub use weapons::DamageType;
