//! Gameplay screen systems.

use bevy::prelude::*;
use rand::Rng;

use super::GameScreen;
use crate::ai::{
    NarrativeRequest, NarrativeRequestKind, NarrativeResponse, NarrativeResponseKind,
    RoomContentData,
};
use crate::game::{
    CombatStats, GameLog, GameMap, Health, Item, ItemKind, Monster, MonsterKind,
    Player, Position, Renderable, RenderColor,
};
use crate::tui::CurrentNarrative;

/// Plugin for gameplay systems.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameScreen::Playing), setup_gameplay)
        .add_systems(
            Update,
            (
                check_room_entry,
                handle_narrative_responses,
                check_player_death,
            )
                .run_if(in_state(GameScreen::Playing)),
        );
}

/// Resource to track the last room the player was in.
#[derive(Resource, Default)]
struct LastRoom {
    room_id: Option<usize>,
}

/// Set up the initial game state.
fn setup_gameplay(
    mut commands: Commands,
    mut map: ResMut<GameMap>,
    mut log: ResMut<GameLog>,
) {
    // Initialize game log
    *log = GameLog::new(100);
    log.add("Welcome to NeuroHack. The dungeon awaits...");
    log.add("Use hjkl or arrow keys to move.");

    // Generate dungeon
    let mut rng = rand::rng();
    map.generate_dungeon(&mut rng);

    // Spawn player in the first room
    let (player_x, player_y) = map.rooms.first().map(|r| r.center()).unwrap_or((40, 12));

    commands.spawn((
        Name::new("Player"),
        Player,
        Position::new(player_x, player_y),
        Health::new(30),
        CombatStats::new(5, 2),
        Renderable {
            glyph: '@',
            fg_color: RenderColor::Yellow,
            bg_color: RenderColor::Reset,
        },
    ));

    // Compute initial FOV
    map.compute_fov(player_x, player_y, 8);

    // Spawn monsters in rooms (skip the first room where player spawns)
    for room in map.rooms.iter().skip(1) {
        // 60% chance of monster in each room
        if rng.random_bool(0.6) {
            let (cx, cy) = room.center();
            let monster_kind = random_monster(&mut rng);

            commands.spawn((
                Name::new(monster_kind.name()),
                Monster { kind: monster_kind },
                Position::new(cx, cy),
                Health::new(monster_health(monster_kind)),
                CombatStats::new(monster_attack(monster_kind), monster_defense(monster_kind)),
                Renderable {
                    glyph: monster_kind.glyph(),
                    fg_color: RenderColor::Red,
                    bg_color: RenderColor::Reset,
                },
            ));
        }

        // 40% chance of item in each room
        if rng.random_bool(0.4) {
            let (cx, cy) = room.center();
            // Offset item from center so it doesn't overlap with monster
            let item_x = cx + rng.random_range(-2..=2);
            let item_y = cy + rng.random_range(-2..=2);
            let item_kind = random_item(&mut rng);

            commands.spawn((
                Name::new(item_kind.name()),
                Item { kind: item_kind },
                Position::new(item_x, item_y),
                Renderable {
                    glyph: item_kind.glyph(),
                    fg_color: RenderColor::Blue,
                    bg_color: RenderColor::Reset,
                },
            ));
        }
    }

    // Initialize room tracking
    commands.insert_resource(LastRoom { room_id: None });

    log.add("You descend into the darkness...");
}

fn random_monster(rng: &mut impl Rng) -> MonsterKind {
    match rng.random_range(0..6) {
        0 => MonsterKind::Orc,
        1 => MonsterKind::Goblin,
        2 => MonsterKind::Troll,
        3 => MonsterKind::Wraith,
        4 => MonsterKind::Skeleton,
        _ => MonsterKind::Demon,
    }
}

fn monster_health(kind: MonsterKind) -> i32 {
    match kind {
        MonsterKind::Goblin => 5,
        MonsterKind::Orc => 10,
        MonsterKind::Skeleton => 8,
        MonsterKind::Wraith => 12,
        MonsterKind::Troll => 20,
        MonsterKind::Demon => 25,
    }
}

fn monster_attack(kind: MonsterKind) -> i32 {
    match kind {
        MonsterKind::Goblin => 2,
        MonsterKind::Orc => 4,
        MonsterKind::Skeleton => 3,
        MonsterKind::Wraith => 5,
        MonsterKind::Troll => 6,
        MonsterKind::Demon => 8,
    }
}

fn monster_defense(kind: MonsterKind) -> i32 {
    match kind {
        MonsterKind::Goblin => 0,
        MonsterKind::Orc => 1,
        MonsterKind::Skeleton => 1,
        MonsterKind::Wraith => 2,
        MonsterKind::Troll => 3,
        MonsterKind::Demon => 4,
    }
}

fn random_item(rng: &mut impl Rng) -> ItemKind {
    match rng.random_range(0..7) {
        0 => ItemKind::HealthPotion,
        1 => ItemKind::Sword,
        2 => ItemKind::Shield,
        3 => ItemKind::Gold,
        4 => ItemKind::Key,
        5 => ItemKind::Scroll,
        _ => ItemKind::Chest,
    }
}

/// Check if player entered a new room and request AI description.
fn check_room_entry(
    player_query: Query<&Position, (With<Player>, Changed<Position>)>,
    monster_query: Query<(&Position, &Monster)>,
    item_query: Query<(&Position, &Item)>,
    map: Res<GameMap>,
    mut last_room: ResMut<LastRoom>,
    mut narrative_requests: MessageWriter<NarrativeRequest>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    // Check if player is in a room
    if let Some(room) = map.room_at(player_pos.x, player_pos.y) {
        // Only request description if this is a new room
        if last_room.room_id != Some(room.room_id) {
            last_room.room_id = Some(room.room_id);

            // Gather room contents
            let mut contents = Vec::new();

            for (pos, monster) in &monster_query {
                if room.contains(pos.x, pos.y) {
                    contents.push(RoomContentData::Monster(monster.kind));
                }
            }

            for (pos, item) in &item_query {
                if room.contains(pos.x, pos.y) {
                    contents.push(RoomContentData::Item(item.kind));
                }
            }

            // Request AI description
            narrative_requests.write(NarrativeRequest {
                id: room.room_id as u64,
                kind: NarrativeRequestKind::RoomDescription {
                    room: room.clone(),
                    contents,
                },
                target_entity: None,
            });
        }
    } else {
        // Player is in a corridor
        last_room.room_id = None;
    }
}

/// Handle AI narrative responses.
fn handle_narrative_responses(
    mut responses: MessageReader<NarrativeResponse>,
    mut narrative: ResMut<CurrentNarrative>,
    mut log: ResMut<GameLog>,
) {
    for response in responses.read() {
        match &response.kind {
            NarrativeResponseKind::RoomDescription(desc) => {
                narrative.title = "Room".to_string();
                narrative.text = desc.text.clone();
                narrative.atmosphere = Some(desc.atmosphere.clone());
                narrative.visible = true;

                // Also add to log
                log.add(&desc.text);
            }
            NarrativeResponseKind::MonsterDialogue(dialogue) => {
                log.add(format!("\"{}\"", dialogue.current_line));
            }
            NarrativeResponseKind::ActionResult(result) => {
                log.add(&result.description);
                if let Some(consequence) = &result.consequence {
                    log.add(consequence);
                }
            }
            NarrativeResponseKind::DeathNarration(death) => {
                narrative.title = "Death".to_string();
                narrative.text = death.epitaph.clone();
                narrative.atmosphere = Some(death.final_words.clone());
                narrative.visible = true;
            }
            NarrativeResponseKind::LoreDiscovery(lore) => {
                if let Some(inscription) = &lore.inscription {
                    log.add(format!("You read: \"{}\"", inscription));
                }
                log.add(&lore.history);
            }
            NarrativeResponseKind::Error(err) => {
                // Silently handle errors - game continues with fallback
                tracing::warn!("AI error: {}", err);
            }
        }
    }
}

/// Check if player has died.
fn check_player_death(
    player_query: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut log: ResMut<GameLog>,
) {
    if let Ok(health) = player_query.single() {
        if health.is_dead() {
            log.add("You have died. The dungeon claims another soul...");
            next_state.set(GameScreen::GameOver);
        }
    }
}
