//! D&D 5e Status Effects and Conditions.
//!
//! Implements the condition system from the SRD including:
//! - Standard conditions (Poisoned, Frightened, etc.)
//! - Damage resistance/vulnerability/immunity
//! - Duration tracking
//! - Effect application and removal

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::dice::DiceType;
use super::weapons::DamageType;

// Note: Serialize/Deserialize are used for condition types and damage over time,
// but ActiveEffect doesn't derive them because Entity can't be serialized.

/// Plugin for effect systems.
pub fn plugin(app: &mut App) {
    app.add_message::<ApplyEffectMessage>()
        .add_message::<RemoveEffectMessage>()
        .register_type::<StatusEffects>()
        .register_type::<DamageModifiers>()
        .add_systems(Update, (tick_effects, process_apply_effects, process_remove_effects).chain());
}

/// Standard D&D 5e conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum Condition {
    /// Disadvantage on ability checks and attack rolls.
    Poisoned,
    /// Disadvantage on ability checks and attack rolls;
    /// can't move closer to source of fear.
    Frightened,
    /// Speed becomes 0; can't benefit from bonuses to speed.
    Grappled,
    /// Can't take actions or reactions.
    Incapacitated,
    /// Auto-fail STR and DEX saves; attacks against have advantage;
    /// melee attacks within 5ft are critical hits.
    Paralyzed,
    /// Disadvantage on attack rolls; attacks against have advantage.
    Prone,
    /// Can't move; attacks against have advantage;
    /// attacks have disadvantage.
    Restrained,
    /// Auto-fail STR and DEX saves; attacks against have advantage;
    /// can't take actions or reactions.
    Stunned,
    /// Can't see; auto-fail ability checks requiring sight;
    /// attacks have disadvantage; attacks against have advantage.
    Blinded,
    /// Can't hear; auto-fail ability checks requiring hearing.
    Deafened,
    /// Can't speak or cast spells with verbal components.
    Silenced,
    /// Speed is 0; can't benefit from bonuses to speed;
    /// can't take actions or reactions.
    Petrified,
    /// HP is 0; unconscious.
    Unconscious,
    /// Creature is hidden from the target.
    Invisible,
    /// Charmed by source; can't attack source.
    Charmed,
}

impl Condition {
    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            Condition::Poisoned => "Poisoned",
            Condition::Frightened => "Frightened",
            Condition::Grappled => "Grappled",
            Condition::Incapacitated => "Incapacitated",
            Condition::Paralyzed => "Paralyzed",
            Condition::Prone => "Prone",
            Condition::Restrained => "Restrained",
            Condition::Stunned => "Stunned",
            Condition::Blinded => "Blinded",
            Condition::Deafened => "Deafened",
            Condition::Silenced => "Silenced",
            Condition::Petrified => "Petrified",
            Condition::Unconscious => "Unconscious",
            Condition::Invisible => "Invisible",
            Condition::Charmed => "Charmed",
        }
    }

    /// Check if this condition prevents taking actions.
    pub fn prevents_actions(&self) -> bool {
        matches!(
            self,
            Condition::Incapacitated
                | Condition::Paralyzed
                | Condition::Stunned
                | Condition::Petrified
                | Condition::Unconscious
        )
    }

    /// Check if this condition causes disadvantage on attacks.
    pub fn attack_disadvantage(&self) -> bool {
        matches!(
            self,
            Condition::Poisoned | Condition::Prone | Condition::Restrained | Condition::Blinded
        )
    }

    /// Check if attackers have advantage against this condition.
    pub fn grants_advantage_to_attackers(&self) -> bool {
        matches!(
            self,
            Condition::Paralyzed
                | Condition::Prone
                | Condition::Restrained
                | Condition::Stunned
                | Condition::Blinded
                | Condition::Unconscious
        )
    }
}

/// How damage is modified (resistance, vulnerability, immunity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DamageModifier {
    /// Take half damage (rounded down).
    Resistance,
    /// Take double damage.
    Vulnerability,
    /// Take no damage.
    Immunity,
}

/// Duration types for effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum EffectDuration {
    /// Lasts for a number of rounds.
    Rounds(u32),
    /// Lasts for a number of turns.
    Turns(u32),
    /// Lasts until a condition is met (e.g., save succeeds).
    UntilSave,
    /// Permanent until removed.
    Permanent,
    /// Instantaneous effect (apply and remove immediately).
    Instantaneous,
}

/// An active status effect on an entity.
#[derive(Debug, Clone, Reflect)]
pub struct ActiveEffect {
    /// The condition applied.
    pub condition: Condition,
    /// Remaining duration.
    pub duration: EffectDuration,
    /// Source of the effect (for fear/charm).
    #[reflect(ignore)]
    pub source: Option<Entity>,
    /// Save DC to end the effect (if applicable).
    pub save_dc: Option<i32>,
}

impl ActiveEffect {
    pub fn new(condition: Condition, duration: EffectDuration) -> Self {
        Self {
            condition,
            duration,
            source: None,
            save_dc: None,
        }
    }

    pub fn with_source(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_save_dc(mut self, dc: i32) -> Self {
        self.save_dc = Some(dc);
        self
    }
}

/// Component tracking all active status effects on an entity.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct StatusEffects {
    /// Active effects.
    pub effects: Vec<ActiveEffect>,
}

impl StatusEffects {
    /// Check if entity has a specific condition.
    pub fn has_condition(&self, condition: Condition) -> bool {
        self.effects.iter().any(|e| e.condition == condition)
    }

    /// Add an effect.
    pub fn add(&mut self, effect: ActiveEffect) {
        // Don't stack same conditions, but update duration if longer
        if let Some(existing) = self.effects.iter_mut().find(|e| e.condition == effect.condition) {
            existing.duration = effect.duration;
            existing.source = effect.source;
            existing.save_dc = effect.save_dc;
        } else {
            self.effects.push(effect);
        }
    }

    /// Remove all effects of a specific condition.
    pub fn remove_condition(&mut self, condition: Condition) {
        self.effects.retain(|e| e.condition != condition);
    }

    /// Get all active conditions.
    pub fn conditions(&self) -> impl Iterator<Item = Condition> + '_ {
        self.effects.iter().map(|e| e.condition)
    }

    /// Check if attacks have disadvantage due to conditions.
    pub fn has_attack_disadvantage(&self) -> bool {
        self.effects.iter().any(|e| e.condition.attack_disadvantage())
    }

    /// Check if actions are prevented.
    pub fn is_incapacitated(&self) -> bool {
        self.effects
            .iter()
            .any(|e| e.condition.prevents_actions())
    }

    /// Check if attackers have advantage.
    pub fn grants_advantage(&self) -> bool {
        self.effects
            .iter()
            .any(|e| e.condition.grants_advantage_to_attackers())
    }
}

/// Component tracking damage resistances, vulnerabilities, and immunities.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct DamageModifiers {
    /// Damage resistances (take half damage).
    pub resistances: Vec<DamageType>,
    /// Damage vulnerabilities (take double damage).
    pub vulnerabilities: Vec<DamageType>,
    /// Damage immunities (take no damage).
    pub immunities: Vec<DamageType>,
    /// Condition immunities.
    pub condition_immunities: Vec<Condition>,
}

impl DamageModifiers {
    /// Get the modifier for a damage type.
    pub fn get_modifier(&self, damage_type: DamageType) -> Option<DamageModifier> {
        if self.immunities.contains(&damage_type) {
            Some(DamageModifier::Immunity)
        } else if self.resistances.contains(&damage_type) {
            Some(DamageModifier::Resistance)
        } else if self.vulnerabilities.contains(&damage_type) {
            Some(DamageModifier::Vulnerability)
        } else {
            None
        }
    }

    /// Apply damage modifiers to an amount.
    pub fn apply(&self, amount: i32, damage_type: DamageType) -> i32 {
        match self.get_modifier(damage_type) {
            Some(DamageModifier::Immunity) => 0,
            Some(DamageModifier::Resistance) => amount / 2,
            Some(DamageModifier::Vulnerability) => amount * 2,
            None => amount,
        }
    }

    /// Check if immune to a condition.
    pub fn is_immune_to_condition(&self, condition: Condition) -> bool {
        self.condition_immunities.contains(&condition)
    }

    /// Add a resistance.
    pub fn add_resistance(&mut self, damage_type: DamageType) {
        if !self.resistances.contains(&damage_type) {
            self.resistances.push(damage_type);
        }
    }

    /// Add a vulnerability.
    pub fn add_vulnerability(&mut self, damage_type: DamageType) {
        if !self.vulnerabilities.contains(&damage_type) {
            self.vulnerabilities.push(damage_type);
        }
    }

    /// Add an immunity.
    pub fn add_immunity(&mut self, damage_type: DamageType) {
        if !self.immunities.contains(&damage_type) {
            self.immunities.push(damage_type);
        }
    }

    /// Add a condition immunity.
    pub fn add_condition_immunity(&mut self, condition: Condition) {
        if !self.condition_immunities.contains(&condition) {
            self.condition_immunities.push(condition);
        }
    }
}

/// Damage over time effect.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct DamageOverTime {
    /// Damage dice count.
    pub dice_count: i32,
    /// Damage die type.
    pub dice_type: DiceType,
    /// Flat damage modifier.
    pub modifier: i32,
    /// Type of damage.
    pub damage_type: DamageType,
    /// Remaining duration in rounds.
    pub remaining_rounds: u32,
    /// When to apply (start or end of turn).
    pub on_turn_start: bool,
}

/// Component for damage over time effects.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct DamageOverTimeEffects {
    pub effects: Vec<DamageOverTime>,
}

/// Message to apply an effect to an entity.
#[derive(Message, Debug, Clone)]
pub struct ApplyEffectMessage {
    pub target: Entity,
    pub effect: ActiveEffect,
}

/// Message to remove an effect from an entity.
#[derive(Message, Debug, Clone)]
pub struct RemoveEffectMessage {
    pub target: Entity,
    pub condition: Condition,
}

/// Process effect application messages.
fn process_apply_effects(
    mut messages: MessageReader<ApplyEffectMessage>,
    mut query: Query<(&mut StatusEffects, Option<&DamageModifiers>)>,
) {
    for message in messages.read() {
        if let Ok((mut effects, modifiers)) = query.get_mut(message.target) {
            // Check for condition immunity
            if let Some(mods) = modifiers {
                if mods.is_immune_to_condition(message.effect.condition) {
                    continue;
                }
            }
            effects.add(message.effect.clone());
        }
    }
}

/// Process effect removal messages.
fn process_remove_effects(
    mut messages: MessageReader<RemoveEffectMessage>,
    mut query: Query<&mut StatusEffects>,
) {
    for message in messages.read() {
        if let Ok(mut effects) = query.get_mut(message.target) {
            effects.remove_condition(message.condition);
        }
    }
}

/// Tick effect durations at the start of each round.
fn tick_effects(mut query: Query<&mut StatusEffects>) {
    for mut effects in &mut query {
        effects.effects.retain_mut(|effect| {
            match &mut effect.duration {
                EffectDuration::Rounds(remaining) => {
                    if *remaining > 0 {
                        *remaining -= 1;
                    }
                    *remaining > 0
                }
                EffectDuration::Turns(remaining) => {
                    if *remaining > 0 {
                        *remaining -= 1;
                    }
                    *remaining > 0
                }
                EffectDuration::Permanent | EffectDuration::UntilSave => true,
                EffectDuration::Instantaneous => false,
            }
        });
    }
}

/// Create standard undead damage modifiers.
pub fn undead_modifiers() -> DamageModifiers {
    DamageModifiers {
        resistances: vec![],
        vulnerabilities: vec![DamageType::Radiant],
        immunities: vec![DamageType::Poison, DamageType::Necrotic],
        condition_immunities: vec![
            Condition::Poisoned,
            Condition::Frightened,
            Condition::Charmed,
        ],
    }
}

/// Create standard construct damage modifiers.
pub fn construct_modifiers() -> DamageModifiers {
    DamageModifiers {
        resistances: vec![],
        vulnerabilities: vec![],
        immunities: vec![DamageType::Poison, DamageType::Psychic],
        condition_immunities: vec![
            Condition::Poisoned,
            Condition::Frightened,
            Condition::Charmed,
            Condition::Paralyzed,
            Condition::Petrified,
        ],
    }
}

/// Create fiend damage modifiers.
pub fn fiend_modifiers() -> DamageModifiers {
    DamageModifiers {
        resistances: vec![DamageType::Cold, DamageType::Fire, DamageType::Lightning],
        vulnerabilities: vec![DamageType::Radiant],
        immunities: vec![DamageType::Poison],
        condition_immunities: vec![Condition::Poisoned],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_modifiers() {
        let mods = undead_modifiers();

        // Immune to poison
        assert_eq!(mods.apply(10, DamageType::Poison), 0);

        // Vulnerable to radiant
        assert_eq!(mods.apply(10, DamageType::Radiant), 20);

        // Normal damage from slashing
        assert_eq!(mods.apply(10, DamageType::Slashing), 10);
    }

    #[test]
    fn test_condition_effects() {
        let poisoned = ActiveEffect::new(Condition::Poisoned, EffectDuration::Rounds(3));
        assert!(poisoned.condition.attack_disadvantage());
        assert!(!poisoned.condition.prevents_actions());
    }

    #[test]
    fn test_status_effects() {
        let mut effects = StatusEffects::default();

        effects.add(ActiveEffect::new(
            Condition::Poisoned,
            EffectDuration::Rounds(3),
        ));
        assert!(effects.has_condition(Condition::Poisoned));
        assert!(effects.has_attack_disadvantage());

        effects.remove_condition(Condition::Poisoned);
        assert!(!effects.has_condition(Condition::Poisoned));
    }
}
