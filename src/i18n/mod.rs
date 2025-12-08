//! Internationalization (i18n) module using Mozilla's Fluent format.
//!
//! This module provides localization support for the game using `bevy_fluent`.
//! Translation strings are stored in `.ftl` files under `assets/locales/`.
//!
//! ## Supported Locales
//! - `en-US` - English (default)
//! - `ru-RU` - Russian
//!
//! ## Usage
//! ```ignore
//! // Access localized strings via the Localization resource
//! let text = localization.get("ui-dungeon-title");
//! ```

use bevy::prelude::*;
use unic_langid::LanguageIdentifier;

// Note: bevy_fluent is available for full .ftl file loading when assets are supported.
// Currently we use compiled-in translations for minimal dependency on asset loading.

/// Plugin for localization support.
pub fn plugin(app: &mut App) {
    app.init_resource::<LocaleSettings>()
        .init_resource::<Localization>()
        .add_systems(Update, update_localization);
}

/// Settings for the current locale.
#[derive(Resource)]
pub struct LocaleSettings {
    /// Current locale identifier (e.g., "en-US", "ru-RU").
    pub current_locale: LanguageIdentifier,
    /// Available locales.
    pub available_locales: Vec<LanguageIdentifier>,
}

impl Default for LocaleSettings {
    fn default() -> Self {
        Self {
            current_locale: "en-US".parse().expect("valid locale"),
            available_locales: vec![
                "en-US".parse().expect("valid locale"),
                "ru-RU".parse().expect("valid locale"),
            ],
        }
    }
}

impl LocaleSettings {
    /// Set the current locale.
    pub fn set_locale(&mut self, locale: &str) -> Result<(), LocaleError> {
        let lang_id: LanguageIdentifier = locale
            .parse()
            .map_err(|_| LocaleError::InvalidLocale(locale.to_string()))?;

        if self.available_locales.contains(&lang_id) {
            self.current_locale = lang_id;
            Ok(())
        } else {
            Err(LocaleError::UnsupportedLocale(locale.to_string()))
        }
    }

    /// Get locale display name.
    pub fn locale_display_name(locale: &LanguageIdentifier) -> &'static str {
        match locale.to_string().as_str() {
            "en-US" => "English",
            "ru-RU" => "Русский",
            _ => "Unknown",
        }
    }
}

/// Errors related to localization.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LocaleError {
    #[error("Invalid locale format: {0}")]
    InvalidLocale(String),
    #[error("Unsupported locale: {0}")]
    UnsupportedLocale(String),
}

/// Resource for accessing localized strings.
///
/// This is a simple wrapper that provides fallback to English
/// and placeholder handling when translations are missing.
#[derive(Resource, Default)]
pub struct Localization {
    /// Cached translations for current locale.
    translations: std::collections::HashMap<String, String>,
    /// Whether translations have been loaded.
    loaded: bool,
}

impl Localization {
    /// Get a localized string by key.
    ///
    /// Returns the key itself if translation is not found (useful for debugging).
    pub fn get(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| self.fallback(key))
    }

    /// Get a localized string with variable substitution.
    ///
    /// Variables in the format string use `{$var}` syntax from Fluent.
    pub fn get_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.get(key);
        for (var, value) in args {
            let placeholder = format!("{{${}}}", var);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Fallback for missing translations (returns built-in English).
    fn fallback(&self, key: &str) -> String {
        // Built-in English fallbacks for critical UI strings
        match key {
            // Widget titles
            "ui-dungeon-title" => " Dungeon ".to_string(),
            "ui-messages-title" => " Messages ".to_string(),
            "ui-status-title" => " Status ".to_string(),
            "ui-controls-title" => " Controls ".to_string(),
            "ui-look-title" => " Look ".to_string(),
            "ui-inventory-title" => " Inventory ".to_string(),

            // Status display
            "ui-hp-label" => "HP".to_string(),
            "ui-floor-label" => "Floor".to_string(),
            "ui-gold-label" => "Gold".to_string(),

            // Look mode
            "ui-look-mode-header" => "-- LOOK MODE --".to_string(),
            "ui-position-label" => "Position".to_string(),
            "ui-tile-label" => "Tile".to_string(),
            "ui-not-in-view" => "Not in view".to_string(),
            "ui-unexplored" => "Unexplored".to_string(),

            // Tile names
            "tile-wall" => "Wall".to_string(),
            "tile-floor" => "Floor".to_string(),
            "tile-door" => "Door".to_string(),
            "tile-stairs-down" => "Stairs Down".to_string(),
            "tile-stairs-up" => "Stairs Up".to_string(),

            // Controls
            "ctrl-move" => "hjkl/arrows: Move".to_string(),
            "ctrl-move-cursor" => "hjkl/arrows: Move cursor".to_string(),
            "ctrl-wait" => ".: Wait".to_string(),
            "ctrl-look" => "x: Look mode".to_string(),
            "ctrl-select" => "Enter/Space: Select".to_string(),
            "ctrl-exit-look" => "x/Esc: Exit look mode".to_string(),
            "ctrl-inventory" => "i: Inventory".to_string(),
            "ctrl-quit" => "q: Quit".to_string(),

            // Inventory
            "inv-empty" => "Your inventory is empty.".to_string(),
            "inv-close-hint" => "Press 'i' or Esc to close.".to_string(),

            // Combat messages
            "combat-bump-wall" => "You bump into a wall.".to_string(),
            "combat-fumble" => "{$attacker} swings wildly at {$defender} but completely misses!".to_string(),
            "combat-critical" => "{$attacker} lands a CRITICAL HIT on {$defender}! {$damage} {$type} damage!".to_string(),
            "combat-hit" => "{$attacker} hits {$defender} for {$damage} {$type} damage.".to_string(),
            "combat-miss" => "{$attacker} attacks {$defender} but misses. ({$roll} vs AC {$ac})".to_string(),
            "combat-slain" => "{$target} is slain!".to_string(),

            // Entity names
            "entity-you" => "You".to_string(),
            "entity-you-lowercase" => "you".to_string(),
            "entity-something" => "Something".to_string(),
            "entity-something-lowercase" => "something".to_string(),

            _ => format!("[{}]", key), // Show key for unknown strings
        }
    }

    /// Load translations from built-in defaults.
    /// In a full implementation, this would load from .ftl files via bevy_fluent.
    fn load_defaults(&mut self, locale: &LanguageIdentifier) {
        self.translations.clear();

        match locale.to_string().as_str() {
            "ru-RU" => self.load_russian(),
            _ => self.load_english(),
        }

        self.loaded = true;
    }

    /// Load English translations (default).
    fn load_english(&mut self) {
        // Widget titles
        self.translations.insert("ui-dungeon-title".into(), " Dungeon ".into());
        self.translations.insert("ui-messages-title".into(), " Messages ".into());
        self.translations.insert("ui-status-title".into(), " Status ".into());
        self.translations.insert("ui-controls-title".into(), " Controls ".into());
        self.translations.insert("ui-look-title".into(), " Look ".into());
        self.translations.insert("ui-inventory-title".into(), " Inventory ".into());

        // Status display
        self.translations.insert("ui-hp-label".into(), "HP".into());
        self.translations.insert("ui-floor-label".into(), "Floor".into());
        self.translations.insert("ui-gold-label".into(), "Gold".into());

        // Look mode
        self.translations.insert("ui-look-mode-header".into(), "-- LOOK MODE --".into());
        self.translations.insert("ui-position-label".into(), "Position".into());
        self.translations.insert("ui-tile-label".into(), "Tile".into());
        self.translations.insert("ui-not-in-view".into(), "Not in view".into());
        self.translations.insert("ui-unexplored".into(), "Unexplored".into());

        // Tile names
        self.translations.insert("tile-wall".into(), "Wall".into());
        self.translations.insert("tile-floor".into(), "Floor".into());
        self.translations.insert("tile-door".into(), "Door".into());
        self.translations.insert("tile-stairs-down".into(), "Stairs Down".into());
        self.translations.insert("tile-stairs-up".into(), "Stairs Up".into());

        // Controls
        self.translations.insert("ctrl-move".into(), "hjkl/arrows: Move".into());
        self.translations.insert("ctrl-move-cursor".into(), "hjkl/arrows: Move cursor".into());
        self.translations.insert("ctrl-wait".into(), ".: Wait".into());
        self.translations.insert("ctrl-look".into(), "x: Look mode".into());
        self.translations.insert("ctrl-select".into(), "Enter/Space: Select".into());
        self.translations.insert("ctrl-exit-look".into(), "x/Esc: Exit look mode".into());
        self.translations.insert("ctrl-inventory".into(), "i: Inventory".into());
        self.translations.insert("ctrl-quit".into(), "q: Quit".into());

        // Inventory
        self.translations.insert("inv-empty".into(), "Your inventory is empty.".into());
        self.translations.insert("inv-close-hint".into(), "Press 'i' or Esc to close.".into());

        // Combat messages
        self.translations.insert("combat-bump-wall".into(), "You bump into a wall.".into());
        self.translations.insert("combat-fumble".into(), "{$attacker} swings wildly at {$defender} but completely misses!".into());
        self.translations.insert("combat-critical".into(), "{$attacker} lands a CRITICAL HIT on {$defender}! {$damage} {$type} damage!".into());
        self.translations.insert("combat-hit".into(), "{$attacker} hits {$defender} for {$damage} {$type} damage.".into());
        self.translations.insert("combat-miss".into(), "{$attacker} attacks {$defender} but misses. ({$roll} vs AC {$ac})".into());
        self.translations.insert("combat-slain".into(), "{$target} is slain!".into());

        // Entity names
        self.translations.insert("entity-you".into(), "You".into());
        self.translations.insert("entity-you-lowercase".into(), "you".into());
        self.translations.insert("entity-something".into(), "Something".into());
        self.translations.insert("entity-something-lowercase".into(), "something".into());
    }

    /// Load Russian translations.
    fn load_russian(&mut self) {
        // Widget titles
        self.translations.insert("ui-dungeon-title".into(), " Подземелье ".into());
        self.translations.insert("ui-messages-title".into(), " Сообщения ".into());
        self.translations.insert("ui-status-title".into(), " Статус ".into());
        self.translations.insert("ui-controls-title".into(), " Управление ".into());
        self.translations.insert("ui-look-title".into(), " Осмотр ".into());
        self.translations.insert("ui-inventory-title".into(), " Инвентарь ".into());

        // Status display
        self.translations.insert("ui-hp-label".into(), "ОЗ".into());
        self.translations.insert("ui-floor-label".into(), "Этаж".into());
        self.translations.insert("ui-gold-label".into(), "Золото".into());

        // Look mode
        self.translations.insert("ui-look-mode-header".into(), "-- РЕЖИМ ОСМОТРА --".into());
        self.translations.insert("ui-position-label".into(), "Позиция".into());
        self.translations.insert("ui-tile-label".into(), "Клетка".into());
        self.translations.insert("ui-not-in-view".into(), "Не видно".into());
        self.translations.insert("ui-unexplored".into(), "Неизведано".into());

        // Tile names
        self.translations.insert("tile-wall".into(), "Стена".into());
        self.translations.insert("tile-floor".into(), "Пол".into());
        self.translations.insert("tile-door".into(), "Дверь".into());
        self.translations.insert("tile-stairs-down".into(), "Лестница вниз".into());
        self.translations.insert("tile-stairs-up".into(), "Лестница вверх".into());

        // Controls
        self.translations.insert("ctrl-move".into(), "hjkl/стрелки: Движение".into());
        self.translations.insert("ctrl-move-cursor".into(), "hjkl/стрелки: Курсор".into());
        self.translations.insert("ctrl-wait".into(), ".: Ждать".into());
        self.translations.insert("ctrl-look".into(), "x: Осмотр".into());
        self.translations.insert("ctrl-select".into(), "Enter/Пробел: Выбрать".into());
        self.translations.insert("ctrl-exit-look".into(), "x/Esc: Выйти из осмотра".into());
        self.translations.insert("ctrl-inventory".into(), "i: Инвентарь".into());
        self.translations.insert("ctrl-quit".into(), "q: Выход".into());

        // Inventory
        self.translations.insert("inv-empty".into(), "Ваш инвентарь пуст.".into());
        self.translations.insert("inv-close-hint".into(), "Нажмите 'i' или Esc чтобы закрыть.".into());

        // Combat messages
        self.translations.insert("combat-bump-wall".into(), "Вы врезались в стену.".into());
        self.translations.insert("combat-fumble".into(), "{$attacker} промахивается по {$defender}!".into());
        self.translations.insert("combat-critical".into(), "{$attacker} наносит КРИТИЧЕСКИЙ УДАР по {$defender}! {$damage} ед. урона ({$type})!".into());
        self.translations.insert("combat-hit".into(), "{$attacker} попадает по {$defender}: {$damage} ед. урона ({$type}).".into());
        self.translations.insert("combat-miss".into(), "{$attacker} атакует {$defender}, но промахивается. ({$roll} против КЗ {$ac})".into());
        self.translations.insert("combat-slain".into(), "{$target} повержен!".into());

        // Entity names
        self.translations.insert("entity-you".into(), "Вы".into());
        self.translations.insert("entity-you-lowercase".into(), "вас".into());
        self.translations.insert("entity-something".into(), "Нечто".into());
        self.translations.insert("entity-something-lowercase".into(), "нечто".into());
    }
}

/// System to update localization when locale changes.
fn update_localization(
    settings: Res<LocaleSettings>,
    mut localization: ResMut<Localization>,
) {
    // Only update if settings changed or not yet loaded
    if settings.is_changed() || !localization.loaded {
        localization.load_defaults(&settings.current_locale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_settings_default() {
        let settings = LocaleSettings::default();
        assert_eq!(settings.current_locale.to_string(), "en-US");
        assert_eq!(settings.available_locales.len(), 2);
    }

    #[test]
    fn test_set_valid_locale() {
        let mut settings = LocaleSettings::default();
        assert!(settings.set_locale("ru-RU").is_ok());
        assert_eq!(settings.current_locale.to_string(), "ru-RU");
    }

    #[test]
    fn test_set_invalid_locale() {
        let mut settings = LocaleSettings::default();
        assert!(settings.set_locale("invalid").is_err());
    }

    #[test]
    fn test_set_unsupported_locale() {
        let mut settings = LocaleSettings::default();
        assert!(settings.set_locale("de-DE").is_err());
    }

    #[test]
    fn test_localization_fallback() {
        let loc = Localization::default();
        // Unknown key returns bracketed key
        assert_eq!(loc.get("unknown-key"), "[unknown-key]");
        // Known fallback key returns English
        assert_eq!(loc.get("ui-dungeon-title"), " Dungeon ");
    }

    #[test]
    fn test_localization_with_args() {
        let mut loc = Localization::default();
        loc.load_defaults(&"en-US".parse().unwrap());

        let result = loc.get_with_args("combat-hit", &[
            ("attacker", "You"),
            ("defender", "the goblin"),
            ("damage", "5"),
            ("type", "slashing"),
        ]);

        assert!(result.contains("You"));
        assert!(result.contains("the goblin"));
        assert!(result.contains("5"));
        assert!(result.contains("slashing"));
    }

    #[test]
    fn test_load_english() {
        let mut loc = Localization::default();
        loc.load_defaults(&"en-US".parse().unwrap());

        assert_eq!(loc.get("ui-dungeon-title"), " Dungeon ");
        assert_eq!(loc.get("tile-wall"), "Wall");
    }

    #[test]
    fn test_load_russian() {
        let mut loc = Localization::default();
        loc.load_defaults(&"ru-RU".parse().unwrap());

        assert_eq!(loc.get("ui-dungeon-title"), " Подземелье ");
        assert_eq!(loc.get("tile-wall"), "Стена");
    }
}
