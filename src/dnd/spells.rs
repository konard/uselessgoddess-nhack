//! D&D 5e Spell System.
//!
//! Implements a simple spell system including:
//! - Spell slots and casting
//! - Basic SRD spells (cantrips and low-level spells)
//! - Spell effects (damage, healing, conditions)

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ability::Ability;
use super::dice::{Dice, DiceType};
use super::effects::{ActiveEffect, Condition, EffectDuration};
use super::weapons::DamageType;

/// Plugin for spell systems.
pub fn plugin(app: &mut App) {
    app.add_message::<CastSpellMessage>()
        .register_type::<SpellSlots>()
        .register_type::<KnownSpells>()
        .register_type::<SpellcastingAbility>();
}

/// Spell school.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum SpellSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

impl SpellSchool {
    pub fn name(&self) -> &'static str {
        match self {
            SpellSchool::Abjuration => "Abjuration",
            SpellSchool::Conjuration => "Conjuration",
            SpellSchool::Divination => "Divination",
            SpellSchool::Enchantment => "Enchantment",
            SpellSchool::Evocation => "Evocation",
            SpellSchool::Illusion => "Illusion",
            SpellSchool::Necromancy => "Necromancy",
            SpellSchool::Transmutation => "Transmutation",
        }
    }
}

/// Type of target for a spell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum SpellTarget {
    /// Single target.
    Single,
    /// Self only.
    SelfTarget,
    /// Area of effect (radius in feet).
    Area(i32),
    /// Cone (length in feet).
    Cone(i32),
    /// Line (length in feet).
    Line(i32),
    /// Touch range.
    Touch,
}

/// Components of a spell (verbal, somatic, material).
#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct SpellComponents {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<String>,
}

impl SpellComponents {
    pub fn vs() -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: None,
        }
    }

    pub fn v() -> Self {
        Self {
            verbal: true,
            somatic: false,
            material: None,
        }
    }

    pub fn s() -> Self {
        Self {
            verbal: false,
            somatic: true,
            material: None,
        }
    }

    pub fn vsm(material: impl Into<String>) -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: Some(material.into()),
        }
    }
}

/// Type of spell effect.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum SpellEffect {
    /// Direct damage.
    Damage {
        dice: Dice,
        damage_type: DamageType,
        save_for_half: Option<Ability>,
    },
    /// Healing.
    Healing { dice: Dice },
    /// Apply a condition.
    ApplyCondition {
        condition: Condition,
        duration: EffectDuration,
        save: Option<Ability>,
    },
    /// Buff (add temporary HP, etc.).
    TempHp { amount: i32 },
    /// Utility effect (light, etc.).
    Utility { description: String },
}

/// Known spell from the SRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum SpellId {
    // Cantrips (Level 0)
    FireBolt,
    RayOfFrost,
    SacredFlame,
    ChillTouch,
    PoisonSpray,
    ShockingGrasp,
    // Level 1
    BurningHands,
    MagicMissile,
    Shield,
    CureWounds,
    Bless,
    Bane,
    HealingWord,
    // Level 2
    ScorchingRay,
    HoldPerson,
    MistyStep,
    // Level 3
    Fireball,
    LightningBolt,
}

/// Static spell data.
#[derive(Debug, Clone)]
pub struct SpellData {
    pub name: &'static str,
    pub level: u8,
    pub school: SpellSchool,
    pub casting_time: &'static str,
    pub range: i32,
    pub target: SpellTarget,
    pub components: SpellComponents,
    pub duration: &'static str,
    pub description: &'static str,
    pub effects: Vec<SpellEffect>,
}

impl SpellId {
    /// Get the spell data for this spell.
    pub fn data(&self) -> SpellData {
        match self {
            // Cantrips
            SpellId::FireBolt => SpellData {
                name: "Fire Bolt",
                level: 0,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 120,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "A mote of fire streaks toward a creature.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D10),
                    damage_type: DamageType::Fire,
                    save_for_half: None,
                }],
            },
            SpellId::RayOfFrost => SpellData {
                name: "Ray of Frost",
                level: 0,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 60,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "A frigid beam of blue-white light streaks toward a creature.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D8),
                    damage_type: DamageType::Cold,
                    save_for_half: None,
                }],
            },
            SpellId::SacredFlame => SpellData {
                name: "Sacred Flame",
                level: 0,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 60,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "Flame-like radiance descends on a creature.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D8),
                    damage_type: DamageType::Radiant,
                    save_for_half: Some(Ability::Dexterity),
                }],
            },
            SpellId::ChillTouch => SpellData {
                name: "Chill Touch",
                level: 0,
                school: SpellSchool::Necromancy,
                casting_time: "1 action",
                range: 120,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "1 round",
                description: "A ghostly, skeletal hand assails a creature.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D8),
                    damage_type: DamageType::Necrotic,
                    save_for_half: None,
                }],
            },
            SpellId::PoisonSpray => SpellData {
                name: "Poison Spray",
                level: 0,
                school: SpellSchool::Conjuration,
                casting_time: "1 action",
                range: 10,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "A puff of noxious gas sprays at a creature.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D12),
                    damage_type: DamageType::Poison,
                    save_for_half: Some(Ability::Constitution),
                }],
            },
            SpellId::ShockingGrasp => SpellData {
                name: "Shocking Grasp",
                level: 0,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 0,
                target: SpellTarget::Touch,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "Lightning springs from your hand to deliver a shock.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(1, DiceType::D8),
                    damage_type: DamageType::Lightning,
                    save_for_half: None,
                }],
            },
            // Level 1
            SpellId::BurningHands => SpellData {
                name: "Burning Hands",
                level: 1,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 0,
                target: SpellTarget::Cone(15),
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "A thin sheet of flames shoots forth from your fingertips.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(3, DiceType::D6),
                    damage_type: DamageType::Fire,
                    save_for_half: Some(Ability::Dexterity),
                }],
            },
            SpellId::MagicMissile => SpellData {
                name: "Magic Missile",
                level: 1,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 120,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "Three glowing darts of magical force unerringly strike.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::new(3, DiceType::D4, 3), // 3d4+3
                    damage_type: DamageType::Force,
                    save_for_half: None,
                }],
            },
            SpellId::Shield => SpellData {
                name: "Shield",
                level: 1,
                school: SpellSchool::Abjuration,
                casting_time: "1 reaction",
                range: 0,
                target: SpellTarget::SelfTarget,
                components: SpellComponents::vs(),
                duration: "1 round",
                description: "+5 bonus to AC until the start of your next turn.",
                effects: vec![SpellEffect::Utility {
                    description: "+5 AC".to_string(),
                }],
            },
            SpellId::CureWounds => SpellData {
                name: "Cure Wounds",
                level: 1,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 0,
                target: SpellTarget::Touch,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "A creature you touch regains hit points.",
                effects: vec![SpellEffect::Healing {
                    dice: Dice::simple(1, DiceType::D8),
                }],
            },
            SpellId::Bless => SpellData {
                name: "Bless",
                level: 1,
                school: SpellSchool::Enchantment,
                casting_time: "1 action",
                range: 30,
                target: SpellTarget::Single,
                components: SpellComponents::vsm("a sprinkling of holy water"),
                duration: "Concentration, 1 minute",
                description: "Bless up to three creatures with +1d4 to attack rolls and saves.",
                effects: vec![SpellEffect::Utility {
                    description: "+1d4 to attacks and saves".to_string(),
                }],
            },
            SpellId::Bane => SpellData {
                name: "Bane",
                level: 1,
                school: SpellSchool::Enchantment,
                casting_time: "1 action",
                range: 30,
                target: SpellTarget::Single,
                components: SpellComponents::vsm("a drop of blood"),
                duration: "Concentration, 1 minute",
                description: "Up to three creatures must subtract 1d4 from attacks and saves.",
                effects: vec![SpellEffect::ApplyCondition {
                    condition: Condition::Frightened,
                    duration: EffectDuration::Rounds(10),
                    save: Some(Ability::Charisma),
                }],
            },
            SpellId::HealingWord => SpellData {
                name: "Healing Word",
                level: 1,
                school: SpellSchool::Evocation,
                casting_time: "1 bonus action",
                range: 60,
                target: SpellTarget::Single,
                components: SpellComponents::v(),
                duration: "Instantaneous",
                description: "A creature regains hit points at range.",
                effects: vec![SpellEffect::Healing {
                    dice: Dice::simple(1, DiceType::D4),
                }],
            },
            // Level 2
            SpellId::ScorchingRay => SpellData {
                name: "Scorching Ray",
                level: 2,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 120,
                target: SpellTarget::Single,
                components: SpellComponents::vs(),
                duration: "Instantaneous",
                description: "Three rays of fire. Each ray deals 2d6 fire damage.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(6, DiceType::D6), // 3 rays * 2d6
                    damage_type: DamageType::Fire,
                    save_for_half: None,
                }],
            },
            SpellId::HoldPerson => SpellData {
                name: "Hold Person",
                level: 2,
                school: SpellSchool::Enchantment,
                casting_time: "1 action",
                range: 60,
                target: SpellTarget::Single,
                components: SpellComponents::vsm("a small piece of iron"),
                duration: "Concentration, 1 minute",
                description: "A humanoid must succeed on a WIS save or be paralyzed.",
                effects: vec![SpellEffect::ApplyCondition {
                    condition: Condition::Paralyzed,
                    duration: EffectDuration::UntilSave,
                    save: Some(Ability::Wisdom),
                }],
            },
            SpellId::MistyStep => SpellData {
                name: "Misty Step",
                level: 2,
                school: SpellSchool::Conjuration,
                casting_time: "1 bonus action",
                range: 0,
                target: SpellTarget::SelfTarget,
                components: SpellComponents::v(),
                duration: "Instantaneous",
                description: "Teleport up to 30 feet to an unoccupied space.",
                effects: vec![SpellEffect::Utility {
                    description: "Teleport 30 feet".to_string(),
                }],
            },
            // Level 3
            SpellId::Fireball => SpellData {
                name: "Fireball",
                level: 3,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 150,
                target: SpellTarget::Area(20),
                components: SpellComponents::vsm("bat guano and sulfur"),
                duration: "Instantaneous",
                description: "A bright streak explodes into a 20-foot radius sphere of fire.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(8, DiceType::D6),
                    damage_type: DamageType::Fire,
                    save_for_half: Some(Ability::Dexterity),
                }],
            },
            SpellId::LightningBolt => SpellData {
                name: "Lightning Bolt",
                level: 3,
                school: SpellSchool::Evocation,
                casting_time: "1 action",
                range: 0,
                target: SpellTarget::Line(100),
                components: SpellComponents::vsm("fur and a glass rod"),
                duration: "Instantaneous",
                description: "A stroke of lightning 100 feet long and 5 feet wide.",
                effects: vec![SpellEffect::Damage {
                    dice: Dice::simple(8, DiceType::D6),
                    damage_type: DamageType::Lightning,
                    save_for_half: Some(Ability::Dexterity),
                }],
            },
        }
    }

    /// Get the spell name.
    pub fn name(&self) -> &'static str {
        self.data().name
    }

    /// Get the spell level (0 for cantrips).
    pub fn level(&self) -> u8 {
        self.data().level
    }

    /// Check if this is a cantrip.
    pub fn is_cantrip(&self) -> bool {
        self.level() == 0
    }

    /// Scale cantrip damage based on character level.
    pub fn cantrip_dice_count(character_level: i32) -> i32 {
        match character_level {
            1..=4 => 1,
            5..=10 => 2,
            11..=16 => 3,
            _ => 4,
        }
    }
}

/// Component tracking spell slots.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SpellSlots {
    /// Current slots per level (index 0 = level 1).
    pub current: [u8; 9],
    /// Maximum slots per level.
    pub max: [u8; 9],
}

impl Default for SpellSlots {
    fn default() -> Self {
        Self {
            current: [0; 9],
            max: [0; 9],
        }
    }
}

impl SpellSlots {
    /// Create spell slots for a full caster at a given level.
    pub fn for_full_caster(level: u8) -> Self {
        let max = match level {
            1 => [2, 0, 0, 0, 0, 0, 0, 0, 0],
            2 => [3, 0, 0, 0, 0, 0, 0, 0, 0],
            3 => [4, 2, 0, 0, 0, 0, 0, 0, 0],
            4 => [4, 3, 0, 0, 0, 0, 0, 0, 0],
            5 => [4, 3, 2, 0, 0, 0, 0, 0, 0],
            6 => [4, 3, 3, 0, 0, 0, 0, 0, 0],
            7 => [4, 3, 3, 1, 0, 0, 0, 0, 0],
            8 => [4, 3, 3, 2, 0, 0, 0, 0, 0],
            9 => [4, 3, 3, 3, 1, 0, 0, 0, 0],
            10 => [4, 3, 3, 3, 2, 0, 0, 0, 0],
            11..=12 => [4, 3, 3, 3, 2, 1, 0, 0, 0],
            13..=14 => [4, 3, 3, 3, 2, 1, 1, 0, 0],
            15..=16 => [4, 3, 3, 3, 2, 1, 1, 1, 0],
            17 => [4, 3, 3, 3, 2, 1, 1, 1, 1],
            18 => [4, 3, 3, 3, 3, 1, 1, 1, 1],
            19 => [4, 3, 3, 3, 3, 2, 1, 1, 1],
            _ => [4, 3, 3, 3, 3, 2, 2, 1, 1],
        };
        Self { current: max, max }
    }

    /// Check if a spell slot of the given level is available.
    pub fn has_slot(&self, level: u8) -> bool {
        if level == 0 || level > 9 {
            return level == 0; // Cantrips always available
        }
        self.current[(level - 1) as usize] > 0
    }

    /// Use a spell slot.
    pub fn use_slot(&mut self, level: u8) -> bool {
        if level == 0 {
            return true; // Cantrips don't use slots
        }
        if level > 9 {
            return false;
        }
        let idx = (level - 1) as usize;
        if self.current[idx] > 0 {
            self.current[idx] -= 1;
            true
        } else {
            false
        }
    }

    /// Restore all spell slots (long rest).
    pub fn restore_all(&mut self) {
        self.current = self.max;
    }

    /// Restore a single slot of the given level.
    pub fn restore_slot(&mut self, level: u8) {
        if level > 0 && level <= 9 {
            let idx = (level - 1) as usize;
            self.current[idx] = self.current[idx].min(self.max[idx]);
            if self.current[idx] < self.max[idx] {
                self.current[idx] += 1;
            }
        }
    }
}

/// Component tracking known spells.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct KnownSpells {
    pub spells: Vec<SpellId>,
}

impl KnownSpells {
    pub fn add(&mut self, spell: SpellId) {
        if !self.spells.contains(&spell) {
            self.spells.push(spell);
        }
    }

    pub fn knows(&self, spell: SpellId) -> bool {
        self.spells.contains(&spell)
    }

    /// Get all known cantrips.
    pub fn cantrips(&self) -> impl Iterator<Item = &SpellId> {
        self.spells.iter().filter(|s| s.is_cantrip())
    }

    /// Get all known spells of a given level.
    pub fn spells_of_level(&self, level: u8) -> impl Iterator<Item = &SpellId> {
        self.spells.iter().filter(move |s| s.level() == level)
    }
}

/// Component for spellcasting ability.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SpellcastingAbility {
    pub ability: Ability,
}

impl Default for SpellcastingAbility {
    fn default() -> Self {
        Self {
            ability: Ability::Intelligence,
        }
    }
}

/// Message to cast a spell.
#[derive(Message, Debug, Clone)]
pub struct CastSpellMessage {
    pub caster: Entity,
    pub spell: SpellId,
    pub target: Option<Entity>,
    pub slot_level: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_data() {
        let fireball = SpellId::Fireball.data();
        assert_eq!(fireball.name, "Fireball");
        assert_eq!(fireball.level, 3);
        assert_eq!(fireball.school, SpellSchool::Evocation);
    }

    #[test]
    fn test_spell_slots() {
        let mut slots = SpellSlots::for_full_caster(5);
        assert!(slots.has_slot(1));
        assert!(slots.has_slot(3));
        assert!(!slots.has_slot(4));

        assert!(slots.use_slot(3));
        assert!(slots.use_slot(3));
        assert!(!slots.use_slot(3)); // No more level 3 slots

        slots.restore_all();
        assert!(slots.has_slot(3));
    }

    #[test]
    fn test_known_spells() {
        let mut known = KnownSpells::default();
        known.add(SpellId::FireBolt);
        known.add(SpellId::MagicMissile);
        known.add(SpellId::Fireball);

        assert!(known.knows(SpellId::FireBolt));
        assert!(known.knows(SpellId::Fireball));
        assert!(!known.knows(SpellId::Shield));

        assert_eq!(known.cantrips().count(), 1);
        assert_eq!(known.spells_of_level(1).count(), 1);
        assert_eq!(known.spells_of_level(3).count(), 1);
    }

    #[test]
    fn test_cantrip_scaling() {
        assert_eq!(SpellId::cantrip_dice_count(1), 1);
        assert_eq!(SpellId::cantrip_dice_count(5), 2);
        assert_eq!(SpellId::cantrip_dice_count(11), 3);
        assert_eq!(SpellId::cantrip_dice_count(17), 4);
    }
}
