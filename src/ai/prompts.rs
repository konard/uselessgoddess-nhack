//! Prompt templates for AI narrative generation.

use crate::game::{ItemKind, MonsterKind, Room};

/// System prompt establishing the AI Game Master persona.
pub const SYSTEM_PROMPT: &str = r#"You are the Game Master for NeuroHack, a horror-themed roguelike dungeon crawler.
Your role is to generate atmospheric, creepy descriptions that enhance the player's experience.
Always maintain a dark, foreboding tone reminiscent of Lovecraftian horror.
Be descriptive but concise - players need information quickly.
Focus on sensory details: sounds, smells, textures, and the feeling of dread."#;

/// Generate a prompt for room description.
pub fn room_description_prompt(room: &Room, contents: &[RoomContent]) -> String {
    let contents_str = contents
        .iter()
        .map(|c| match c {
            RoomContent::Monster(kind) => format!("Monster: {}", kind.name()),
            RoomContent::Item(kind) => format!("Item: {}", kind.name()),
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"{SYSTEM_PROMPT}

Generate a horror-themed description for a dungeon room.

Room details:
- Room ID: {}
- Size: {}x{}
- Contents: [{}]

Respond with ONLY a JSON object in this exact format:
{{
    "text": "A 1-2 sentence description of the room's appearance",
    "atmosphere": "A short phrase describing the feeling/atmosphere",
    "lore": "Optional hint about the room's history or a dark secret (null if none)"
}}

Be creative but stay within the horror theme. The description should make the player feel uneasy."#,
        room.room_id, room.width, room.height, contents_str
    )
}

/// Content types that can be in a room.
pub enum RoomContent {
    Monster(MonsterKind),
    Item(ItemKind),
}

/// Generate a prompt for monster encounter dialogue.
pub fn monster_dialogue_prompt(monster: MonsterKind, player_health_percent: f32) -> String {
    let health_state = if player_health_percent > 0.7 {
        "healthy and strong"
    } else if player_health_percent > 0.3 {
        "wounded but standing"
    } else {
        "near death, barely conscious"
    };

    format!(
        r#"{SYSTEM_PROMPT}

A player encounters a {} in the dungeon. The player appears {}.

Generate dialogue/sounds for this monster encounter.

Respond with ONLY a JSON object in this exact format:
{{
    "current_line": "What the monster says, growls, or the sound it makes",
    "options": ["Possible player response 1", "Possible player response 2"],
    "mood": "The monster's current disposition (aggressive, curious, hungry, etc.)"
}}

Make it atmospheric and unsettling. Monsters can speak in broken common, ancient tongues, or just make threatening sounds."#,
        monster.name(),
        health_state
    )
}

/// Generate a prompt for interpreting improvisational player actions.
pub fn interpret_action_prompt(action: &str, context: &str) -> String {
    format!(
        r#"{SYSTEM_PROMPT}

The player attempts an unusual action in the dungeon.

Current context: {}
Player's action: "{}"

Determine if this action is:
1. Possible given the environment
2. What the outcome should be
3. Any consequences or discoveries

Respond with ONLY a JSON object in this exact format:
{{
    "success": true or false,
    "description": "What happens when the player attempts this",
    "consequence": "Any lasting effect or discovery (null if none)",
    "damage_to_player": 0 (number, if the action hurts them),
    "items_found": [] (array of item names discovered, empty if none)
}}

Be fair but maintain the horror atmosphere. Creative actions should sometimes be rewarded."#,
        context, action
    )
}

/// Generate a prompt for death narration.
pub fn death_narration_prompt(cause: &str, floor: i32) -> String {
    format!(
        r#"{SYSTEM_PROMPT}

The player has died in the dungeon.

Cause of death: {}
Dungeon floor reached: {}

Write a dramatic, haunting epitaph for this fallen adventurer.

Respond with ONLY a JSON object in this exact format:
{{
    "epitaph": "A 2-3 sentence dramatic description of the player's demise",
    "final_words": "The last thought that crossed their mind",
    "legacy": "What whispers will tell of this adventurer"
}}

Make it memorable and appropriately grim."#,
        cause, floor
    )
}

/// Generate a prompt for discovering lore.
pub fn lore_discovery_prompt(item_name: &str, location_description: &str) -> String {
    format!(
        r#"{SYSTEM_PROMPT}

The player examines an interesting object in the dungeon.

Object: {}
Found in: {}

Generate mysterious lore about this object.

Respond with ONLY a JSON object in this exact format:
{{
    "inscription": "Any writing or symbols on the object (null if none)",
    "history": "A brief, cryptic hint about the object's origin",
    "warning": "An ominous feeling or premonition (null if benign)"
}}

Keep it mysterious. Lore should hint at a larger, darker story."#,
        item_name, location_description
    )
}
