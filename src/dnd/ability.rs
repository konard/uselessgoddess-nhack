//! D&D 5e Ability Scores and Modifiers.
//!
//! The six ability scores are:
//! - Strength (STR): Physical power, athletic ability
//! - Dexterity (DEX): Agility, reflexes, balance
//! - Constitution (CON): Health, stamina, endurance
//! - Intelligence (INT): Mental acuity, reasoning, memory
//! - Wisdom (WIS): Perception, insight, willpower
//! - Charisma (CHA): Force of personality, social influence

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for ability score systems.
pub fn plugin(app: &mut App) {
    app.register_type::<AbilityScores>();
}

/// The six D&D ability scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    /// Get the short name (3-letter abbreviation).
    pub fn short_name(&self) -> &'static str {
        match self {
            Ability::Strength => "STR",
            Ability::Dexterity => "DEX",
            Ability::Constitution => "CON",
            Ability::Intelligence => "INT",
            Ability::Wisdom => "WIS",
            Ability::Charisma => "CHA",
        }
    }

    /// Get the full name.
    pub fn full_name(&self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Charisma => "Charisma",
        }
    }

    /// Get all abilities in order.
    pub fn all() -> [Ability; 6] {
        [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ]
    }
}

/// Container for all six ability scores.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AbilityScores {
    /// Strength score (1-30, typically 8-18 for characters).
    pub strength: i32,
    /// Dexterity score.
    pub dexterity: i32,
    /// Constitution score.
    pub constitution: i32,
    /// Intelligence score.
    pub intelligence: i32,
    /// Wisdom score.
    pub wisdom: i32,
    /// Charisma score.
    pub charisma: i32,
}

impl Default for AbilityScores {
    fn default() -> Self {
        // Standard array: 15, 14, 13, 12, 10, 8
        // Default distribution for a balanced character
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl AbilityScores {
    /// Create ability scores from individual values.
    pub fn new(str: i32, dex: i32, con: i32, int: i32, wis: i32, cha: i32) -> Self {
        Self {
            strength: str,
            dexterity: dex,
            constitution: con,
            intelligence: int,
            wisdom: wis,
            charisma: cha,
        }
    }

    /// Create ability scores using the standard array (15, 14, 13, 12, 10, 8).
    /// Order: STR, DEX, CON, INT, WIS, CHA
    pub fn standard_array(order: [Ability; 6]) -> Self {
        let values = [15, 14, 13, 12, 10, 8];
        let mut scores = Self::default();

        for (ability, &value) in order.iter().zip(values.iter()) {
            scores.set(*ability, value);
        }

        scores
    }

    /// Get a specific ability score.
    pub fn get(&self, ability: Ability) -> i32 {
        match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        }
    }

    /// Set a specific ability score.
    pub fn set(&mut self, ability: Ability, value: i32) {
        match ability {
            Ability::Strength => self.strength = value,
            Ability::Dexterity => self.dexterity = value,
            Ability::Constitution => self.constitution = value,
            Ability::Intelligence => self.intelligence = value,
            Ability::Wisdom => self.wisdom = value,
            Ability::Charisma => self.charisma = value,
        }
    }

    /// Get the modifier for a specific ability.
    /// Modifier = (score - 10) / 2 (rounded down)
    pub fn modifier(&self, ability: Ability) -> i32 {
        calculate_modifier(self.get(ability))
    }

    /// Get the Strength modifier.
    pub fn str_mod(&self) -> i32 {
        calculate_modifier(self.strength)
    }

    /// Get the Dexterity modifier.
    pub fn dex_mod(&self) -> i32 {
        calculate_modifier(self.dexterity)
    }

    /// Get the Constitution modifier.
    pub fn con_mod(&self) -> i32 {
        calculate_modifier(self.constitution)
    }

    /// Get the Intelligence modifier.
    pub fn int_mod(&self) -> i32 {
        calculate_modifier(self.intelligence)
    }

    /// Get the Wisdom modifier.
    pub fn wis_mod(&self) -> i32 {
        calculate_modifier(self.wisdom)
    }

    /// Get the Charisma modifier.
    pub fn cha_mod(&self) -> i32 {
        calculate_modifier(self.charisma)
    }
}

/// Calculate the ability modifier from an ability score.
///
/// Formula: (score - 10) / 2, rounded down
///
/// # Examples
/// - Score 1 = -5 modifier
/// - Score 10-11 = +0 modifier
/// - Score 18-19 = +4 modifier
/// - Score 30 = +10 modifier
pub fn calculate_modifier(score: i32) -> i32 {
    // Integer division in Rust rounds toward zero, but we need floor division
    // For negative results, we need to round down (toward negative infinity)
    (score - 10).div_euclid(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_modifier_calculation() {
        // Test standard modifier table values
        assert_eq!(calculate_modifier(1), -5);
        assert_eq!(calculate_modifier(2), -4);
        assert_eq!(calculate_modifier(3), -4);
        assert_eq!(calculate_modifier(4), -3);
        assert_eq!(calculate_modifier(5), -3);
        assert_eq!(calculate_modifier(8), -1);
        assert_eq!(calculate_modifier(9), -1);
        assert_eq!(calculate_modifier(10), 0);
        assert_eq!(calculate_modifier(11), 0);
        assert_eq!(calculate_modifier(12), 1);
        assert_eq!(calculate_modifier(13), 1);
        assert_eq!(calculate_modifier(14), 2);
        assert_eq!(calculate_modifier(15), 2);
        assert_eq!(calculate_modifier(16), 3);
        assert_eq!(calculate_modifier(17), 3);
        assert_eq!(calculate_modifier(18), 4);
        assert_eq!(calculate_modifier(19), 4);
        assert_eq!(calculate_modifier(20), 5);
        assert_eq!(calculate_modifier(30), 10);
    }

    #[test]
    fn test_ability_scores() {
        let scores = AbilityScores::new(16, 14, 12, 10, 8, 13);

        assert_eq!(scores.str_mod(), 3);
        assert_eq!(scores.dex_mod(), 2);
        assert_eq!(scores.con_mod(), 1);
        assert_eq!(scores.int_mod(), 0);
        assert_eq!(scores.wis_mod(), -1);
        assert_eq!(scores.cha_mod(), 1);
    }

    #[test]
    fn test_standard_array() {
        let order = [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ];
        let scores = AbilityScores::standard_array(order);

        assert_eq!(scores.strength, 15);
        assert_eq!(scores.dexterity, 14);
        assert_eq!(scores.constitution, 13);
        assert_eq!(scores.intelligence, 12);
        assert_eq!(scores.wisdom, 10);
        assert_eq!(scores.charisma, 8);
    }
}
