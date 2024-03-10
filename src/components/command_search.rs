use std::{collections::HashMap, time::Duration, vec};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use crate::config::key_event_to_string;
use crate::mode::Mode;

#[derive(Default)]
pub struct CommandSearch {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
}

impl CommandSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for CommandSearch {
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
        // draw the search bar
        let search_bar = Block::default()
            .title("Search")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(search_bar, area);
        Ok(())
    }
}