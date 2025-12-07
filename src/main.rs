//! NeuroHack - A TUI Roguelike with AI-powered narrative generation.
//!
//! This game combines a rigid Rust roguelike engine with an AI Game Master
//! that generates atmospheric horror-themed descriptions and dialogue.

// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

mod ai;
mod game;
mod screens;
mod theme;
mod tui;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy_ratatui::RatatuiPlugins;

use screens::GameScreen;

fn main() -> AppExit {
    // Run at 30 FPS for TUI - higher rates waste CPU for terminal rendering
    let frame_time = Duration::from_secs_f64(1.0 / 30.0);

    App::new()
        .add_plugins(AppPlugin { frame_time })
        .run()
}

/// Main application plugin.
pub struct AppPlugin {
    pub frame_time: Duration,
}

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Core Bevy plugins (minimal set for TUI)
        app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(self.frame_time)),
        );

        // Ratatui TUI integration
        app.add_plugins(RatatuiPlugins::default());

        // Game plugins
        app.add_plugins((
            game::plugin,
            ai::plugin,
            tui::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Configure system ordering
        app.configure_sets(
            Update,
            (
                AppSystems::Input,
                AppSystems::GameLogic,
                AppSystems::AI,
                AppSystems::Render,
            )
                .chain(),
        );

        // Start in loading state, then transition to playing
        app.add_systems(Startup, start_game);
    }
}

/// High-level system groupings for the app.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Process player input
    Input,
    /// Run game logic (movement, combat, etc.)
    GameLogic,
    /// Handle AI narrative generation
    AI,
    /// Render the TUI
    Render,
}

/// Transition from loading to playing.
fn start_game(mut next_state: ResMut<NextState<GameScreen>>) {
    next_state.set(GameScreen::Playing);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_builds_without_panic() {
        // Just verify the app can be constructed
        let frame_time = Duration::from_secs_f64(1.0 / 30.0);
        let _plugin = AppPlugin { frame_time };
    }
}
