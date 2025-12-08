//! D&D 5e Weapons.
//!
//! Implements the weapon system from the SRD including:
//! - Simple and Martial weapons
//! - Melee and Ranged weapons
//! - Weapon properties (Finesse, Light, Heavy, etc.)
//! - Damage types (Slashing, Piercing, Bludgeoning)

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ability::Ability;
use super::dice::{Dice, DiceType};

/// Plugin for weapon systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<WeaponDatabase>()
        .register_type::<Weapon>()
        .register_type::<WeaponType>();
}

/// Weapon proficiency category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum WeaponCategory {
    Simple,
    Martial,
    Improvised,
}

/// Melee vs Ranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum WeaponRange {
    Melee,
    Ranged,
}

/// Weapon property flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum WeaponProperty {
    /// Can use DEX instead of STR for attack/damage.
    Finesse,
    /// Can be dual-wielded.
    Light,
    /// Small creatures have disadvantage; requires STR 13+.
    Heavy,
    /// +5 ft reach.
    Reach,
    /// Can be thrown.
    Thrown,
    /// Requires two hands.
    TwoHanded,
    /// Can use one or two hands (with different damage).
    Versatile,
    /// Uses ammunition.
    Ammunition,
    /// Requires reloading after each shot.
    Loading,
    /// Has a special rule.
    Special,
}

/// Types of damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize, Default)]
pub enum DamageType {
    #[default]
    Bludgeoning,
    Piercing,
    Slashing,
    // Elemental damage types
    Fire,
    Cold,
    Lightning,
    Thunder,
    Acid,
    Poison,
    // Other damage types
    Radiant,
    Necrotic,
    Force,
    Psychic,
}

impl DamageType {
    pub fn name(&self) -> &'static str {
        match self {
            DamageType::Bludgeoning => "bludgeoning",
            DamageType::Piercing => "piercing",
            DamageType::Slashing => "slashing",
            DamageType::Fire => "fire",
            DamageType::Cold => "cold",
            DamageType::Lightning => "lightning",
            DamageType::Thunder => "thunder",
            DamageType::Acid => "acid",
            DamageType::Poison => "poison",
            DamageType::Radiant => "radiant",
            DamageType::Necrotic => "necrotic",
            DamageType::Force => "force",
            DamageType::Psychic => "psychic",
        }
    }
}

/// Known weapon types from the SRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum WeaponType {
    // Simple Melee
    Club,
    Dagger,
    Greatclub,
    Handaxe,
    Javelin,
    LightHammer,
    Mace,
    Quarterstaff,
    Sickle,
    Spear,
    // Simple Ranged
    LightCrossbow,
    Dart,
    Shortbow,
    Sling,
    // Martial Melee
    Battleaxe,
    Flail,
    Glaive,
    Greataxe,
    Greatsword,
    Halberd,
    Lance,
    Longsword,
    Maul,
    Morningstar,
    Pike,
    Rapier,
    Scimitar,
    Shortsword,
    Trident,
    WarPick,
    Warhammer,
    Whip,
    // Martial Ranged
    Blowgun,
    HandCrossbow,
    HeavyCrossbow,
    Longbow,
    Net,
    // Special
    Unarmed,
    Improvised,
}

impl WeaponType {
    /// Get the base weapon data for this type.
    pub fn base_data(&self) -> WeaponData {
        match self {
            // Simple Melee
            WeaponType::Club => WeaponData {
                name: "Club",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Light],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Dagger => WeaponData {
                name: "Dagger",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Finesse,
                    WeaponProperty::Light,
                    WeaponProperty::Thrown,
                ],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Greatclub => WeaponData {
                name: "Greatclub",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::TwoHanded],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Handaxe => WeaponData {
                name: "Handaxe",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Light, WeaponProperty::Thrown],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Javelin => WeaponData {
                name: "Javelin",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Thrown],
                range_normal: 30,
                range_long: 120,
                versatile_damage: None,
            },
            WeaponType::LightHammer => WeaponData {
                name: "Light Hammer",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Light, WeaponProperty::Thrown],
                range_normal: 20,
                range_long: 60,
                versatile_damage: None,
            },
            WeaponType::Mace => WeaponData {
                name: "Mace",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Bludgeoning,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Quarterstaff => WeaponData {
                name: "Quarterstaff",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Versatile],
                range_normal: 5,
                range_long: 5,
                versatile_damage: Some(Dice::simple(1, DiceType::D8)),
            },
            WeaponType::Sickle => WeaponData {
                name: "Sickle",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Light],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Spear => WeaponData {
                name: "Spear",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Thrown, WeaponProperty::Versatile],
                range_normal: 20,
                range_long: 60,
                versatile_damage: Some(Dice::simple(1, DiceType::D8)),
            },
            // Simple Ranged
            WeaponType::LightCrossbow => WeaponData {
                name: "Light Crossbow",
                category: WeaponCategory::Simple,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Ammunition,
                    WeaponProperty::Loading,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 80,
                range_long: 320,
                versatile_damage: None,
            },
            WeaponType::Dart => WeaponData {
                name: "Dart",
                category: WeaponCategory::Simple,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Finesse, WeaponProperty::Thrown],
                range_normal: 20,
                range_long: 60,
                versatile_damage: None,
            },
            WeaponType::Shortbow => WeaponData {
                name: "Shortbow",
                category: WeaponCategory::Simple,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Ammunition, WeaponProperty::TwoHanded],
                range_normal: 80,
                range_long: 320,
                versatile_damage: None,
            },
            WeaponType::Sling => WeaponData {
                name: "Sling",
                category: WeaponCategory::Simple,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Ammunition],
                range_normal: 30,
                range_long: 120,
                versatile_damage: None,
            },
            // Martial Melee
            WeaponType::Battleaxe => WeaponData {
                name: "Battleaxe",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Versatile],
                range_normal: 5,
                range_long: 5,
                versatile_damage: Some(Dice::simple(1, DiceType::D10)),
            },
            WeaponType::Flail => WeaponData {
                name: "Flail",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Bludgeoning,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Glaive => WeaponData {
                name: "Glaive",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D10),
                damage_type: DamageType::Slashing,
                properties: vec![
                    WeaponProperty::Heavy,
                    WeaponProperty::Reach,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 10,
                range_long: 10,
                versatile_damage: None,
            },
            WeaponType::Greataxe => WeaponData {
                name: "Greataxe",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D12),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Greatsword => WeaponData {
                name: "Greatsword",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(2, DiceType::D6),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Halberd => WeaponData {
                name: "Halberd",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D10),
                damage_type: DamageType::Slashing,
                properties: vec![
                    WeaponProperty::Heavy,
                    WeaponProperty::Reach,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 10,
                range_long: 10,
                versatile_damage: None,
            },
            WeaponType::Lance => WeaponData {
                name: "Lance",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D12),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Reach, WeaponProperty::Special],
                range_normal: 10,
                range_long: 10,
                versatile_damage: None,
            },
            WeaponType::Longsword => WeaponData {
                name: "Longsword",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Versatile],
                range_normal: 5,
                range_long: 5,
                versatile_damage: Some(Dice::simple(1, DiceType::D10)),
            },
            WeaponType::Maul => WeaponData {
                name: "Maul",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(2, DiceType::D6),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Morningstar => WeaponData {
                name: "Morningstar",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Piercing,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Pike => WeaponData {
                name: "Pike",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D10),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Heavy,
                    WeaponProperty::Reach,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 10,
                range_long: 10,
                versatile_damage: None,
            },
            WeaponType::Rapier => WeaponData {
                name: "Rapier",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Finesse],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Scimitar => WeaponData {
                name: "Scimitar",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Finesse, WeaponProperty::Light],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Shortsword => WeaponData {
                name: "Shortsword",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Finesse, WeaponProperty::Light],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Trident => WeaponData {
                name: "Trident",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Thrown, WeaponProperty::Versatile],
                range_normal: 20,
                range_long: 60,
                versatile_damage: Some(Dice::simple(1, DiceType::D8)),
            },
            WeaponType::WarPick => WeaponData {
                name: "War Pick",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Piercing,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Warhammer => WeaponData {
                name: "Warhammer",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Versatile],
                range_normal: 5,
                range_long: 5,
                versatile_damage: Some(Dice::simple(1, DiceType::D10)),
            },
            WeaponType::Whip => WeaponData {
                name: "Whip",
                category: WeaponCategory::Martial,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Slashing,
                properties: vec![WeaponProperty::Finesse, WeaponProperty::Reach],
                range_normal: 10,
                range_long: 10,
                versatile_damage: None,
            },
            // Martial Ranged
            WeaponType::Blowgun => WeaponData {
                name: "Blowgun",
                category: WeaponCategory::Martial,
                range: WeaponRange::Ranged,
                damage: Dice::new(0, DiceType::D4, 1), // 1 damage
                damage_type: DamageType::Piercing,
                properties: vec![WeaponProperty::Ammunition, WeaponProperty::Loading],
                range_normal: 25,
                range_long: 100,
                versatile_damage: None,
            },
            WeaponType::HandCrossbow => WeaponData {
                name: "Hand Crossbow",
                category: WeaponCategory::Martial,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D6),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Ammunition,
                    WeaponProperty::Light,
                    WeaponProperty::Loading,
                ],
                range_normal: 30,
                range_long: 120,
                versatile_damage: None,
            },
            WeaponType::HeavyCrossbow => WeaponData {
                name: "Heavy Crossbow",
                category: WeaponCategory::Martial,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D10),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Ammunition,
                    WeaponProperty::Heavy,
                    WeaponProperty::Loading,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 100,
                range_long: 400,
                versatile_damage: None,
            },
            WeaponType::Longbow => WeaponData {
                name: "Longbow",
                category: WeaponCategory::Martial,
                range: WeaponRange::Ranged,
                damage: Dice::simple(1, DiceType::D8),
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Ammunition,
                    WeaponProperty::Heavy,
                    WeaponProperty::TwoHanded,
                ],
                range_normal: 150,
                range_long: 600,
                versatile_damage: None,
            },
            WeaponType::Net => WeaponData {
                name: "Net",
                category: WeaponCategory::Martial,
                range: WeaponRange::Ranged,
                damage: Dice::new(0, DiceType::D4, 0), // No damage
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Thrown, WeaponProperty::Special],
                range_normal: 5,
                range_long: 15,
                versatile_damage: None,
            },
            // Special
            WeaponType::Unarmed => WeaponData {
                name: "Unarmed Strike",
                category: WeaponCategory::Simple,
                range: WeaponRange::Melee,
                damage: Dice::new(0, DiceType::D4, 1), // 1 + STR mod
                damage_type: DamageType::Bludgeoning,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
            WeaponType::Improvised => WeaponData {
                name: "Improvised Weapon",
                category: WeaponCategory::Improvised,
                range: WeaponRange::Melee,
                damage: Dice::simple(1, DiceType::D4),
                damage_type: DamageType::Bludgeoning,
                properties: vec![],
                range_normal: 5,
                range_long: 5,
                versatile_damage: None,
            },
        }
    }
}

/// Static weapon data from the SRD.
#[derive(Debug, Clone)]
pub struct WeaponData {
    pub name: &'static str,
    pub category: WeaponCategory,
    pub range: WeaponRange,
    pub damage: Dice,
    pub damage_type: DamageType,
    pub properties: Vec<WeaponProperty>,
    pub range_normal: i32,
    pub range_long: i32,
    pub versatile_damage: Option<Dice>,
}

impl WeaponData {
    /// Check if this weapon has a specific property.
    pub fn has_property(&self, property: WeaponProperty) -> bool {
        self.properties.contains(&property)
    }

    /// Get the ability used for attack/damage rolls.
    pub fn attack_ability(&self) -> Ability {
        if self.has_property(WeaponProperty::Finesse) {
            // Finesse weapons can use DEX or STR (caller chooses higher)
            Ability::Dexterity
        } else if self.range == WeaponRange::Ranged {
            Ability::Dexterity
        } else {
            Ability::Strength
        }
    }
}

/// A weapon instance with potential customization (name, bonuses).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Weapon {
    /// Base weapon type.
    pub weapon_type: WeaponType,
    /// Custom name (e.g., "Rusted Pickaxe", "Snaggletooth's Cleaver").
    pub flavor_name: Option<String>,
    /// Magic bonus to attack and damage.
    pub magic_bonus: i32,
    /// Whether this is equipped.
    pub equipped: bool,
}

impl Weapon {
    /// Create a new weapon of the given type.
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon_type,
            flavor_name: None,
            magic_bonus: 0,
            equipped: false,
        }
    }

    /// Create a weapon with a custom flavor name.
    pub fn with_flavor_name(mut self, name: impl Into<String>) -> Self {
        self.flavor_name = Some(name.into());
        self
    }

    /// Create a magic weapon with a bonus.
    pub fn with_magic_bonus(mut self, bonus: i32) -> Self {
        self.magic_bonus = bonus;
        self
    }

    /// Set the equipped state.
    pub fn equipped(mut self, equipped: bool) -> Self {
        self.equipped = equipped;
        self
    }

    /// Get the base weapon data.
    pub fn data(&self) -> WeaponData {
        self.weapon_type.base_data()
    }

    /// Get the display name.
    pub fn display_name(&self) -> String {
        if let Some(ref flavor) = self.flavor_name {
            flavor.clone()
        } else {
            self.weapon_type.base_data().name.to_string()
        }
    }

    /// Get the damage dice including magic bonus.
    pub fn damage_dice(&self) -> Dice {
        let base = self.data().damage;
        Dice::new(base.count, base.die_type, base.modifier + self.magic_bonus)
    }
}

/// Database of all weapon types for lookup and AI mapping.
#[derive(Resource)]
pub struct WeaponDatabase {
    weapons: Vec<WeaponType>,
}

impl Default for WeaponDatabase {
    fn default() -> Self {
        Self {
            weapons: vec![
                // Simple Melee
                WeaponType::Club,
                WeaponType::Dagger,
                WeaponType::Greatclub,
                WeaponType::Handaxe,
                WeaponType::Javelin,
                WeaponType::LightHammer,
                WeaponType::Mace,
                WeaponType::Quarterstaff,
                WeaponType::Sickle,
                WeaponType::Spear,
                // Simple Ranged
                WeaponType::LightCrossbow,
                WeaponType::Dart,
                WeaponType::Shortbow,
                WeaponType::Sling,
                // Martial Melee
                WeaponType::Battleaxe,
                WeaponType::Flail,
                WeaponType::Glaive,
                WeaponType::Greataxe,
                WeaponType::Greatsword,
                WeaponType::Halberd,
                WeaponType::Lance,
                WeaponType::Longsword,
                WeaponType::Maul,
                WeaponType::Morningstar,
                WeaponType::Pike,
                WeaponType::Rapier,
                WeaponType::Scimitar,
                WeaponType::Shortsword,
                WeaponType::Trident,
                WeaponType::WarPick,
                WeaponType::Warhammer,
                WeaponType::Whip,
                // Martial Ranged
                WeaponType::Blowgun,
                WeaponType::HandCrossbow,
                WeaponType::HeavyCrossbow,
                WeaponType::Longbow,
                WeaponType::Net,
            ],
        }
    }
}

impl WeaponDatabase {
    /// Find a weapon type by name (case-insensitive, partial match).
    pub fn find_by_name(&self, name: &str) -> Option<WeaponType> {
        let name_lower = name.to_lowercase();

        // First try exact match
        for &weapon in &self.weapons {
            let data = weapon.base_data();
            if data.name.to_lowercase() == name_lower {
                return Some(weapon);
            }
        }

        // Then try partial match
        for &weapon in &self.weapons {
            let data = weapon.base_data();
            if data.name.to_lowercase().contains(&name_lower)
                || name_lower.contains(&data.name.to_lowercase())
            {
                return Some(weapon);
            }
        }

        None
    }

    /// Get all weapons.
    pub fn all(&self) -> &[WeaponType] {
        &self.weapons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_data() {
        let longsword = WeaponType::Longsword.base_data();
        assert_eq!(longsword.name, "Longsword");
        assert_eq!(longsword.category, WeaponCategory::Martial);
        assert_eq!(longsword.damage.count, 1);
        assert_eq!(longsword.damage.die_type, DiceType::D8);
        assert!(longsword.has_property(WeaponProperty::Versatile));
        assert!(longsword.versatile_damage.is_some());
    }

    #[test]
    fn test_weapon_database() {
        let db = WeaponDatabase::default();
        assert!(db.find_by_name("Longsword").is_some());
        assert!(db.find_by_name("sword").is_some()); // Partial match
        assert!(db.find_by_name("xyz").is_none());
    }

    #[test]
    fn test_finesse_weapon() {
        let rapier = WeaponType::Rapier.base_data();
        assert!(rapier.has_property(WeaponProperty::Finesse));
        assert_eq!(rapier.attack_ability(), Ability::Dexterity);
    }
}
