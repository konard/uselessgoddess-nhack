//! AI Flavor Generation - The "Constructive DM" pattern.
//!
//! The AI generates *flavor* (names, descriptions, appearances) while Rust
//! controls the *mechanics* (stats, damage, AC). This separation ensures:
//! - AI creativity for immersion
//! - Deterministic game balance
//! - Type-safe ECS integration
//!
//! This module uses Ollama's structured outputs feature for reliable JSON generation.
//! See: <https://ollama.com/blog/structured-outputs>

use bevy::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dnd::armor::{ArmorDatabase, ArmorType};
use crate::dnd::monsters::{MonsterDatabase, MonsterTemplate, MonsterType};
use crate::dnd::weapons::{WeaponDatabase, WeaponType};

use super::bridge::AiBridgeConfig;
use super::client::{OllamaClient, OllamaError};
use super::schema;

/// Plugin for AI flavor generation.
pub fn plugin(app: &mut App) {
    app.add_message::<FlavorRequestMessage>()
        .add_message::<FlavorResponseMessage>();
}

/// Request for AI-generated entity flavor.
#[derive(Message, Debug, Clone)]
pub struct FlavorRequestMessage {
    /// Unique request ID for tracking.
    pub request_id: u64,
    /// What kind of flavor to generate.
    pub kind: FlavorRequestKind,
    /// Target entity to apply flavor to.
    pub target_entity: Option<Entity>,
}

/// Types of flavor requests.
#[derive(Debug, Clone)]
pub enum FlavorRequestKind {
    /// Generate flavor for a monster.
    /// Input: "Spawn a CR 1/4 Skeleton"
    /// Output: Custom name, visual description, weapon flavor
    Monster {
        monster_type: MonsterType,
    },
    /// Generate flavor for a weapon.
    Weapon {
        weapon_type: WeaponType,
        context: String, // e.g., "found in skeleton's hand"
    },
    /// Generate flavor for armor.
    Armor {
        armor_type: ArmorType,
        context: String,
    },
    /// Generate a custom item description.
    Item {
        item_name: String,
        context: String,
    },
}

/// Response with AI-generated flavor.
#[derive(Message, Debug, Clone)]
pub struct FlavorResponseMessage {
    /// Matching request ID.
    pub request_id: u64,
    /// The generated flavor.
    pub kind: FlavorResponseKind,
    /// Target entity to apply to.
    pub target_entity: Option<Entity>,
}

/// Types of flavor responses.
#[derive(Debug, Clone)]
pub enum FlavorResponseKind {
    Monster(MonsterFlavor),
    Weapon(WeaponFlavor),
    Armor(ArmorFlavor),
    Item(ItemFlavor),
    Error(String),
}

/// AI-generated flavor for a monster.
///
/// Example JSON from AI:
/// ```json
/// {
///     "name": "Calcified Guardian",
///     "visual_desc": "Bones fused with iron ore, eye sockets glow with pale fire",
///     "weapon_flavor": "Rusted Pickaxe",
///     "armor_flavor": null,
///     "personality_trait": "Territorial, guards ancient mining shaft"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "AI-generated flavor for a monster with custom name and atmospheric descriptions")]
pub struct MonsterFlavor {
    /// A unique, evocative name (e.g., "Calcified Guardian", "Snaggletooth the Rotting").
    #[schemars(description = "A unique, evocative name (e.g., 'Calcified Guardian', 'Snaggletooth the Rotting')")]
    pub name: String,
    /// 2-3 sentences describing appearance, focusing on unsettling details.
    #[schemars(description = "2-3 sentences describing appearance, focusing on unsettling details")]
    pub visual_desc: String,
    /// Custom name for their weapon (e.g., "Rusted Pickaxe") or null if unarmed.
    #[schemars(description = "Custom name for their weapon (e.g., 'Rusted Pickaxe') or null if unarmed")]
    pub weapon_flavor: Option<String>,
    /// Custom name for their armor (e.g., "Moldy Leather Armor") or null if unarmored.
    #[schemars(description = "Custom name for their armor (e.g., 'Moldy Leather Armor') or null if unarmored")]
    pub armor_flavor: Option<String>,
    /// One-line personality or behavior trait.
    #[schemars(description = "One-line personality or behavior trait")]
    pub personality_trait: Option<String>,
}

/// AI-generated flavor for a weapon.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "AI-generated flavor for a weapon with custom name and atmospheric description")]
pub struct WeaponFlavor {
    /// A descriptive name (e.g., "Rusted Pickaxe", "Bone-Hilted Dagger").
    #[schemars(description = "A descriptive name (e.g., 'Rusted Pickaxe', 'Bone-Hilted Dagger')")]
    pub name: String,
    /// 1-2 sentences describing appearance and condition.
    #[schemars(description = "1-2 sentences describing appearance and condition")]
    pub visual_desc: String,
    /// Any markings, runes, or inscriptions (or null).
    #[schemars(description = "Any markings, runes, or inscriptions (or null)")]
    pub inscription: Option<String>,
}

/// AI-generated flavor for armor.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "AI-generated flavor for armor with custom name and atmospheric description")]
pub struct ArmorFlavor {
    /// A descriptive name (e.g., "Moldy Leather Armor", "Bone-Studded Vest").
    #[schemars(description = "A descriptive name (e.g., 'Moldy Leather Armor', 'Bone-Studded Vest')")]
    pub name: String,
    /// 1-2 sentences describing appearance and condition.
    #[schemars(description = "1-2 sentences describing appearance and condition")]
    pub visual_desc: String,
    /// Any notable features or damage (or null).
    #[schemars(description = "Any notable features or damage (or null)")]
    pub features: Option<String>,
}

/// AI-generated flavor for a generic item.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "AI-generated flavor for a generic item with custom name and lore")]
pub struct ItemFlavor {
    /// Custom item name.
    #[schemars(description = "Custom item name")]
    pub name: String,
    /// Visual description of the item.
    #[schemars(description = "Visual description of the item")]
    pub visual_desc: String,
    /// Lore or history of the item.
    #[schemars(description = "Lore or history of the item")]
    pub lore: Option<String>,
}

/// The Flavor Engine - generates entity flavor via AI.
pub struct FlavorEngine {
    client: OllamaClient,
}

impl FlavorEngine {
    /// Create a new flavor engine with configuration.
    pub fn new(config: &AiBridgeConfig) -> Self {
        let client = OllamaClient::new(&config.model).with_url(&config.ollama_url);
        Self { client }
    }

    /// Generate flavor for a monster using structured outputs.
    ///
    /// Uses Ollama's structured outputs feature to ensure the AI response
    /// matches the expected JSON schema. This is more reliable than basic
    /// JSON mode as the model's output is constrained by the schema.
    pub async fn generate_monster_flavor(
        &self,
        monster_type: MonsterType,
    ) -> Result<MonsterFlavor, OllamaError> {
        let data = monster_type.data();

        let system_msg = "You are a horror-themed Dungeon Master generating flavor for D&D monsters. \
            Be creative and unsettling. Focus on sensory details that create atmosphere.";

        let prompt = format!(
            "Generate unique, atmospheric flavor for this creature:\n\n\
            Monster Type: {} (CR {})\n\
            Creature Type: {:?}\n\
            Size: {:?}\n\n\
            The game uses D&D mechanical stats (HP, AC, damage) - you only provide narrative flavor.",
            data.name,
            data.challenge_rating.display(),
            data.creature_type,
            data.size,
        );

        self.client
            .generate_structured_with_system(system_msg, &prompt, schema::monster_flavor_schema())
            .await
    }

    /// Generate flavor for a weapon found in the world using structured outputs.
    pub async fn generate_weapon_flavor(
        &self,
        weapon_type: WeaponType,
        context: &str,
    ) -> Result<WeaponFlavor, OllamaError> {
        let data = weapon_type.base_data();

        let system_msg = "You are a horror-themed Dungeon Master generating flavor for weapons. \
            Make descriptions atmospheric and slightly unsettling.";

        let prompt = format!(
            "Generate atmospheric flavor for this weapon:\n\n\
            Weapon Type: {} ({}d{} {} damage)\n\
            Context: {}\n\n\
            The game uses D&D mechanics - you provide narrative flavor only.",
            data.name,
            data.damage.count,
            data.damage.die_type,
            data.damage_type.name(),
            context,
        );

        self.client
            .generate_structured_with_system(system_msg, &prompt, schema::weapon_flavor_schema())
            .await
    }

    /// Generate flavor for armor using structured outputs.
    pub async fn generate_armor_flavor(
        &self,
        armor_type: ArmorType,
        context: &str,
    ) -> Result<ArmorFlavor, OllamaError> {
        let data = armor_type.base_data();

        let system_msg = "You are a horror-themed Dungeon Master generating flavor for armor. \
            Make descriptions atmospheric and slightly unsettling.";

        let prompt = format!(
            "Generate atmospheric flavor for this armor:\n\n\
            Armor Type: {} (AC {}, {:?})\n\
            Context: {}",
            data.name,
            data.base_ac,
            data.category,
            context,
        );

        self.client
            .generate_structured_with_system(system_msg, &prompt, schema::armor_flavor_schema())
            .await
    }
}

/// Map AI-generated weapon flavor back to actual weapon type.
///
/// Example: "Rusted Pickaxe" -> WeaponType::WarPick
pub fn map_weapon_flavor_to_type(
    flavor_name: &str,
    weapon_db: &WeaponDatabase,
) -> Option<WeaponType> {
    // Try to find a matching weapon type
    // The flavor name might not match exactly, so we try partial matching
    weapon_db.find_by_name(flavor_name)
}

/// Map AI-generated armor flavor back to actual armor type.
pub fn map_armor_flavor_to_type(
    flavor_name: &str,
    armor_db: &ArmorDatabase,
) -> Option<ArmorType> {
    armor_db.find_by_name(flavor_name)
}

/// Apply monster flavor to a MonsterTemplate component.
pub fn apply_monster_flavor(template: &mut MonsterTemplate, flavor: &MonsterFlavor) {
    template.custom_name = Some(flavor.name.clone());
    template.visual_description = Some(flavor.visual_desc.clone());
    template.weapon_flavor = flavor.weapon_flavor.clone();
    template.armor_flavor = flavor.armor_flavor.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_flavor_deserialization() {
        let json = r#"{
            "name": "Calcified Guardian",
            "visual_desc": "Bones fused with iron ore",
            "weapon_flavor": "Rusted Pickaxe",
            "armor_flavor": null,
            "personality_trait": "Territorial"
        }"#;

        let flavor: MonsterFlavor = serde_json::from_str(json).unwrap();
        assert_eq!(flavor.name, "Calcified Guardian");
        assert!(flavor.weapon_flavor.is_some());
        assert!(flavor.armor_flavor.is_none());
    }

    #[test]
    fn test_weapon_mapping() {
        let db = WeaponDatabase::default();

        // Direct match
        assert!(map_weapon_flavor_to_type("Longsword", &db).is_some());

        // Partial match (pickaxe -> war pick)
        assert!(map_weapon_flavor_to_type("Pick", &db).is_some());
    }
}
