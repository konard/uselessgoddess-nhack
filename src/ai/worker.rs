//! Background AI worker for non-blocking narrative generation.

use bevy::prelude::*;
use tokio::sync::mpsc;

use super::bridge::{
    ActionInterpretation, AiBridge, AiBridgeConfig, DeathNarration, LoreDiscovery,
};
use super::prompts::RoomContent;
use crate::game::{Description, Dialogue, ItemKind, MonsterKind, Room};

/// Plugin for the AI worker system.
pub fn plugin(app: &mut App) {
    app.add_message::<NarrativeRequest>()
        .add_message::<NarrativeResponse>()
        .init_resource::<AiWorkerChannels>()
        .add_systems(Startup, spawn_ai_worker)
        .add_systems(Update, (send_requests_to_worker, receive_responses_from_worker));
}

/// Requests that can be sent to the AI worker.
#[derive(Debug, Clone)]
pub enum NarrativeRequestKind {
    RoomDescription {
        room: Room,
        contents: Vec<RoomContentData>,
    },
    MonsterDialogue {
        monster: MonsterKind,
        player_health_percent: f32,
    },
    InterpretAction {
        action: String,
        context: String,
    },
    DeathNarration {
        cause: String,
        floor: i32,
    },
    LoreDiscovery {
        item_name: String,
        location_description: String,
    },
}

/// Serializable version of RoomContent for sending through channels.
#[derive(Debug, Clone)]
pub enum RoomContentData {
    Monster(MonsterKind),
    Item(ItemKind),
}

impl From<&RoomContentData> for RoomContent {
    fn from(data: &RoomContentData) -> Self {
        match data {
            RoomContentData::Monster(kind) => RoomContent::Monster(*kind),
            RoomContentData::Item(kind) => RoomContent::Item(*kind),
        }
    }
}

/// Message for requesting AI-generated narrative.
#[derive(Message, Debug, Clone)]
pub struct NarrativeRequest {
    pub id: u64,
    pub kind: NarrativeRequestKind,
    pub target_entity: Option<Entity>,
}

/// Responses from the AI worker.
#[derive(Debug, Clone)]
pub enum NarrativeResponseKind {
    RoomDescription(Description),
    MonsterDialogue(Dialogue),
    ActionResult(ActionInterpretation),
    DeathNarration(DeathNarration),
    LoreDiscovery(LoreDiscovery),
    Error(String),
}

/// Message for receiving AI-generated narrative.
#[derive(Message, Debug, Clone)]
pub struct NarrativeResponse {
    pub request_id: u64,
    pub kind: NarrativeResponseKind,
    pub target_entity: Option<Entity>,
}

/// Internal message type for the worker channel.
#[derive(Debug)]
struct WorkerRequest {
    id: u64,
    kind: NarrativeRequestKind,
    target_entity: Option<Entity>,
}

/// Channels for communicating with the AI worker.
#[derive(Resource)]
pub struct AiWorkerChannels {
    request_tx: mpsc::Sender<WorkerRequest>,
    response_rx: mpsc::Receiver<NarrativeResponse>,
    #[allow(dead_code)]
    next_request_id: u64,
}

impl Default for AiWorkerChannels {
    fn default() -> Self {
        // Create channels with reasonable buffer sizes
        let (request_tx, _) = mpsc::channel(32);
        let (_, response_rx) = mpsc::channel(32);

        Self {
            request_tx,
            response_rx,
            next_request_id: 0,
        }
    }
}

/// Spawns the background AI worker task.
fn spawn_ai_worker(mut commands: Commands, config: Res<AiBridgeConfig>) {
    let (request_tx, mut request_rx) = mpsc::channel::<WorkerRequest>(32);
    let (response_tx, response_rx) = mpsc::channel::<NarrativeResponse>(32);

    let config = config.clone();

    // Spawn the tokio runtime and worker task
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async move {
            let bridge = AiBridge::new(&config);

            // Check if AI is available
            if !bridge.is_available().await {
                tracing::warn!("Ollama is not available - AI features will use fallback responses");
            }

            while let Some(request) = request_rx.recv().await {
                let response = process_request(&bridge, &request).await;

                if response_tx.send(response).await.is_err() {
                    tracing::error!("Failed to send response - main thread disconnected");
                    break;
                }
            }
        });
    });

    commands.insert_resource(AiWorkerChannels {
        request_tx,
        response_rx,
        next_request_id: 0,
    });
}

/// Process a single request using the AI bridge.
async fn process_request(bridge: &AiBridge, request: &WorkerRequest) -> NarrativeResponse {
    let kind = match &request.kind {
        NarrativeRequestKind::RoomDescription { room, contents } => {
            let contents_ref: Vec<RoomContent> = contents.iter().map(|c| c.into()).collect();
            match bridge.generate_room_description(room, &contents_ref).await {
                Ok(desc) => NarrativeResponseKind::RoomDescription(desc),
                Err(e) => NarrativeResponseKind::Error(e.to_string()),
            }
        }
        NarrativeRequestKind::MonsterDialogue {
            monster,
            player_health_percent,
        } => match bridge
            .generate_monster_dialogue(*monster, *player_health_percent)
            .await
        {
            Ok(dialogue) => NarrativeResponseKind::MonsterDialogue(dialogue),
            Err(e) => NarrativeResponseKind::Error(e.to_string()),
        },
        NarrativeRequestKind::InterpretAction { action, context } => {
            match bridge.interpret_action(action, context).await {
                Ok(result) => NarrativeResponseKind::ActionResult(result),
                Err(e) => NarrativeResponseKind::Error(e.to_string()),
            }
        }
        NarrativeRequestKind::DeathNarration { cause, floor } => {
            match bridge.generate_death_narration(cause, *floor).await {
                Ok(narration) => NarrativeResponseKind::DeathNarration(narration),
                Err(e) => NarrativeResponseKind::Error(e.to_string()),
            }
        }
        NarrativeRequestKind::LoreDiscovery {
            item_name,
            location_description,
        } => match bridge.generate_lore(item_name, location_description).await {
            Ok(lore) => NarrativeResponseKind::LoreDiscovery(lore),
            Err(e) => NarrativeResponseKind::Error(e.to_string()),
        },
    };

    NarrativeResponse {
        request_id: request.id,
        kind,
        target_entity: request.target_entity,
    }
}

/// System to send narrative requests to the worker.
fn send_requests_to_worker(
    mut messages: MessageReader<NarrativeRequest>,
    channels: Res<AiWorkerChannels>,
) {
    for message in messages.read() {
        let request = WorkerRequest {
            id: message.id,
            kind: message.kind.clone(),
            target_entity: message.target_entity,
        };

        // Try to send without blocking
        if channels.request_tx.try_send(request).is_err() {
            tracing::warn!("AI worker queue full - request dropped");
        }
    }
}

/// System to receive responses from the worker.
fn receive_responses_from_worker(
    mut channels: ResMut<AiWorkerChannels>,
    mut messages: MessageWriter<NarrativeResponse>,
) {
    // Non-blocking receive of all available responses
    while let Ok(response) = channels.response_rx.try_recv() {
        messages.write(response);
    }
}

/// Helper to create a room description request.
#[allow(dead_code)]
pub fn request_room_description(
    request_tx: &mut MessageWriter<NarrativeRequest>,
    id: u64,
    room: Room,
    contents: Vec<RoomContentData>,
    target_entity: Option<Entity>,
) {
    request_tx.write(NarrativeRequest {
        id,
        kind: NarrativeRequestKind::RoomDescription { room, contents },
        target_entity,
    });
}

/// Helper to create a monster dialogue request.
#[allow(dead_code)]
pub fn request_monster_dialogue(
    request_tx: &mut MessageWriter<NarrativeRequest>,
    id: u64,
    monster: MonsterKind,
    player_health_percent: f32,
    target_entity: Option<Entity>,
) {
    request_tx.write(NarrativeRequest {
        id,
        kind: NarrativeRequestKind::MonsterDialogue {
            monster,
            player_health_percent,
        },
        target_entity,
    });
}
