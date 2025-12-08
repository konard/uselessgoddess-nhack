//! Custom TUI widgets for the game.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::game::{GameLog, GameMap, Health, Position, TileType};

/// Widget for rendering the dungeon map.
pub struct MapWidget<'a> {
    map: &'a GameMap,
    player_pos: Option<Position>,
    cursor_pos: Option<Position>,
    entities: Vec<(Position, char, Color)>,
}

impl<'a> MapWidget<'a> {
    pub fn new(map: &'a GameMap) -> Self {
        Self {
            map,
            player_pos: None,
            cursor_pos: None,
            entities: Vec::new(),
        }
    }

    pub fn with_player(mut self, pos: Position) -> Self {
        self.player_pos = Some(pos);
        self
    }

    /// Add a cursor position for targeting/look mode.
    pub fn with_cursor(mut self, pos: Position) -> Self {
        self.cursor_pos = Some(pos);
        self
    }

    #[allow(dead_code)]
    pub fn with_entity(mut self, pos: Position, glyph: char, color: Color) -> Self {
        self.entities.push((pos, glyph, color));
        self
    }

    pub fn with_entities(mut self, entities: Vec<(Position, char, Color)>) -> Self {
        self.entities = entities;
        self
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Dungeon ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        // Calculate viewport offset to center on player
        let (offset_x, offset_y) = if let Some(player_pos) = self.player_pos {
            let center_x = inner.width as i32 / 2;
            let center_y = inner.height as i32 / 2;
            (player_pos.x - center_x, player_pos.y - center_y)
        } else {
            (0, 0)
        };

        // Render tiles
        for screen_y in 0..inner.height {
            for screen_x in 0..inner.width {
                let map_x = screen_x as i32 + offset_x;
                let map_y = screen_y as i32 + offset_y;

                if !self.map.in_bounds(map_x, map_y) {
                    continue;
                }

                let idx = self.map.xy_idx(map_x, map_y);
                let visible = self.map.visible.get(idx).copied().unwrap_or(false);
                let revealed = self.map.revealed.get(idx).copied().unwrap_or(false);

                if !revealed {
                    continue;
                }

                let tile = self.map.tiles[idx];
                let (glyph, fg) = tile_appearance(tile, visible);

                buf[(inner.x + screen_x, inner.y + screen_y)]
                    .set_char(glyph)
                    .set_fg(fg);
            }
        }

        // Render entities
        for (pos, glyph, color) in &self.entities {
            let screen_x = (pos.x - offset_x) as u16;
            let screen_y = (pos.y - offset_y) as u16;

            if screen_x < inner.width && screen_y < inner.height {
                let idx = self.map.xy_idx(pos.x, pos.y);
                if self.map.visible.get(idx).copied().unwrap_or(false) {
                    buf[(inner.x + screen_x, inner.y + screen_y)]
                        .set_char(*glyph)
                        .set_fg(*color);
                }
            }
        }

        // Render player (always on top of entities)
        if let Some(player_pos) = self.player_pos {
            let screen_x = (player_pos.x - offset_x) as u16;
            let screen_y = (player_pos.y - offset_y) as u16;

            if screen_x < inner.width && screen_y < inner.height {
                buf[(inner.x + screen_x, inner.y + screen_y)]
                    .set_char('@')
                    .set_fg(Color::Yellow)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }

        // Render cursor (on top of everything, highlighted)
        if let Some(cursor_pos) = self.cursor_pos {
            let screen_x = (cursor_pos.x - offset_x) as u16;
            let screen_y = (cursor_pos.y - offset_y) as u16;

            if screen_x < inner.width && screen_y < inner.height {
                // Highlight the cursor position with a distinctive background
                let cell = &mut buf[(inner.x + screen_x, inner.y + screen_y)];
                cell.set_bg(Color::DarkGray)
                    .set_style(Style::default().add_modifier(Modifier::REVERSED));
            }
        }
    }
}

fn tile_appearance(tile: TileType, visible: bool) -> (char, Color) {
    let glyph = tile.glyph();
    let fg = match tile {
        TileType::Wall => {
            if visible {
                Color::White
            } else {
                Color::DarkGray
            }
        }
        TileType::Floor => {
            if visible {
                Color::Gray
            } else {
                Color::DarkGray
            }
        }
        TileType::Door => {
            if visible {
                Color::Yellow
            } else {
                Color::DarkGray
            }
        }
        TileType::StairsDown | TileType::StairsUp => {
            if visible {
                Color::Cyan
            } else {
                Color::DarkGray
            }
        }
    };
    (glyph, fg)
}

/// Widget for displaying the game log.
pub struct LogWidget<'a> {
    log: &'a GameLog,
    max_lines: usize,
}

impl<'a> LogWidget<'a> {
    pub fn new(log: &'a GameLog) -> Self {
        Self { log, max_lines: 5 }
    }

    pub fn with_max_lines(mut self, lines: usize) -> Self {
        self.max_lines = lines;
        self
    }
}

impl Widget for LogWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Messages ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        let messages: Vec<Line> = self
            .log
            .recent(self.max_lines)
            .map(|msg| Line::from(Span::raw(msg.as_str())))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        Paragraph::new(messages)
            .style(Style::default().fg(Color::White))
            .render(inner, buf);
    }
}

/// Widget for displaying player stats.
pub struct StatsWidget {
    health: Health,
    floor: i32,
    gold: i32,
}

impl StatsWidget {
    pub fn new(health: Health, floor: i32, gold: i32) -> Self {
        Self {
            health,
            floor,
            gold,
        }
    }
}

impl Widget for StatsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        // Health bar
        let health_pct = self.health.percentage();
        let health_color = if health_pct > 0.6 {
            Color::Green
        } else if health_pct > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        let health_text = format!("HP: {}/{}", self.health.current, self.health.max);
        let floor_text = format!("Floor: {}", self.floor);
        let gold_text = format!("Gold: {}", self.gold);

        let lines = vec![
            Line::from(Span::styled(health_text, Style::default().fg(health_color))),
            Line::from(Span::styled(floor_text, Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(gold_text, Style::default().fg(Color::Yellow))),
        ];

        Paragraph::new(lines).render(inner, buf);
    }
}

/// Widget for displaying AI-generated narrative.
pub struct NarrativeWidget<'a> {
    title: &'a str,
    text: &'a str,
    atmosphere: Option<&'a str>,
}

impl<'a> NarrativeWidget<'a> {
    pub fn new(title: &'a str, text: &'a str) -> Self {
        Self {
            title,
            text,
            atmosphere: None,
        }
    }

    pub fn with_atmosphere(mut self, atmosphere: &'a str) -> Self {
        self.atmosphere = Some(atmosphere);
        self
    }
}

impl Widget for NarrativeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = vec![Line::from(Span::styled(
            self.text,
            Style::default().fg(Color::White),
        ))];

        if let Some(atmo) = self.atmosphere {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("~ {} ~", atmo),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        Paragraph::new(lines).render(inner, buf);
    }
}

/// Widget for the help/controls display.
pub struct HelpWidget {
    targeting_mode: bool,
}

impl HelpWidget {
    pub fn new(targeting_mode: bool) -> Self {
        Self { targeting_mode }
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        let controls = if self.targeting_mode {
            vec![
                Line::from(Span::styled(
                    "-- LOOK MODE --",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from("hjkl/arrows: Move cursor"),
                Line::from("Enter/Space: Select"),
                Line::from("x/Esc: Exit look mode"),
            ]
        } else {
            vec![
                Line::from("hjkl/arrows: Move"),
                Line::from(".: Wait"),
                Line::from("x: Look mode"),
                Line::from("i: Inventory"),
                Line::from("q: Quit"),
            ]
        };

        Paragraph::new(controls)
            .style(Style::default().fg(Color::Gray))
            .render(inner, buf);
    }
}

/// Widget for displaying cursor/target information.
pub struct CursorWidget<'a> {
    position: &'a Position,
    map: &'a GameMap,
}

impl<'a> CursorWidget<'a> {
    pub fn new(position: &'a Position, map: &'a GameMap) -> Self {
        Self { position, map }
    }
}

impl Widget for CursorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Look ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = vec![
            Line::from(Span::styled(
                format!("Position: ({}, {})", self.position.x, self.position.y),
                Style::default().fg(Color::White),
            )),
        ];

        // Show tile information if visible
        if self.map.in_bounds(self.position.x, self.position.y) {
            let idx = self.map.xy_idx(self.position.x, self.position.y);
            let visible = self.map.visible.get(idx).copied().unwrap_or(false);
            let revealed = self.map.revealed.get(idx).copied().unwrap_or(false);

            if visible {
                let tile = self.map.tiles[idx];
                let tile_name = match tile {
                    TileType::Wall => "Wall",
                    TileType::Floor => "Floor",
                    TileType::Door => "Door",
                    TileType::StairsDown => "Stairs Down",
                    TileType::StairsUp => "Stairs Up",
                };
                lines.push(Line::from(Span::styled(
                    format!("Tile: {}", tile_name),
                    Style::default().fg(Color::Gray),
                )));
            } else if revealed {
                lines.push(Line::from(Span::styled(
                    "Not in view",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Unexplored",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        // TODO: Add entity information at cursor position

        Paragraph::new(lines).render(inner, buf);
    }
}

/// Widget for displaying the inventory screen.
pub struct InventoryWidget {
    // TODO: Add inventory items
}

impl InventoryWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InventoryWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for InventoryWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Inventory ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        // Placeholder content for inventory
        let lines = vec![
            Line::from(Span::styled(
                "Your inventory is empty.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'i' or Esc to close.",
                Style::default().fg(Color::Gray),
            )),
        ];

        Paragraph::new(lines).render(inner, buf);
    }
}
