//! JSON Schema definitions for structured AI outputs.
//!
//! This module provides JSON schemas that constrain Ollama's responses
//! to match our expected Rust types. Using schemas ensures more reliable
//! and consistent output than basic JSON mode.

use serde_json::{json, Value};

/// JSON schema for MonsterFlavor.
pub fn monster_flavor_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "A unique, evocative name (e.g., 'Calcified Guardian', 'Snaggletooth the Rotting')"
            },
            "visual_desc": {
                "type": "string",
                "description": "2-3 sentences describing appearance, focusing on unsettling details"
            },
            "weapon_flavor": {
                "type": ["string", "null"],
                "description": "Custom name for their weapon (e.g., 'Rusted Pickaxe') or null if unarmed"
            },
            "armor_flavor": {
                "type": ["string", "null"],
                "description": "Custom name for their armor (e.g., 'Moldy Leather Armor') or null if unarmored"
            },
            "personality_trait": {
                "type": ["string", "null"],
                "description": "One-line personality or behavior trait"
            }
        },
        "required": ["name", "visual_desc"]
    })
}

/// JSON schema for WeaponFlavor.
pub fn weapon_flavor_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "A descriptive name (e.g., 'Rusted Pickaxe', 'Bone-Hilted Dagger')"
            },
            "visual_desc": {
                "type": "string",
                "description": "1-2 sentences describing appearance and condition"
            },
            "inscription": {
                "type": ["string", "null"],
                "description": "Any markings, runes, or inscriptions (or null)"
            }
        },
        "required": ["name", "visual_desc"]
    })
}

/// JSON schema for ArmorFlavor.
pub fn armor_flavor_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "A descriptive name (e.g., 'Moldy Leather Armor', 'Bone-Studded Vest')"
            },
            "visual_desc": {
                "type": "string",
                "description": "1-2 sentences describing appearance and condition"
            },
            "features": {
                "type": ["string", "null"],
                "description": "Any notable features or damage (or null)"
            }
        },
        "required": ["name", "visual_desc"]
    })
}

/// JSON schema for ItemFlavor.
pub fn item_flavor_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Custom item name"
            },
            "visual_desc": {
                "type": "string",
                "description": "Visual description of the item"
            },
            "lore": {
                "type": ["string", "null"],
                "description": "Lore or history of the item"
            }
        },
        "required": ["name", "visual_desc"]
    })
}

/// JSON schema for RoomDescriptionResponse.
pub fn room_description_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "Main room description (2-4 sentences)"
            },
            "atmosphere": {
                "type": "string",
                "description": "One-word or short phrase describing the atmosphere"
            },
            "lore": {
                "type": ["string", "null"],
                "description": "Optional lore about the room's history"
            }
        },
        "required": ["text", "atmosphere"]
    })
}

/// JSON schema for MonsterDialogueResponse.
pub fn monster_dialogue_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "current_line": {
                "type": "string",
                "description": "What the monster says"
            },
            "options": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Possible player responses (2-4 options)"
            },
            "mood": {
                "type": "string",
                "description": "The monster's current mood"
            }
        },
        "required": ["current_line", "options", "mood"]
    })
}

/// JSON schema for ActionInterpretation.
pub fn action_interpretation_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "success": {
                "type": "boolean",
                "description": "Whether the action succeeded"
            },
            "description": {
                "type": "string",
                "description": "Narrative description of what happened"
            },
            "consequence": {
                "type": ["string", "null"],
                "description": "Any consequences of the action"
            },
            "damage_to_player": {
                "type": "integer",
                "description": "HP damage taken by player (0 if none)"
            },
            "items_found": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Items discovered by the action"
            }
        },
        "required": ["success", "description", "damage_to_player", "items_found"]
    })
}

/// JSON schema for DeathNarration.
pub fn death_narration_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "epitaph": {
                "type": "string",
                "description": "A fitting epitaph for the fallen adventurer"
            },
            "final_words": {
                "type": "string",
                "description": "The adventurer's last words or thoughts"
            },
            "legacy": {
                "type": "string",
                "description": "What legacy they leave behind"
            }
        },
        "required": ["epitaph", "final_words", "legacy"]
    })
}

/// JSON schema for LoreDiscovery.
pub fn lore_discovery_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "inscription": {
                "type": ["string", "null"],
                "description": "Text of any inscription found"
            },
            "history": {
                "type": "string",
                "description": "Historical context of the discovery"
            },
            "warning": {
                "type": ["string", "null"],
                "description": "Any warnings revealed"
            }
        },
        "required": ["history"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_flavor_schema_valid() {
        let schema = monster_flavor_schema();
        assert!(schema.is_object());
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["visual_desc"].is_object());
    }

    #[test]
    fn test_room_description_schema_valid() {
        let schema = room_description_schema();
        assert!(schema.is_object());
        assert!(schema["required"].is_array());
    }
}
