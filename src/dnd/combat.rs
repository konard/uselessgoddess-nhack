//! D&D 5e Combat System.
//!
//! Implements combat mechanics including:
//! - Attack rolls (d20 + modifiers vs AC)
//! - Damage rolls
//! - Critical hits
//! - Advantage/Disadvantage

use bevy::prelude::*;

use super::ability::{Ability, AbilityScores};
use super::components::{ArmorClass, Level, Proficiencies};
use super::dice::{Dice, DiceRoll, DiceRoller};
use super::weapons::{DamageType, Weapon, WeaponProperty};

/// Plugin for combat systems.
pub fn plugin(app: &mut App) {
    app.add_message::<AttackMessage>()
        .add_message::<DamageMessage>()
        .add_systems(Update, process_attacks.run_if(on_message::<AttackMessage>));
}

/// Message for initiating an attack.
#[derive(Message, Debug, Clone)]
pub struct AttackMessage {
    pub attacker: Entity,
    pub defender: Entity,
    pub weapon: Option<WeaponType>,
    pub advantage: AttackModifier,
}

/// Advantage/disadvantage state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackModifier {
    #[default]
    Normal,
    Advantage,
    Disadvantage,
}

use super::weapons::WeaponType;

/// Message for applying damage.
#[derive(Message, Debug, Clone)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: i32,
    pub damage_type: DamageType,
    pub source: Option<Entity>,
    pub is_critical: bool,
}

/// Result of an attack roll.
#[derive(Debug, Clone)]
pub struct AttackResult {
    /// The d20 roll.
    pub attack_roll: DiceRoll,
    /// Total attack bonus applied.
    pub attack_bonus: i32,
    /// Final attack total.
    pub total: i32,
    /// Target's AC.
    pub target_ac: i32,
    /// Whether the attack hit.
    pub hit: bool,
    /// Whether this was a critical hit (natural 20).
    pub critical: bool,
    /// Whether this was a critical miss (natural 1).
    pub fumble: bool,
    /// Damage roll (if hit).
    pub damage: Option<DiceRoll>,
    /// Total damage dealt.
    pub damage_total: i32,
    /// Type of damage.
    pub damage_type: DamageType,
}

impl AttackResult {
    /// Create a miss result.
    pub fn miss(attack_roll: DiceRoll, attack_bonus: i32, target_ac: i32) -> Self {
        let total = attack_roll.total + attack_bonus;
        let fumble = attack_roll.is_natural_1();
        Self {
            attack_roll,
            attack_bonus,
            total,
            target_ac,
            hit: false,
            critical: false,
            fumble,
            damage: None,
            damage_total: 0,
            damage_type: DamageType::Bludgeoning,
        }
    }

    /// Create a hit result.
    pub fn hit(
        attack_roll: DiceRoll,
        attack_bonus: i32,
        target_ac: i32,
        damage: DiceRoll,
        damage_type: DamageType,
    ) -> Self {
        let total = attack_roll.total + attack_bonus;
        let critical = attack_roll.is_natural_20();
        Self {
            attack_roll,
            attack_bonus,
            total,
            target_ac,
            hit: true,
            critical,
            fumble: false,
            damage_total: damage.total,
            damage: Some(damage),
            damage_type,
        }
    }

    /// Create a critical hit result.
    pub fn critical_hit(
        attack_roll: DiceRoll,
        attack_bonus: i32,
        target_ac: i32,
        damage: DiceRoll,
        damage_type: DamageType,
    ) -> Self {
        let total = attack_roll.total + attack_bonus;
        Self {
            attack_roll,
            attack_bonus,
            total,
            target_ac,
            hit: true,
            critical: true,
            fumble: false,
            damage_total: damage.total,
            damage: Some(damage),
            damage_type,
        }
    }

    /// Get a description of the result.
    pub fn description(&self) -> String {
        if self.fumble {
            "Critical miss!".to_string()
        } else if self.critical {
            format!(
                "Critical hit! {} {} damage!",
                self.damage_total,
                self.damage_type.name()
            )
        } else if self.hit {
            format!(
                "Hit for {} {} damage.",
                self.damage_total,
                self.damage_type.name()
            )
        } else {
            format!(
                "Miss. ({} vs AC {})",
                self.total, self.target_ac
            )
        }
    }
}

/// Calculate attack bonus for a weapon attack.
pub fn calculate_attack_bonus(
    ability_scores: &AbilityScores,
    proficiencies: &Proficiencies,
    level: &Level,
    weapon: &Weapon,
) -> i32 {
    let data = weapon.data();

    // Determine which ability to use
    let ability = if data.has_property(WeaponProperty::Finesse) {
        // Finesse: use higher of STR or DEX
        if ability_scores.dex_mod() > ability_scores.str_mod() {
            Ability::Dexterity
        } else {
            Ability::Strength
        }
    } else {
        data.attack_ability()
    };

    let ability_mod = ability_scores.modifier(ability);

    // Add proficiency if proficient
    let prof_bonus = if proficiencies.is_weapon_proficient(weapon) {
        level.proficiency_bonus()
    } else {
        0
    };

    // Add magic bonus
    ability_mod + prof_bonus + weapon.magic_bonus
}

/// Calculate damage bonus for a weapon attack.
pub fn calculate_damage_bonus(ability_scores: &AbilityScores, weapon: &Weapon) -> i32 {
    let data = weapon.data();

    // Determine which ability to use
    let ability = if data.has_property(WeaponProperty::Finesse) {
        // Finesse: use higher of STR or DEX
        if ability_scores.dex_mod() > ability_scores.str_mod() {
            Ability::Dexterity
        } else {
            Ability::Strength
        }
    } else {
        data.attack_ability()
    };

    ability_scores.modifier(ability) + weapon.magic_bonus
}

/// Perform a weapon attack.
pub fn make_attack(
    dice_roller: &mut DiceRoller,
    ability_scores: &AbilityScores,
    proficiencies: &Proficiencies,
    level: &Level,
    weapon: &Weapon,
    target_ac: i32,
    modifier: AttackModifier,
) -> AttackResult {
    let attack_bonus = calculate_attack_bonus(ability_scores, proficiencies, level, weapon);

    // Roll d20 with advantage/disadvantage
    let attack_roll = match modifier {
        AttackModifier::Normal => dice_roller.roll_d20(),
        AttackModifier::Advantage => dice_roller.roll_d20_advantage(),
        AttackModifier::Disadvantage => dice_roller.roll_d20_disadvantage(),
    };

    // Natural 1 always misses
    if attack_roll.is_natural_1() {
        return AttackResult::miss(attack_roll, attack_bonus, target_ac);
    }

    // Natural 20 always hits and is a critical
    let is_critical = attack_roll.is_natural_20();
    let total = attack_roll.total + attack_bonus;

    if is_critical || total >= target_ac {
        // Hit - roll damage
        let data = weapon.data();
        let damage_bonus = calculate_damage_bonus(ability_scores, weapon);

        let damage_dice = if is_critical {
            // Critical: double the dice
            Dice::new(
                data.damage.count * 2,
                data.damage.die_type,
                damage_bonus,
            )
        } else {
            Dice::new(data.damage.count, data.damage.die_type, damage_bonus)
        };

        let damage_roll = dice_roller.roll(&damage_dice);

        if is_critical {
            AttackResult::critical_hit(attack_roll, attack_bonus, target_ac, damage_roll, data.damage_type)
        } else {
            AttackResult::hit(attack_roll, attack_bonus, target_ac, damage_roll, data.damage_type)
        }
    } else {
        AttackResult::miss(attack_roll, attack_bonus, target_ac)
    }
}

/// Calculate saving throw DC based on ability.
pub fn calculate_save_dc(ability_scores: &AbilityScores, level: &Level, ability: Ability) -> i32 {
    8 + level.proficiency_bonus() + ability_scores.modifier(ability)
}

/// Perform a saving throw.
pub fn make_saving_throw(
    dice_roller: &mut DiceRoller,
    ability_scores: &AbilityScores,
    proficiencies: &Proficiencies,
    level: &Level,
    ability: Ability,
    dc: i32,
    modifier: AttackModifier,
) -> (DiceRoll, bool) {
    // Roll d20 with advantage/disadvantage
    let roll = match modifier {
        AttackModifier::Normal => dice_roller.roll_d20(),
        AttackModifier::Advantage => dice_roller.roll_d20_advantage(),
        AttackModifier::Disadvantage => dice_roller.roll_d20_disadvantage(),
    };

    let save_bonus = proficiencies.save_modifier(ability, ability_scores, level.proficiency_bonus());
    let total = roll.total + save_bonus;

    (roll, total >= dc)
}

/// Process attack messages.
fn process_attacks(
    mut attack_messages: MessageReader<AttackMessage>,
    mut damage_messages: MessageWriter<DamageMessage>,
    mut dice_roller: ResMut<DiceRoller>,
    attacker_query: Query<(&AbilityScores, &Proficiencies, &Level, Option<&Weapon>)>,
    defender_query: Query<&ArmorClass>,
) {
    for message in attack_messages.read() {
        let Ok((ability_scores, proficiencies, level, equipped_weapon)) =
            attacker_query.get(message.attacker)
        else {
            continue;
        };

        let Ok(target_ac) = defender_query.get(message.defender) else {
            continue;
        };

        // Determine weapon to use
        let weapon = if let Some(weapon_type) = message.weapon {
            Weapon::new(weapon_type)
        } else if let Some(w) = equipped_weapon {
            w.clone()
        } else {
            // Unarmed strike
            Weapon::new(WeaponType::Unarmed)
        };

        let result = make_attack(
            &mut dice_roller,
            ability_scores,
            proficiencies,
            level,
            &weapon,
            target_ac.total(),
            message.advantage,
        );

        if result.hit {
            damage_messages.write(DamageMessage {
                target: message.defender,
                amount: result.damage_total,
                damage_type: result.damage_type,
                source: Some(message.attacker),
                is_critical: result.critical,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_bonus_calculation() {
        let scores = AbilityScores::new(16, 14, 10, 10, 10, 10); // +3 STR, +2 DEX
        let mut profs = Proficiencies::default();
        profs.martial_weapons = true;
        let level = Level::new(5); // +3 proficiency

        // Longsword (STR weapon, martial)
        let longsword = Weapon::new(WeaponType::Longsword);
        let bonus = calculate_attack_bonus(&scores, &profs, &level, &longsword);
        assert_eq!(bonus, 6); // +3 STR + 3 prof

        // Rapier (finesse weapon, uses DEX if higher)
        let rapier = Weapon::new(WeaponType::Rapier);
        let bonus = calculate_attack_bonus(&scores, &profs, &level, &rapier);
        assert_eq!(bonus, 6); // Uses STR (+3) since it's higher than DEX (+2)

        // Test with higher DEX
        let dex_scores = AbilityScores::new(10, 18, 10, 10, 10, 10); // +0 STR, +4 DEX
        let bonus = calculate_attack_bonus(&dex_scores, &profs, &level, &rapier);
        assert_eq!(bonus, 7); // +4 DEX + 3 prof
    }

    #[test]
    fn test_save_dc() {
        let scores = AbilityScores::new(10, 10, 14, 16, 10, 10); // +3 INT
        let level = Level::new(5); // +3 prof

        let dc = calculate_save_dc(&scores, &level, Ability::Intelligence);
        assert_eq!(dc, 14); // 8 + 3 prof + 3 INT
    }
}
