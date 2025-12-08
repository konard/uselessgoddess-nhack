//! D&D ECS Components for character sheets and game entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ability::{Ability, AbilityScores};
use super::armor::{Armor, ArmorCategory, ArmorType};
use super::dice::{Dice, DiceType};
use super::weapons::{Weapon, WeaponCategory, WeaponType};

/// Plugin for D&D components.
pub fn plugin(app: &mut App) {
    app.register_type::<CharacterSheet>()
        .register_type::<ArmorClass>()
        .register_type::<HitPoints>()
        .register_type::<Proficiencies>()
        .register_type::<Skills>()
        .register_type::<Equipment>()
        .register_type::<Level>();
}

/// Proficiency bonus by character level.
pub fn proficiency_bonus(level: i32) -> i32 {
    match level {
        1..=4 => 2,
        5..=8 => 3,
        9..=12 => 4,
        13..=16 => 5,
        17..=20 => 6,
        _ => 2,
    }
}

/// Character or monster level/CR.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Level {
    /// Character level (1-20) or monster CR as effective level.
    pub value: i32,
}

impl Default for Level {
    fn default() -> Self {
        Self { value: 1 }
    }
}

impl Level {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Get the proficiency bonus for this level.
    pub fn proficiency_bonus(&self) -> i32 {
        proficiency_bonus(self.value)
    }
}

/// Hit Points component (D&D style with hit dice).
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct HitPoints {
    /// Current hit points.
    pub current: i32,
    /// Maximum hit points.
    pub max: i32,
    /// Temporary hit points.
    pub temp: i32,
    /// Hit dice (e.g., "2d8" for 2 levels with d8 hit die).
    pub hit_dice_count: i32,
    /// Hit die type.
    pub hit_die_type: DiceType,
    /// Remaining hit dice for short rests.
    pub hit_dice_remaining: i32,
}

impl Default for HitPoints {
    fn default() -> Self {
        Self {
            current: 10,
            max: 10,
            temp: 0,
            hit_dice_count: 1,
            hit_die_type: DiceType::D8,
            hit_dice_remaining: 1,
        }
    }
}

impl HitPoints {
    /// Create hit points for a character with given max HP.
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
            temp: 0,
            hit_dice_count: 1,
            hit_die_type: DiceType::D8,
            hit_dice_remaining: 1,
        }
    }

    /// Create hit points with hit dice info.
    pub fn with_hit_dice(mut self, count: i32, die_type: DiceType) -> Self {
        self.hit_dice_count = count;
        self.hit_die_type = die_type;
        self.hit_dice_remaining = count;
        self
    }

    /// Calculate starting HP: max hit die + CON modifier per level.
    pub fn calculate_for_level(level: i32, hit_die: DiceType, con_modifier: i32) -> Self {
        // First level: max hit die + CON
        // Each additional level: average + CON (or roll)
        let first_level_hp = hit_die.sides() + con_modifier;
        let per_level_avg = (hit_die.sides() / 2 + 1) + con_modifier;
        let additional_hp = (level - 1) * per_level_avg;
        let max_hp = (first_level_hp + additional_hp).max(1);

        Self {
            current: max_hp,
            max: max_hp,
            temp: 0,
            hit_dice_count: level,
            hit_die_type: hit_die,
            hit_dice_remaining: level,
        }
    }

    /// Take damage, consuming temp HP first.
    pub fn take_damage(&mut self, damage: i32) {
        let mut remaining = damage;

        // Temp HP absorbs first
        if self.temp > 0 {
            if self.temp >= remaining {
                self.temp -= remaining;
                return;
            } else {
                remaining -= self.temp;
                self.temp = 0;
            }
        }

        // Then current HP
        self.current = (self.current - remaining).max(0);
    }

    /// Heal hit points (can't exceed max).
    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Add temporary hit points (doesn't stack).
    pub fn add_temp_hp(&mut self, amount: i32) {
        // Temp HP doesn't stack, take the higher value
        self.temp = self.temp.max(amount);
    }

    /// Check if at 0 HP.
    pub fn is_unconscious(&self) -> bool {
        self.current <= 0
    }

    /// Check if dead (massive damage rule simplified).
    pub fn is_dead(&self) -> bool {
        self.current <= -self.max
    }

    /// Get HP as a percentage.
    pub fn percentage(&self) -> f32 {
        if self.max <= 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }

    /// Get the hit dice expression.
    pub fn hit_dice(&self) -> Dice {
        Dice::simple(self.hit_dice_count, self.hit_die_type)
    }
}

/// Armor Class component.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ArmorClass {
    /// Base AC (from armor or unarmored defense).
    pub base: i32,
    /// Bonus from shield.
    pub shield_bonus: i32,
    /// Bonus from magic items or spells.
    pub misc_bonus: i32,
    /// Whether wearing armor that grants stealth disadvantage.
    pub stealth_disadvantage: bool,
}

impl Default for ArmorClass {
    fn default() -> Self {
        Self {
            base: 10,
            shield_bonus: 0,
            misc_bonus: 0,
            stealth_disadvantage: false,
        }
    }
}

impl ArmorClass {
    /// Create AC from a specific value.
    pub fn new(base: i32) -> Self {
        Self {
            base,
            ..Default::default()
        }
    }

    /// Calculate AC from armor and DEX modifier.
    pub fn from_armor(armor: &Armor, dex_modifier: i32) -> Self {
        let data = armor.data();
        Self {
            base: armor.calculate_ac(dex_modifier),
            shield_bonus: 0,
            misc_bonus: 0,
            stealth_disadvantage: data.stealth_disadvantage,
        }
    }

    /// Add shield bonus.
    pub fn with_shield(mut self) -> Self {
        self.shield_bonus = 2;
        self
    }

    /// Add miscellaneous bonus.
    pub fn with_bonus(mut self, bonus: i32) -> Self {
        self.misc_bonus = bonus;
        self
    }

    /// Get total AC.
    pub fn total(&self) -> i32 {
        self.base + self.shield_bonus + self.misc_bonus
    }
}

/// D&D Skills component.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Skills {
    // STR skills
    pub athletics: SkillRank,

    // DEX skills
    pub acrobatics: SkillRank,
    pub sleight_of_hand: SkillRank,
    pub stealth: SkillRank,

    // INT skills
    pub arcana: SkillRank,
    pub history: SkillRank,
    pub investigation: SkillRank,
    pub nature: SkillRank,
    pub religion: SkillRank,

    // WIS skills
    pub animal_handling: SkillRank,
    pub insight: SkillRank,
    pub medicine: SkillRank,
    pub perception: SkillRank,
    pub survival: SkillRank,

    // CHA skills
    pub deception: SkillRank,
    pub intimidation: SkillRank,
    pub performance: SkillRank,
    pub persuasion: SkillRank,
}

/// Skill proficiency level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum SkillRank {
    #[default]
    None,
    Proficient,
    Expertise, // Double proficiency bonus
}

impl SkillRank {
    /// Get the proficiency multiplier.
    pub fn multiplier(&self) -> i32 {
        match self {
            SkillRank::None => 0,
            SkillRank::Proficient => 1,
            SkillRank::Expertise => 2,
        }
    }
}

/// All skills and their governing abilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Skill {
    Athletics,
    Acrobatics,
    SleightOfHand,
    Stealth,
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl Skill {
    /// Get the ability that governs this skill.
    pub fn ability(&self) -> Ability {
        match self {
            Skill::Athletics => Ability::Strength,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Ability::Dexterity,
            Skill::Arcana
            | Skill::History
            | Skill::Investigation
            | Skill::Nature
            | Skill::Religion => Ability::Intelligence,
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Perception
            | Skill::Survival => Ability::Wisdom,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => {
                Ability::Charisma
            }
        }
    }
}

impl Skills {
    /// Get the rank for a specific skill.
    pub fn get(&self, skill: Skill) -> SkillRank {
        match skill {
            Skill::Athletics => self.athletics,
            Skill::Acrobatics => self.acrobatics,
            Skill::SleightOfHand => self.sleight_of_hand,
            Skill::Stealth => self.stealth,
            Skill::Arcana => self.arcana,
            Skill::History => self.history,
            Skill::Investigation => self.investigation,
            Skill::Nature => self.nature,
            Skill::Religion => self.religion,
            Skill::AnimalHandling => self.animal_handling,
            Skill::Insight => self.insight,
            Skill::Medicine => self.medicine,
            Skill::Perception => self.perception,
            Skill::Survival => self.survival,
            Skill::Deception => self.deception,
            Skill::Intimidation => self.intimidation,
            Skill::Performance => self.performance,
            Skill::Persuasion => self.persuasion,
        }
    }

    /// Set the rank for a specific skill.
    pub fn set(&mut self, skill: Skill, rank: SkillRank) {
        match skill {
            Skill::Athletics => self.athletics = rank,
            Skill::Acrobatics => self.acrobatics = rank,
            Skill::SleightOfHand => self.sleight_of_hand = rank,
            Skill::Stealth => self.stealth = rank,
            Skill::Arcana => self.arcana = rank,
            Skill::History => self.history = rank,
            Skill::Investigation => self.investigation = rank,
            Skill::Nature => self.nature = rank,
            Skill::Religion => self.religion = rank,
            Skill::AnimalHandling => self.animal_handling = rank,
            Skill::Insight => self.insight = rank,
            Skill::Medicine => self.medicine = rank,
            Skill::Perception => self.perception = rank,
            Skill::Survival => self.survival = rank,
            Skill::Deception => self.deception = rank,
            Skill::Intimidation => self.intimidation = rank,
            Skill::Performance => self.performance = rank,
            Skill::Persuasion => self.persuasion = rank,
        }
    }

    /// Calculate skill modifier.
    pub fn modifier(
        &self,
        skill: Skill,
        ability_scores: &AbilityScores,
        proficiency_bonus: i32,
    ) -> i32 {
        let ability_mod = ability_scores.modifier(skill.ability());
        let prof_multiplier = self.get(skill).multiplier();
        ability_mod + (proficiency_bonus * prof_multiplier)
    }
}

/// Proficiency with weapons, armor, tools, and saving throws.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Proficiencies {
    // Weapon proficiencies
    pub simple_weapons: bool,
    pub martial_weapons: bool,
    pub specific_weapons: Vec<WeaponType>,

    // Armor proficiencies
    pub light_armor: bool,
    pub medium_armor: bool,
    pub heavy_armor: bool,
    pub shields: bool,

    // Saving throw proficiencies
    pub strength_save: bool,
    pub dexterity_save: bool,
    pub constitution_save: bool,
    pub intelligence_save: bool,
    pub wisdom_save: bool,
    pub charisma_save: bool,
}

impl Proficiencies {
    /// Check if proficient with a weapon.
    pub fn is_weapon_proficient(&self, weapon: &Weapon) -> bool {
        let data = weapon.data();

        // Check specific weapon proficiency
        if self.specific_weapons.contains(&weapon.weapon_type) {
            return true;
        }

        // Check category proficiency
        match data.category {
            WeaponCategory::Simple => self.simple_weapons,
            WeaponCategory::Martial => self.martial_weapons,
            WeaponCategory::Improvised => false, // Never proficient with improvised
        }
    }

    /// Check if proficient with armor.
    pub fn is_armor_proficient(&self, armor: &Armor) -> bool {
        let data = armor.data();
        match data.category {
            ArmorCategory::Light => self.light_armor,
            ArmorCategory::Medium => self.medium_armor,
            ArmorCategory::Heavy => self.heavy_armor,
            ArmorCategory::Shield => self.shields,
        }
    }

    /// Check if proficient in a saving throw.
    pub fn has_save_proficiency(&self, ability: Ability) -> bool {
        match ability {
            Ability::Strength => self.strength_save,
            Ability::Dexterity => self.dexterity_save,
            Ability::Constitution => self.constitution_save,
            Ability::Intelligence => self.intelligence_save,
            Ability::Wisdom => self.wisdom_save,
            Ability::Charisma => self.charisma_save,
        }
    }

    /// Calculate saving throw modifier.
    pub fn save_modifier(
        &self,
        ability: Ability,
        ability_scores: &AbilityScores,
        proficiency_bonus: i32,
    ) -> i32 {
        let ability_mod = ability_scores.modifier(ability);
        if self.has_save_proficiency(ability) {
            ability_mod + proficiency_bonus
        } else {
            ability_mod
        }
    }
}

/// Equipment slots and inventory.
#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Equipment {
    /// Currently equipped armor (body slot).
    pub armor: Option<ArmorType>,
    /// Currently equipped shield (off-hand).
    pub shield: bool,
    /// Currently wielded weapon (main hand).
    pub main_hand: Option<WeaponType>,
    /// Off-hand weapon (for dual wielding).
    pub off_hand: Option<WeaponType>,
}

impl Equipment {
    /// Check if wielding a shield.
    pub fn has_shield(&self) -> bool {
        self.shield
    }

    /// Check if dual wielding.
    pub fn is_dual_wielding(&self) -> bool {
        self.main_hand.is_some() && self.off_hand.is_some() && !self.shield
    }
}

/// Complete character sheet combining all D&D components.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterSheet {
    /// Character name.
    pub name: String,
    /// Character level.
    pub level: i32,
    /// Experience points (optional, for tracking).
    pub xp: i32,
}

impl Default for CharacterSheet {
    fn default() -> Self {
        Self {
            name: "Adventurer".to_string(),
            level: 1,
            xp: 0,
        }
    }
}

impl CharacterSheet {
    pub fn new(name: impl Into<String>, level: i32) -> Self {
        Self {
            name: name.into(),
            level,
            xp: 0,
        }
    }

    /// Get proficiency bonus based on level.
    pub fn proficiency_bonus(&self) -> i32 {
        proficiency_bonus(self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proficiency_bonus() {
        assert_eq!(proficiency_bonus(1), 2);
        assert_eq!(proficiency_bonus(4), 2);
        assert_eq!(proficiency_bonus(5), 3);
        assert_eq!(proficiency_bonus(8), 3);
        assert_eq!(proficiency_bonus(9), 4);
        assert_eq!(proficiency_bonus(17), 6);
        assert_eq!(proficiency_bonus(20), 6);
    }

    #[test]
    fn test_hit_points() {
        let mut hp = HitPoints::new(20);
        assert_eq!(hp.current, 20);
        assert_eq!(hp.max, 20);

        hp.take_damage(5);
        assert_eq!(hp.current, 15);

        hp.heal(3);
        assert_eq!(hp.current, 18);

        hp.heal(100);
        assert_eq!(hp.current, 20); // Can't exceed max

        hp.add_temp_hp(10);
        hp.take_damage(5);
        assert_eq!(hp.temp, 5);
        assert_eq!(hp.current, 20);

        hp.take_damage(10);
        assert_eq!(hp.temp, 0);
        assert_eq!(hp.current, 15);
    }

    #[test]
    fn test_skill_modifier() {
        let scores = AbilityScores::new(10, 16, 10, 10, 14, 10);
        let mut skills = Skills::default();
        skills.set(Skill::Stealth, SkillRank::Proficient);

        // Unproficient perception: WIS mod only = +2
        assert_eq!(skills.modifier(Skill::Perception, &scores, 2), 2);

        // Proficient stealth: DEX mod + prof = +3 + 2 = +5
        assert_eq!(skills.modifier(Skill::Stealth, &scores, 2), 5);

        // Expertise would be +3 + (2 * 2) = +7
        skills.set(Skill::Stealth, SkillRank::Expertise);
        assert_eq!(skills.modifier(Skill::Stealth, &scores, 2), 7);
    }
}
