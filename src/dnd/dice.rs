//! D&D Dice Rolling System.
//!
//! Implements standard D&D dice notation (NdX+M):
//! - N: Number of dice to roll
//! - X: Sides on each die (d4, d6, d8, d10, d12, d20, d100)
//! - M: Modifier to add to the total

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Plugin for dice systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<DiceRoller>();
}

/// Types of dice used in D&D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100, // Percentile dice
}

impl DiceType {
    /// Get the number of sides on this die.
    pub fn sides(&self) -> i32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
            DiceType::D100 => 100,
        }
    }

    /// Get the average roll for this die.
    pub fn average(&self) -> f32 {
        (self.sides() as f32 + 1.0) / 2.0
    }
}

impl fmt::Display for DiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiceType::D4 => write!(f, "d4"),
            DiceType::D6 => write!(f, "d6"),
            DiceType::D8 => write!(f, "d8"),
            DiceType::D10 => write!(f, "d10"),
            DiceType::D12 => write!(f, "d12"),
            DiceType::D20 => write!(f, "d20"),
            DiceType::D100 => write!(f, "d100"),
        }
    }
}

/// A dice expression (e.g., "2d6+3").
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub struct Dice {
    /// Number of dice to roll.
    pub count: i32,
    /// Type of dice.
    pub die_type: DiceType,
    /// Modifier to add to the total.
    pub modifier: i32,
}

impl Dice {
    /// Create a new dice expression.
    pub fn new(count: i32, die_type: DiceType, modifier: i32) -> Self {
        Self {
            count,
            die_type,
            modifier,
        }
    }

    /// Create a simple NdX dice expression (no modifier).
    pub fn simple(count: i32, die_type: DiceType) -> Self {
        Self::new(count, die_type, 0)
    }

    /// Create a 1dX dice expression.
    pub fn single(die_type: DiceType) -> Self {
        Self::simple(1, die_type)
    }

    /// Create a d20 roll (most common in D&D).
    pub fn d20() -> Self {
        Self::single(DiceType::D20)
    }

    /// Roll the dice and return the result.
    pub fn roll(&self) -> DiceRoll {
        let mut rng = rand::rng();
        self.roll_with(&mut rng)
    }

    /// Roll the dice with a specific RNG.
    pub fn roll_with(&self, rng: &mut impl Rng) -> DiceRoll {
        let sides = self.die_type.sides();
        let mut rolls = Vec::with_capacity(self.count as usize);

        for _ in 0..self.count {
            rolls.push(rng.random_range(1..=sides));
        }

        let dice_total: i32 = rolls.iter().sum();
        let total = dice_total + self.modifier;

        DiceRoll {
            dice: self.clone(),
            rolls,
            total,
        }
    }

    /// Get the minimum possible roll.
    pub fn minimum(&self) -> i32 {
        self.count + self.modifier
    }

    /// Get the maximum possible roll.
    pub fn maximum(&self) -> i32 {
        self.count * self.die_type.sides() + self.modifier
    }

    /// Get the average roll.
    pub fn average(&self) -> f32 {
        self.count as f32 * self.die_type.average() + self.modifier as f32
    }

    /// Parse a dice string like "2d6+3" or "1d8-1".
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();

        // Find the 'd' separator
        let d_pos = s.find('d')?;

        // Parse count (default to 1 if empty)
        let count_str = &s[..d_pos];
        let count = if count_str.is_empty() {
            1
        } else {
            count_str.parse().ok()?
        };

        // Find modifier (+ or -)
        let rest = &s[d_pos + 1..];
        let (die_str, modifier) = if let Some(plus_pos) = rest.find('+') {
            let die_str = &rest[..plus_pos];
            let mod_str = &rest[plus_pos + 1..];
            (die_str, mod_str.parse::<i32>().ok()?)
        } else if let Some(minus_pos) = rest.rfind('-') {
            let die_str = &rest[..minus_pos];
            let mod_str = &rest[minus_pos..]; // Includes the minus sign
            (die_str, mod_str.parse::<i32>().ok()?)
        } else {
            (rest, 0)
        };

        // Parse die type
        let die_type = match die_str.parse::<i32>().ok()? {
            4 => DiceType::D4,
            6 => DiceType::D6,
            8 => DiceType::D8,
            10 => DiceType::D10,
            12 => DiceType::D12,
            20 => DiceType::D20,
            100 => DiceType::D100,
            _ => return None,
        };

        Some(Self::new(count, die_type, modifier))
    }
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifier > 0 {
            write!(f, "{}{}+{}", self.count, self.die_type, self.modifier)
        } else if self.modifier < 0 {
            write!(f, "{}{}{}", self.count, self.die_type, self.modifier)
        } else {
            write!(f, "{}{}", self.count, self.die_type)
        }
    }
}

/// Result of rolling dice.
#[derive(Debug, Clone)]
pub struct DiceRoll {
    /// The dice expression that was rolled.
    pub dice: Dice,
    /// Individual roll results.
    pub rolls: Vec<i32>,
    /// Total after adding modifier.
    pub total: i32,
}

impl DiceRoll {
    /// Check if this was a natural 20 (for d20 rolls).
    pub fn is_natural_20(&self) -> bool {
        self.dice.die_type == DiceType::D20 && self.dice.count == 1 && self.rolls.first() == Some(&20)
    }

    /// Check if this was a natural 1 (for d20 rolls).
    pub fn is_natural_1(&self) -> bool {
        self.dice.die_type == DiceType::D20 && self.dice.count == 1 && self.rolls.first() == Some(&1)
    }

    /// Get the sum of just the dice (without modifier).
    pub fn dice_total(&self) -> i32 {
        self.rolls.iter().sum()
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {} ({:?})", self.dice, self.total, self.rolls)
    }
}

/// Resource for dice rolling operations.
#[derive(Resource)]
pub struct DiceRoller {
    rng: rand::prelude::SmallRng,
}

impl Default for DiceRoller {
    fn default() -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::prelude::SmallRng::from_os_rng(),
        }
    }
}

impl DiceRoller {
    /// Roll a dice expression.
    pub fn roll(&mut self, dice: &Dice) -> DiceRoll {
        dice.roll_with(&mut self.rng)
    }

    /// Roll a d20.
    pub fn roll_d20(&mut self) -> DiceRoll {
        Dice::d20().roll_with(&mut self.rng)
    }

    /// Roll a d20 with advantage (roll twice, take highest).
    pub fn roll_d20_advantage(&mut self) -> DiceRoll {
        let roll1 = self.roll_d20();
        let roll2 = self.roll_d20();
        if roll1.total >= roll2.total {
            roll1
        } else {
            roll2
        }
    }

    /// Roll a d20 with disadvantage (roll twice, take lowest).
    pub fn roll_d20_disadvantage(&mut self) -> DiceRoll {
        let roll1 = self.roll_d20();
        let roll2 = self.roll_d20();
        if roll1.total <= roll2.total {
            roll1
        } else {
            roll2
        }
    }

    /// Roll a single die of the specified type.
    pub fn roll_die(&mut self, die_type: DiceType) -> i32 {
        self.rng.random_range(1..=die_type.sides())
    }

    /// Roll NdX dice.
    pub fn roll_dice(&mut self, count: i32, die_type: DiceType) -> DiceRoll {
        Dice::simple(count, die_type).roll_with(&mut self.rng)
    }

    /// Roll NdX+M dice.
    pub fn roll_dice_with_modifier(
        &mut self,
        count: i32,
        die_type: DiceType,
        modifier: i32,
    ) -> DiceRoll {
        Dice::new(count, die_type, modifier).roll_with(&mut self.rng)
    }
}

/// Helper function to quickly roll dice without a resource.
pub fn roll(dice: &Dice) -> DiceRoll {
    dice.roll()
}

/// Helper function to roll a d20.
pub fn roll_d20() -> DiceRoll {
    Dice::d20().roll()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_parse() {
        let dice = Dice::parse("2d6").unwrap();
        assert_eq!(dice.count, 2);
        assert_eq!(dice.die_type, DiceType::D6);
        assert_eq!(dice.modifier, 0);

        let dice = Dice::parse("1d8+3").unwrap();
        assert_eq!(dice.count, 1);
        assert_eq!(dice.die_type, DiceType::D8);
        assert_eq!(dice.modifier, 3);

        let dice = Dice::parse("d20-1").unwrap();
        assert_eq!(dice.count, 1);
        assert_eq!(dice.die_type, DiceType::D20);
        assert_eq!(dice.modifier, -1);
    }

    #[test]
    fn test_dice_display() {
        assert_eq!(Dice::simple(2, DiceType::D6).to_string(), "2d6");
        assert_eq!(Dice::new(1, DiceType::D8, 3).to_string(), "1d8+3");
        assert_eq!(Dice::new(1, DiceType::D20, -1).to_string(), "1d20-1");
    }

    #[test]
    fn test_dice_bounds() {
        let dice = Dice::new(2, DiceType::D6, 3);
        assert_eq!(dice.minimum(), 5); // 2 (2x1) + 3
        assert_eq!(dice.maximum(), 15); // 12 (2x6) + 3
    }

    #[test]
    fn test_roll_bounds() {
        let dice = Dice::new(2, DiceType::D6, 0);
        for _ in 0..100 {
            let roll = dice.roll();
            assert!(roll.total >= dice.minimum());
            assert!(roll.total <= dice.maximum());
        }
    }
}
