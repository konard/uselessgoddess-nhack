//! D&D 5e Monster Templates.
//!
//! Provides SRD monster stat blocks that can be used as templates
//! for spawning monsters. The AI "Flavor Engine" can customize
//! names and descriptions while the mechanical stats remain fixed.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ability::AbilityScores;
use super::armor::ArmorType;
use super::dice::{Dice, DiceType};
use super::weapons::{DamageType, WeaponType};

/// Plugin for monster systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<MonsterDatabase>()
        .register_type::<MonsterTemplate>();
}

/// Challenge Rating for monsters.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ChallengeRating {
    Zero,
    Eighth,   // 1/8
    Quarter,  // 1/4
    Half,     // 1/2
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    // Higher CRs can be added as needed
}

impl ChallengeRating {
    /// Get the XP value for this CR.
    pub fn xp(&self) -> i32 {
        match self {
            ChallengeRating::Zero => 0,
            ChallengeRating::Eighth => 25,
            ChallengeRating::Quarter => 50,
            ChallengeRating::Half => 100,
            ChallengeRating::One => 200,
            ChallengeRating::Two => 450,
            ChallengeRating::Three => 700,
            ChallengeRating::Four => 1100,
            ChallengeRating::Five => 1800,
            ChallengeRating::Six => 2300,
            ChallengeRating::Seven => 2900,
            ChallengeRating::Eight => 3900,
            ChallengeRating::Nine => 5000,
            ChallengeRating::Ten => 5900,
        }
    }

    /// Get the proficiency bonus for this CR.
    pub fn proficiency_bonus(&self) -> i32 {
        match self {
            ChallengeRating::Zero
            | ChallengeRating::Eighth
            | ChallengeRating::Quarter
            | ChallengeRating::Half
            | ChallengeRating::One
            | ChallengeRating::Two
            | ChallengeRating::Three
            | ChallengeRating::Four => 2,
            ChallengeRating::Five
            | ChallengeRating::Six
            | ChallengeRating::Seven
            | ChallengeRating::Eight => 3,
            ChallengeRating::Nine | ChallengeRating::Ten => 4,
        }
    }

    /// Display string for the CR.
    pub fn display(&self) -> &'static str {
        match self {
            ChallengeRating::Zero => "0",
            ChallengeRating::Eighth => "1/8",
            ChallengeRating::Quarter => "1/4",
            ChallengeRating::Half => "1/2",
            ChallengeRating::One => "1",
            ChallengeRating::Two => "2",
            ChallengeRating::Three => "3",
            ChallengeRating::Four => "4",
            ChallengeRating::Five => "5",
            ChallengeRating::Six => "6",
            ChallengeRating::Seven => "7",
            ChallengeRating::Eight => "8",
            ChallengeRating::Nine => "9",
            ChallengeRating::Ten => "10",
        }
    }
}

/// Size category for creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize, Default)]
pub enum CreatureSize {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Huge,
    Gargantuan,
}

impl CreatureSize {
    /// Get the hit die type for this size.
    pub fn hit_die(&self) -> DiceType {
        match self {
            CreatureSize::Tiny => DiceType::D4,
            CreatureSize::Small => DiceType::D6,
            CreatureSize::Medium => DiceType::D8,
            CreatureSize::Large => DiceType::D10,
            CreatureSize::Huge => DiceType::D12,
            CreatureSize::Gargantuan => DiceType::D20,
        }
    }
}

/// Creature type (for damage resistances/immunities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum CreatureType {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,
}

/// Known monster types from the SRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum MonsterType {
    // CR 1/4
    Skeleton,
    Zombie,
    Goblin,
    Kobold,
    // CR 1/2
    Orc,
    Hobgoblin,
    // CR 1
    Ghoul,
    Bugbear,
    // CR 2
    Ogre,
    Ghast,
    // Higher CR
    Troll,
    Wraith,
    // Demons (various CR)
    Imp,
    Quasit,
}

/// Template for spawning a monster with SRD stats.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MonsterTemplate {
    /// Base monster type.
    pub monster_type: MonsterType,
    /// AI-generated custom name (e.g., "Snaggletooth the Rotting").
    pub custom_name: Option<String>,
    /// AI-generated visual description.
    pub visual_description: Option<String>,
    /// AI-generated weapon flavor (e.g., "Rusted Pickaxe").
    pub weapon_flavor: Option<String>,
    /// AI-generated armor flavor (e.g., "Moldy Leather Armor").
    pub armor_flavor: Option<String>,
}

impl MonsterTemplate {
    /// Create a new monster template.
    pub fn new(monster_type: MonsterType) -> Self {
        Self {
            monster_type,
            custom_name: None,
            visual_description: None,
            weapon_flavor: None,
            armor_flavor: None,
        }
    }

    /// Set a custom name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.custom_name = Some(name.into());
        self
    }

    /// Set a visual description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.visual_description = Some(desc.into());
        self
    }

    /// Get the display name (custom or default).
    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.custom_name {
            name.clone()
        } else {
            self.monster_type.data().name.to_string()
        }
    }

    /// Get the base monster data.
    pub fn data(&self) -> &'static MonsterData {
        self.monster_type.data()
    }
}

/// Static monster data from the SRD.
#[derive(Debug, Clone)]
pub struct MonsterData {
    pub name: &'static str,
    pub size: CreatureSize,
    pub creature_type: CreatureType,
    pub challenge_rating: ChallengeRating,
    pub ability_scores: AbilityScores,
    pub armor_class: i32,
    pub armor_type: Option<ArmorType>,
    pub hit_dice: Dice,
    pub speed: i32,
    pub primary_weapon: WeaponType,
    pub damage_vulnerabilities: &'static [DamageType],
    pub damage_resistances: &'static [DamageType],
    pub damage_immunities: &'static [DamageType],
    pub darkvision: i32,
    pub passive_perception: i32,
}

impl MonsterType {
    /// Get the SRD data for this monster type.
    pub fn data(&self) -> &'static MonsterData {
        match self {
            MonsterType::Skeleton => &SKELETON,
            MonsterType::Zombie => &ZOMBIE,
            MonsterType::Goblin => &GOBLIN,
            MonsterType::Kobold => &KOBOLD,
            MonsterType::Orc => &ORC,
            MonsterType::Hobgoblin => &HOBGOBLIN,
            MonsterType::Ghoul => &GHOUL,
            MonsterType::Bugbear => &BUGBEAR,
            MonsterType::Ogre => &OGRE,
            MonsterType::Ghast => &GHAST,
            MonsterType::Troll => &TROLL,
            MonsterType::Wraith => &WRAITH,
            MonsterType::Imp => &IMP,
            MonsterType::Quasit => &QUASIT,
        }
    }

    /// Get a glyph for TUI rendering.
    pub fn glyph(&self) -> char {
        match self {
            MonsterType::Skeleton => 's',
            MonsterType::Zombie => 'Z',
            MonsterType::Goblin => 'g',
            MonsterType::Kobold => 'k',
            MonsterType::Orc => 'o',
            MonsterType::Hobgoblin => 'H',
            MonsterType::Ghoul => 'G',
            MonsterType::Bugbear => 'B',
            MonsterType::Ogre => 'O',
            MonsterType::Ghast => 'G',
            MonsterType::Troll => 'T',
            MonsterType::Wraith => 'W',
            MonsterType::Imp => 'i',
            MonsterType::Quasit => 'q',
        }
    }
}

// SRD Monster Stat Blocks

static SKELETON: MonsterData = MonsterData {
    name: "Skeleton",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Undead,
    challenge_rating: ChallengeRating::Quarter,
    ability_scores: AbilityScores {
        strength: 10,
        dexterity: 14,
        constitution: 15,
        intelligence: 6,
        wisdom: 8,
        charisma: 5,
    },
    armor_class: 13,
    armor_type: None, // Armor scraps
    hit_dice: Dice {
        count: 2,
        die_type: DiceType::D8,
        modifier: 4, // 2 * CON mod (+2)
    },
    speed: 30,
    primary_weapon: WeaponType::Shortsword,
    damage_vulnerabilities: &[DamageType::Bludgeoning],
    damage_resistances: &[],
    damage_immunities: &[DamageType::Poison],
    darkvision: 60,
    passive_perception: 9,
};

static ZOMBIE: MonsterData = MonsterData {
    name: "Zombie",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Undead,
    challenge_rating: ChallengeRating::Quarter,
    ability_scores: AbilityScores {
        strength: 13,
        dexterity: 6,
        constitution: 16,
        intelligence: 3,
        wisdom: 6,
        charisma: 5,
    },
    armor_class: 8,
    armor_type: None,
    hit_dice: Dice {
        count: 3,
        die_type: DiceType::D8,
        modifier: 9, // 3 * CON mod (+3)
    },
    speed: 20,
    primary_weapon: WeaponType::Unarmed, // Slam attack
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[DamageType::Poison],
    darkvision: 60,
    passive_perception: 8,
};

static GOBLIN: MonsterData = MonsterData {
    name: "Goblin",
    size: CreatureSize::Small,
    creature_type: CreatureType::Humanoid,
    challenge_rating: ChallengeRating::Quarter,
    ability_scores: AbilityScores {
        strength: 8,
        dexterity: 14,
        constitution: 10,
        intelligence: 10,
        wisdom: 8,
        charisma: 8,
    },
    armor_class: 15, // Leather armor + shield
    armor_type: Some(ArmorType::Leather),
    hit_dice: Dice {
        count: 2,
        die_type: DiceType::D6,
        modifier: 0,
    },
    speed: 30,
    primary_weapon: WeaponType::Scimitar,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 9,
};

static KOBOLD: MonsterData = MonsterData {
    name: "Kobold",
    size: CreatureSize::Small,
    creature_type: CreatureType::Humanoid,
    challenge_rating: ChallengeRating::Eighth,
    ability_scores: AbilityScores {
        strength: 7,
        dexterity: 15,
        constitution: 9,
        intelligence: 8,
        wisdom: 7,
        charisma: 8,
    },
    armor_class: 12,
    armor_type: None,
    hit_dice: Dice {
        count: 2,
        die_type: DiceType::D6,
        modifier: -2,
    },
    speed: 30,
    primary_weapon: WeaponType::Dagger,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 8,
};

static ORC: MonsterData = MonsterData {
    name: "Orc",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Humanoid,
    challenge_rating: ChallengeRating::Half,
    ability_scores: AbilityScores {
        strength: 16,
        dexterity: 12,
        constitution: 16,
        intelligence: 7,
        wisdom: 11,
        charisma: 10,
    },
    armor_class: 13, // Hide armor
    armor_type: Some(ArmorType::Hide),
    hit_dice: Dice {
        count: 2,
        die_type: DiceType::D8,
        modifier: 6, // 2 * CON mod (+3)
    },
    speed: 30,
    primary_weapon: WeaponType::Greataxe,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 10,
};

static HOBGOBLIN: MonsterData = MonsterData {
    name: "Hobgoblin",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Humanoid,
    challenge_rating: ChallengeRating::Half,
    ability_scores: AbilityScores {
        strength: 13,
        dexterity: 12,
        constitution: 12,
        intelligence: 10,
        wisdom: 10,
        charisma: 9,
    },
    armor_class: 18, // Chain mail + shield
    armor_type: Some(ArmorType::ChainMail),
    hit_dice: Dice {
        count: 2,
        die_type: DiceType::D8,
        modifier: 2,
    },
    speed: 30,
    primary_weapon: WeaponType::Longsword,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 10,
};

static GHOUL: MonsterData = MonsterData {
    name: "Ghoul",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Undead,
    challenge_rating: ChallengeRating::One,
    ability_scores: AbilityScores {
        strength: 13,
        dexterity: 15,
        constitution: 10,
        intelligence: 7,
        wisdom: 10,
        charisma: 6,
    },
    armor_class: 12,
    armor_type: None,
    hit_dice: Dice {
        count: 5,
        die_type: DiceType::D8,
        modifier: 0,
    },
    speed: 30,
    primary_weapon: WeaponType::Unarmed, // Claws
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[DamageType::Poison],
    darkvision: 60,
    passive_perception: 10,
};

static BUGBEAR: MonsterData = MonsterData {
    name: "Bugbear",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Humanoid,
    challenge_rating: ChallengeRating::One,
    ability_scores: AbilityScores {
        strength: 15,
        dexterity: 14,
        constitution: 13,
        intelligence: 8,
        wisdom: 11,
        charisma: 9,
    },
    armor_class: 16, // Hide armor + shield
    armor_type: Some(ArmorType::Hide),
    hit_dice: Dice {
        count: 5,
        die_type: DiceType::D8,
        modifier: 5,
    },
    speed: 30,
    primary_weapon: WeaponType::Morningstar,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 10,
};

static OGRE: MonsterData = MonsterData {
    name: "Ogre",
    size: CreatureSize::Large,
    creature_type: CreatureType::Giant,
    challenge_rating: ChallengeRating::Two,
    ability_scores: AbilityScores {
        strength: 19,
        dexterity: 8,
        constitution: 16,
        intelligence: 5,
        wisdom: 7,
        charisma: 7,
    },
    armor_class: 11, // Hide armor
    armor_type: Some(ArmorType::Hide),
    hit_dice: Dice {
        count: 7,
        die_type: DiceType::D10,
        modifier: 21, // 7 * CON mod (+3)
    },
    speed: 40,
    primary_weapon: WeaponType::Greatclub,
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 8,
};

static GHAST: MonsterData = MonsterData {
    name: "Ghast",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Undead,
    challenge_rating: ChallengeRating::Two,
    ability_scores: AbilityScores {
        strength: 16,
        dexterity: 17,
        constitution: 10,
        intelligence: 11,
        wisdom: 10,
        charisma: 8,
    },
    armor_class: 13,
    armor_type: None,
    hit_dice: Dice {
        count: 8,
        die_type: DiceType::D8,
        modifier: 0,
    },
    speed: 30,
    primary_weapon: WeaponType::Unarmed, // Claws
    damage_vulnerabilities: &[],
    damage_resistances: &[DamageType::Necrotic],
    damage_immunities: &[DamageType::Poison],
    darkvision: 60,
    passive_perception: 10,
};

static TROLL: MonsterData = MonsterData {
    name: "Troll",
    size: CreatureSize::Large,
    creature_type: CreatureType::Giant,
    challenge_rating: ChallengeRating::Five,
    ability_scores: AbilityScores {
        strength: 18,
        dexterity: 13,
        constitution: 20,
        intelligence: 7,
        wisdom: 9,
        charisma: 7,
    },
    armor_class: 15, // Natural armor
    armor_type: None,
    hit_dice: Dice {
        count: 8,
        die_type: DiceType::D10,
        modifier: 40, // 8 * CON mod (+5)
    },
    speed: 30,
    primary_weapon: WeaponType::Unarmed, // Claws
    damage_vulnerabilities: &[],
    damage_resistances: &[],
    damage_immunities: &[],
    darkvision: 60,
    passive_perception: 12,
};

static WRAITH: MonsterData = MonsterData {
    name: "Wraith",
    size: CreatureSize::Medium,
    creature_type: CreatureType::Undead,
    challenge_rating: ChallengeRating::Five,
    ability_scores: AbilityScores {
        strength: 6,
        dexterity: 16,
        constitution: 16,
        intelligence: 12,
        wisdom: 14,
        charisma: 15,
    },
    armor_class: 13,
    armor_type: None,
    hit_dice: Dice {
        count: 9,
        die_type: DiceType::D8,
        modifier: 27, // 9 * CON mod (+3)
    },
    speed: 60, // Flying
    primary_weapon: WeaponType::Unarmed, // Life Drain
    damage_vulnerabilities: &[],
    damage_resistances: &[
        DamageType::Acid,
        DamageType::Cold,
        DamageType::Fire,
        DamageType::Lightning,
        DamageType::Thunder,
    ],
    damage_immunities: &[DamageType::Necrotic, DamageType::Poison],
    darkvision: 60,
    passive_perception: 12,
};

static IMP: MonsterData = MonsterData {
    name: "Imp",
    size: CreatureSize::Tiny,
    creature_type: CreatureType::Fiend,
    challenge_rating: ChallengeRating::One,
    ability_scores: AbilityScores {
        strength: 6,
        dexterity: 17,
        constitution: 13,
        intelligence: 11,
        wisdom: 12,
        charisma: 14,
    },
    armor_class: 13,
    armor_type: None,
    hit_dice: Dice {
        count: 3,
        die_type: DiceType::D4,
        modifier: 3,
    },
    speed: 20,
    primary_weapon: WeaponType::Dagger, // Sting
    damage_vulnerabilities: &[],
    damage_resistances: &[DamageType::Cold],
    damage_immunities: &[DamageType::Fire, DamageType::Poison],
    darkvision: 120,
    passive_perception: 11,
};

static QUASIT: MonsterData = MonsterData {
    name: "Quasit",
    size: CreatureSize::Tiny,
    creature_type: CreatureType::Fiend,
    challenge_rating: ChallengeRating::One,
    ability_scores: AbilityScores {
        strength: 5,
        dexterity: 17,
        constitution: 10,
        intelligence: 7,
        wisdom: 10,
        charisma: 10,
    },
    armor_class: 13,
    armor_type: None,
    hit_dice: Dice {
        count: 3,
        die_type: DiceType::D4,
        modifier: 0,
    },
    speed: 40,
    primary_weapon: WeaponType::Dagger, // Claws
    damage_vulnerabilities: &[],
    damage_resistances: &[
        DamageType::Cold,
        DamageType::Fire,
        DamageType::Lightning,
    ],
    damage_immunities: &[DamageType::Poison],
    darkvision: 120,
    passive_perception: 10,
};

/// Database of all monster types for lookup.
#[derive(Resource)]
pub struct MonsterDatabase {
    monsters: Vec<MonsterType>,
}

impl Default for MonsterDatabase {
    fn default() -> Self {
        Self {
            monsters: vec![
                MonsterType::Skeleton,
                MonsterType::Zombie,
                MonsterType::Goblin,
                MonsterType::Kobold,
                MonsterType::Orc,
                MonsterType::Hobgoblin,
                MonsterType::Ghoul,
                MonsterType::Bugbear,
                MonsterType::Ogre,
                MonsterType::Ghast,
                MonsterType::Troll,
                MonsterType::Wraith,
                MonsterType::Imp,
                MonsterType::Quasit,
            ],
        }
    }
}

impl MonsterDatabase {
    /// Find monsters by CR.
    pub fn find_by_cr(&self, cr: ChallengeRating) -> Vec<MonsterType> {
        self.monsters
            .iter()
            .filter(|m| m.data().challenge_rating == cr)
            .copied()
            .collect()
    }

    /// Find a monster by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Option<MonsterType> {
        let name_lower = name.to_lowercase();
        self.monsters
            .iter()
            .find(|m| m.data().name.to_lowercase() == name_lower)
            .copied()
    }

    /// Get all monsters.
    pub fn all(&self) -> &[MonsterType] {
        &self.monsters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_data() {
        let skeleton = MonsterType::Skeleton.data();
        assert_eq!(skeleton.name, "Skeleton");
        assert_eq!(skeleton.challenge_rating, ChallengeRating::Quarter);
        assert_eq!(skeleton.armor_class, 13);
        assert!(skeleton.damage_vulnerabilities.contains(&DamageType::Bludgeoning));
        assert!(skeleton.damage_immunities.contains(&DamageType::Poison));
    }

    #[test]
    fn test_monster_database() {
        let db = MonsterDatabase::default();
        let cr_quarter = db.find_by_cr(ChallengeRating::Quarter);
        assert!(cr_quarter.len() >= 3); // Skeleton, Zombie, Goblin, Kobold
    }

    #[test]
    fn test_hit_dice() {
        let skeleton = MonsterType::Skeleton.data();
        // 2d8 + 4 (CON +2 per die) = avg 13 HP
        assert_eq!(skeleton.hit_dice.count, 2);
        assert_eq!(skeleton.hit_dice.die_type, DiceType::D8);
        assert_eq!(skeleton.hit_dice.modifier, 4);
    }
}
