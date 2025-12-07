//! AI module - Ollama integration and narrative generation.

mod bridge;
mod client;
mod prompts;
mod worker;

pub use worker::{
    NarrativeRequest, NarrativeRequestKind, NarrativeResponse, NarrativeResponseKind,
    RoomContentData,
};

use bevy::prelude::*;

/// Plugin for AI systems and resources.
pub fn plugin(app: &mut App) {
    app.add_plugins((bridge::plugin, worker::plugin));
}
