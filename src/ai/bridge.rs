//! AI-to-ECS Bridge - Converts AI responses to ECS components.
//!
//! Uses Ollama's structured outputs for reliable JSON generation.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{Description, Dialogue, GameMap, MonsterKind, Position, Room};

use super::client::OllamaClient;
pub use super::client::OllamaError;
use super::prompts::{self, RoomContent};
use super::schema;

/// Plugin for AI bridge systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<AiBridgeConfig>();
}

/// Configuration for the AI bridge.
#[derive(Resource, Debug, Clone)]
pub struct AiBridgeConfig {
    pub model: String,
    pub ollama_url: String,
    pub enabled: bool,
}

impl Default for AiBridgeConfig {
    fn default() -> Self {
        Self {
            model: "llama3.2".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            enabled: true,
        }
    }
}

/// Raw AI response for room descriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDescriptionResponse {
    pub text: String,
    pub atmosphere: String,
    pub lore: Option<String>,
}

impl From<RoomDescriptionResponse> for Description {
    fn from(response: RoomDescriptionResponse) -> Self {
        Description {
            text: response.text,
            atmosphere: response.atmosphere,
            lore: response.lore,
        }
    }
}

/// Raw AI response for monster dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterDialogueResponse {
    pub current_line: String,
    pub options: Vec<String>,
    pub mood: String,
}

impl From<MonsterDialogueResponse> for Dialogue {
    fn from(response: MonsterDialogueResponse) -> Self {
        Dialogue {
            current_line: response.current_line,
            options: response.options,
            mood: response.mood,
        }
    }
}

/// Raw AI response for improvisational action interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInterpretation {
    pub success: bool,
    pub description: String,
    pub consequence: Option<String>,
    pub damage_to_player: i32,
    pub items_found: Vec<String>,
}

/// Raw AI response for death narration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathNarration {
    pub epitaph: String,
    pub final_words: String,
    pub legacy: String,
}

/// Raw AI response for lore discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreDiscovery {
    pub inscription: Option<String>,
    pub history: String,
    pub warning: Option<String>,
}

/// The AI-to-ECS Bridge handles converting game state to AI prompts
/// and AI responses back to ECS components.
pub struct AiBridge {
    client: OllamaClient,
}

impl AiBridge {
    /// Create a new AI bridge with the specified configuration.
    pub fn new(config: &AiBridgeConfig) -> Self {
        let client = OllamaClient::new(&config.model).with_url(&config.ollama_url);
        Self { client }
    }

    /// Generate a description for a room based on its contents.
    ///
    /// Uses Ollama's structured outputs for reliable JSON generation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bridge = AiBridge::new(&config);
    /// let room = Room::new(0, 0, 10, 8, 3);
    /// let contents = vec![
    ///     RoomContent::Monster(MonsterKind::Orc),
    ///     RoomContent::Item(ItemKind::Chest),
    /// ];
    /// let description = bridge.generate_room_description(&room, &contents).await?;
    /// // description.text = "A dank chamber with moss-covered walls..."
    /// // description.atmosphere = "Oppressive silence"
    /// ```
    pub async fn generate_room_description(
        &self,
        room: &Room,
        contents: &[RoomContent],
    ) -> Result<Description, OllamaError> {
        let prompt = prompts::room_description_prompt(room, contents);
        let response: RoomDescriptionResponse = self
            .client
            .generate_structured(&prompt, schema::room_description_schema())
            .await?;
        Ok(response.into())
    }

    /// Generate dialogue for a monster encounter using structured outputs.
    pub async fn generate_monster_dialogue(
        &self,
        monster: MonsterKind,
        player_health_percent: f32,
    ) -> Result<Dialogue, OllamaError> {
        let prompt = prompts::monster_dialogue_prompt(monster, player_health_percent);
        let response: MonsterDialogueResponse = self
            .client
            .generate_structured(&prompt, schema::monster_dialogue_schema())
            .await?;
        Ok(response.into())
    }

    /// Interpret an improvisational player action using structured outputs.
    pub async fn interpret_action(
        &self,
        action: &str,
        context: &str,
    ) -> Result<ActionInterpretation, OllamaError> {
        let prompt = prompts::interpret_action_prompt(action, context);
        self.client
            .generate_structured(&prompt, schema::action_interpretation_schema())
            .await
    }

    /// Generate a death narration using structured outputs.
    pub async fn generate_death_narration(
        &self,
        cause: &str,
        floor: i32,
    ) -> Result<DeathNarration, OllamaError> {
        let prompt = prompts::death_narration_prompt(cause, floor);
        self.client
            .generate_structured(&prompt, schema::death_narration_schema())
            .await
    }

    /// Generate lore for an item discovery using structured outputs.
    pub async fn generate_lore(
        &self,
        item_name: &str,
        location_description: &str,
    ) -> Result<LoreDiscovery, OllamaError> {
        let prompt = prompts::lore_discovery_prompt(item_name, location_description);
        self.client
            .generate_structured(&prompt, schema::lore_discovery_schema())
            .await
    }

    /// Check if the AI backend is available.
    pub async fn is_available(&self) -> bool {
        self.client.health_check().await
    }
}

/// Build context string from game state for AI prompts.
pub fn build_game_context(
    player_pos: &Position,
    map: &GameMap,
    nearby_entities: &[(String, Position)],
) -> String {
    let room_info = map
        .room_at(player_pos.x, player_pos.y)
        .map(|r| format!("Room {} ({}x{})", r.room_id, r.width, r.height))
        .unwrap_or_else(|| "Corridor".to_string());

    let nearby = nearby_entities
        .iter()
        .map(|(name, pos)| format!("{} at ({}, {})", name, pos.x, pos.y))
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Player at ({}, {}). Location: {}. Floor: {}. Nearby: {}",
        player_pos.x, player_pos.y, room_info, map.current_floor, nearby
    )
}
