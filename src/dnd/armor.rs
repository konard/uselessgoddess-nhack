//! D&D 5e Armor System.
//!
//! Implements the armor system from the SRD including:
//! - Light, Medium, and Heavy armor
//! - Shields
//! - AC calculations

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for armor systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<ArmorDatabase>()
        .register_type::<Armor>()
        .register_type::<ArmorType>();
}

/// Armor proficiency category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ArmorCategory {
    /// AC = base + full DEX modifier
    Light,
    /// AC = base + DEX modifier (max +2)
    Medium,
    /// AC = base (no DEX modifier)
    Heavy,
    /// Adds +2 to AC
    Shield,
}

/// Known armor types from the SRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ArmorType {
    // Unarmored
    Unarmored,
    // Light Armor
    Padded,
    Leather,
    StuddedLeather,
    // Medium Armor
    Hide,
    ChainShirt,
    ScaleMail,
    Breastplate,
    HalfPlate,
    // Heavy Armor
    RingMail,
    ChainMail,
    Splint,
    Plate,
    // Shield
    Shield,
}

impl ArmorType {
    /// Get the base armor data for this type.
    pub fn base_data(&self) -> ArmorData {
        match self {
            ArmorType::Unarmored => ArmorData {
                name: "Unarmored",
                category: ArmorCategory::Light,
                base_ac: 10,
                dex_bonus: DexBonus::Full,
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            // Light Armor
            ArmorType::Padded => ArmorData {
                name: "Padded",
                category: ArmorCategory::Light,
                base_ac: 11,
                dex_bonus: DexBonus::Full,
                strength_requirement: None,
                stealth_disadvantage: true,
            },
            ArmorType::Leather => ArmorData {
                name: "Leather",
                category: ArmorCategory::Light,
                base_ac: 11,
                dex_bonus: DexBonus::Full,
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            ArmorType::StuddedLeather => ArmorData {
                name: "Studded Leather",
                category: ArmorCategory::Light,
                base_ac: 12,
                dex_bonus: DexBonus::Full,
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            // Medium Armor
            ArmorType::Hide => ArmorData {
                name: "Hide",
                category: ArmorCategory::Medium,
                base_ac: 12,
                dex_bonus: DexBonus::Max(2),
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            ArmorType::ChainShirt => ArmorData {
                name: "Chain Shirt",
                category: ArmorCategory::Medium,
                base_ac: 13,
                dex_bonus: DexBonus::Max(2),
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            ArmorType::ScaleMail => ArmorData {
                name: "Scale Mail",
                category: ArmorCategory::Medium,
                base_ac: 14,
                dex_bonus: DexBonus::Max(2),
                strength_requirement: None,
                stealth_disadvantage: true,
            },
            ArmorType::Breastplate => ArmorData {
                name: "Breastplate",
                category: ArmorCategory::Medium,
                base_ac: 14,
                dex_bonus: DexBonus::Max(2),
                strength_requirement: None,
                stealth_disadvantage: false,
            },
            ArmorType::HalfPlate => ArmorData {
                name: "Half Plate",
                category: ArmorCategory::Medium,
                base_ac: 15,
                dex_bonus: DexBonus::Max(2),
                strength_requirement: None,
                stealth_disadvantage: true,
            },
            // Heavy Armor
            ArmorType::RingMail => ArmorData {
                name: "Ring Mail",
                category: ArmorCategory::Heavy,
                base_ac: 14,
                dex_bonus: DexBonus::None,
                strength_requirement: None,
                stealth_disadvantage: true,
            },
            ArmorType::ChainMail => ArmorData {
                name: "Chain Mail",
                category: ArmorCategory::Heavy,
                base_ac: 16,
                dex_bonus: DexBonus::None,
                strength_requirement: Some(13),
                stealth_disadvantage: true,
            },
            ArmorType::Splint => ArmorData {
                name: "Splint",
                category: ArmorCategory::Heavy,
                base_ac: 17,
                dex_bonus: DexBonus::None,
                strength_requirement: Some(15),
                stealth_disadvantage: true,
            },
            ArmorType::Plate => ArmorData {
                name: "Plate",
                category: ArmorCategory::Heavy,
                base_ac: 18,
                dex_bonus: DexBonus::None,
                strength_requirement: Some(15),
                stealth_disadvantage: true,
            },
            // Shield
            ArmorType::Shield => ArmorData {
                name: "Shield",
                category: ArmorCategory::Shield,
                base_ac: 2, // Adds to existing AC
                dex_bonus: DexBonus::None,
                strength_requirement: None,
                stealth_disadvantage: false,
            },
        }
    }
}

/// How DEX modifier applies to this armor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexBonus {
    /// No DEX bonus (heavy armor).
    None,
    /// Full DEX bonus (light armor, unarmored).
    Full,
    /// DEX bonus with a maximum (medium armor).
    Max(i32),
}

/// Static armor data from the SRD.
#[derive(Debug, Clone)]
pub struct ArmorData {
    pub name: &'static str,
    pub category: ArmorCategory,
    pub base_ac: i32,
    pub dex_bonus: DexBonus,
    pub strength_requirement: Option<i32>,
    pub stealth_disadvantage: bool,
}

impl ArmorData {
    /// Calculate AC with this armor and the given DEX modifier.
    pub fn calculate_ac(&self, dex_modifier: i32) -> i32 {
        let dex_bonus = match self.dex_bonus {
            DexBonus::None => 0,
            DexBonus::Full => dex_modifier,
            DexBonus::Max(max) => dex_modifier.min(max),
        };

        self.base_ac + dex_bonus
    }
}

/// An armor instance with potential customization.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Armor {
    /// Base armor type.
    pub armor_type: ArmorType,
    /// Custom name (e.g., "Moldy Leather Armor").
    pub flavor_name: Option<String>,
    /// Magic bonus to AC.
    pub magic_bonus: i32,
    /// Whether this is equipped.
    pub equipped: bool,
}

impl Armor {
    /// Create new armor of the given type.
    pub fn new(armor_type: ArmorType) -> Self {
        Self {
            armor_type,
            flavor_name: None,
            magic_bonus: 0,
            equipped: false,
        }
    }

    /// Create armor with a custom flavor name.
    pub fn with_flavor_name(mut self, name: impl Into<String>) -> Self {
        self.flavor_name = Some(name.into());
        self
    }

    /// Create magic armor with a bonus.
    pub fn with_magic_bonus(mut self, bonus: i32) -> Self {
        self.magic_bonus = bonus;
        self
    }

    /// Set the equipped state.
    pub fn equipped(mut self, equipped: bool) -> Self {
        self.equipped = equipped;
        self
    }

    /// Get the base armor data.
    pub fn data(&self) -> ArmorData {
        self.armor_type.base_data()
    }

    /// Get the display name.
    pub fn display_name(&self) -> String {
        if let Some(ref flavor) = self.flavor_name {
            flavor.clone()
        } else {
            self.armor_type.base_data().name.to_string()
        }
    }

    /// Calculate AC with this armor (including magic bonus).
    pub fn calculate_ac(&self, dex_modifier: i32) -> i32 {
        self.data().calculate_ac(dex_modifier) + self.magic_bonus
    }
}

/// Database of all armor types for lookup and AI mapping.
#[derive(Resource)]
pub struct ArmorDatabase {
    armors: Vec<ArmorType>,
}

impl Default for ArmorDatabase {
    fn default() -> Self {
        Self {
            armors: vec![
                ArmorType::Unarmored,
                // Light
                ArmorType::Padded,
                ArmorType::Leather,
                ArmorType::StuddedLeather,
                // Medium
                ArmorType::Hide,
                ArmorType::ChainShirt,
                ArmorType::ScaleMail,
                ArmorType::Breastplate,
                ArmorType::HalfPlate,
                // Heavy
                ArmorType::RingMail,
                ArmorType::ChainMail,
                ArmorType::Splint,
                ArmorType::Plate,
                // Shield
                ArmorType::Shield,
            ],
        }
    }
}

impl ArmorDatabase {
    /// Find an armor type by name (case-insensitive, partial match).
    pub fn find_by_name(&self, name: &str) -> Option<ArmorType> {
        let name_lower = name.to_lowercase();

        // First try exact match
        for &armor in &self.armors {
            let data = armor.base_data();
            if data.name.to_lowercase() == name_lower {
                return Some(armor);
            }
        }

        // Then try partial match
        for &armor in &self.armors {
            let data = armor.base_data();
            if data.name.to_lowercase().contains(&name_lower)
                || name_lower.contains(&data.name.to_lowercase())
            {
                return Some(armor);
            }
        }

        None
    }

    /// Get all armors.
    pub fn all(&self) -> &[ArmorType] {
        &self.armors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unarmored_ac() {
        let armor = ArmorType::Unarmored.base_data();
        assert_eq!(armor.calculate_ac(2), 12); // 10 + 2 DEX
        assert_eq!(armor.calculate_ac(-1), 9); // 10 - 1 DEX
    }

    #[test]
    fn test_light_armor_ac() {
        let leather = ArmorType::Leather.base_data();
        assert_eq!(leather.calculate_ac(3), 14); // 11 + 3 DEX
    }

    #[test]
    fn test_medium_armor_ac() {
        let chain_shirt = ArmorType::ChainShirt.base_data();
        assert_eq!(chain_shirt.calculate_ac(2), 15); // 13 + 2 DEX (max)
        assert_eq!(chain_shirt.calculate_ac(4), 15); // 13 + 2 DEX (capped at 2)
        assert_eq!(chain_shirt.calculate_ac(1), 14); // 13 + 1 DEX
    }

    #[test]
    fn test_heavy_armor_ac() {
        let plate = ArmorType::Plate.base_data();
        assert_eq!(plate.calculate_ac(0), 18); // 18 flat
        assert_eq!(plate.calculate_ac(5), 18); // DEX doesn't apply
        assert_eq!(plate.calculate_ac(-2), 18); // DEX doesn't apply (even negative)
    }

    #[test]
    fn test_armor_database() {
        let db = ArmorDatabase::default();
        assert!(db.find_by_name("Plate").is_some());
        assert!(db.find_by_name("leather").is_some()); // Case insensitive
        assert!(db.find_by_name("xyz").is_none());
    }
}
