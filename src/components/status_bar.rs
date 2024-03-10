use std::{collections::HashMap, time::Duration, vec};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing_subscriber::fmt::format;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use crate::config::key_event_to_string;
use crate::mode::Mode;

#[derive(Default)]
pub struct StatusBar {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::DagRun,
        }
    }

    pub fn register_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for StatusBar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Render rectangle in left corner of the screen in orange with black text
        // Rectangle does not have a 100% of place in the left corner
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(9)])
            .split(area);
        let text = Text::from(format!("{:?}", self.mode));
        let block = Paragraph::new(text).alignment(Alignment::Center).block(Block::new().style(Style::default().bg(Color::Yellow)));
        f.render_widget(block,chunks[0]);
        Ok(())
    }
}