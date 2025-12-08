//! JSON Schema definitions for structured AI outputs.
//!
//! This module provides JSON schemas that constrain Ollama's responses
//! to match our expected Rust types. Using schemas ensures more reliable
//! and consistent output than basic JSON mode.
//!
//! Schemas are auto-generated from Rust structs using `schemars::JsonSchema`.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::flavor::{ArmorFlavor, ItemFlavor, MonsterFlavor, WeaponFlavor};

/// Get the JSON schema for a type as a serde_json::Value.
pub fn schema_value<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    serde_json::to_value(schema).expect("Schema should serialize to JSON")
}

/// JSON schema for MonsterFlavor (auto-generated).
pub fn monster_flavor_schema() -> Value {
    schema_value::<MonsterFlavor>()
}

/// JSON schema for WeaponFlavor (auto-generated).
pub fn weapon_flavor_schema() -> Value {
    schema_value::<WeaponFlavor>()
}

/// JSON schema for ArmorFlavor (auto-generated).
pub fn armor_flavor_schema() -> Value {
    schema_value::<ArmorFlavor>()
}

/// JSON schema for ItemFlavor (auto-generated).
pub fn item_flavor_schema() -> Value {
    schema_value::<ItemFlavor>()
}

/// AI response for room description.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Description of a dungeon room with atmosphere")]
pub struct RoomDescriptionResponse {
    /// Main room description (2-4 sentences).
    #[schemars(description = "Main room description (2-4 sentences)")]
    pub text: String,
    /// One-word or short phrase describing the atmosphere.
    #[schemars(description = "One-word or short phrase describing the atmosphere")]
    pub atmosphere: String,
    /// Optional lore about the room's history.
    #[schemars(description = "Optional lore about the room's history")]
    pub lore: Option<String>,
}

/// JSON schema for RoomDescriptionResponse.
pub fn room_description_schema() -> Value {
    schema_value::<RoomDescriptionResponse>()
}

/// AI response for monster dialogue.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Dialogue from a monster with player response options")]
pub struct MonsterDialogueResponse {
    /// What the monster says.
    #[schemars(description = "What the monster says")]
    pub current_line: String,
    /// Possible player responses (2-4 options).
    #[schemars(description = "Possible player responses (2-4 options)")]
    pub options: Vec<String>,
    /// The monster's current mood.
    #[schemars(description = "The monster's current mood")]
    pub mood: String,
}

/// JSON schema for MonsterDialogueResponse.
pub fn monster_dialogue_schema() -> Value {
    schema_value::<MonsterDialogueResponse>()
}

/// AI interpretation of a player action.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Interpretation of a player action with narrative consequences")]
pub struct ActionInterpretation {
    /// Whether the action succeeded.
    #[schemars(description = "Whether the action succeeded")]
    pub success: bool,
    /// Narrative description of what happened.
    #[schemars(description = "Narrative description of what happened")]
    pub description: String,
    /// Any consequences of the action.
    #[schemars(description = "Any consequences of the action")]
    pub consequence: Option<String>,
    /// HP damage taken by player (0 if none).
    #[schemars(description = "HP damage taken by player (0 if none)")]
    pub damage_to_player: i32,
    /// Items discovered by the action.
    #[schemars(description = "Items discovered by the action")]
    pub items_found: Vec<String>,
}

/// JSON schema for ActionInterpretation.
pub fn action_interpretation_schema() -> Value {
    schema_value::<ActionInterpretation>()
}

/// AI-generated narration for player death.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Narration for a player character's death")]
pub struct DeathNarration {
    /// A fitting epitaph for the fallen adventurer.
    #[schemars(description = "A fitting epitaph for the fallen adventurer")]
    pub epitaph: String,
    /// The adventurer's last words or thoughts.
    #[schemars(description = "The adventurer's last words or thoughts")]
    pub final_words: String,
    /// What legacy they leave behind.
    #[schemars(description = "What legacy they leave behind")]
    pub legacy: String,
}

/// JSON schema for DeathNarration.
pub fn death_narration_schema() -> Value {
    schema_value::<DeathNarration>()
}

/// AI-generated lore discovery.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Lore discovered in the dungeon")]
pub struct LoreDiscovery {
    /// Text of any inscription found.
    #[schemars(description = "Text of any inscription found")]
    pub inscription: Option<String>,
    /// Historical context of the discovery.
    #[schemars(description = "Historical context of the discovery")]
    pub history: String,
    /// Any warnings revealed.
    #[schemars(description = "Any warnings revealed")]
    pub warning: Option<String>,
}

/// JSON schema for LoreDiscovery.
pub fn lore_discovery_schema() -> Value {
    schema_value::<LoreDiscovery>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_flavor_schema_valid() {
        let schema = monster_flavor_schema();
        assert!(schema.is_object());
        // Auto-generated schemas have different structure
        assert!(schema.get("title").is_some() || schema.get("$schema").is_some());
    }

    #[test]
    fn test_room_description_schema_valid() {
        let schema = room_description_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_schema_contains_properties() {
        let schema = monster_flavor_schema();
        // The schema should have properties for the struct fields
        assert!(schema.get("properties").is_some() || schema.get("definitions").is_some());
    }

    #[test]
    fn test_all_schemas_generate() {
        // Verify all schemas can be generated without panic
        let _ = monster_flavor_schema();
        let _ = weapon_flavor_schema();
        let _ = armor_flavor_schema();
        let _ = item_flavor_schema();
        let _ = room_description_schema();
        let _ = monster_dialogue_schema();
        let _ = action_interpretation_schema();
        let _ = death_narration_schema();
        let _ = lore_discovery_schema();
    }
}
