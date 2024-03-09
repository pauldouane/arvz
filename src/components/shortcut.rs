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
pub struct Shortcut {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
}

impl Shortcut {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for Shortcut {
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
        // Loop through keybindings by mode and display them
        let mut text = vec![];
        for (shortcut, action) in &*self.config.keybindings.get(&self.mode).unwrap() {
            text.push(
                Line::from(vec![
                    Span::styled(format!("<{}>", key_event_to_string(&shortcut[0])), Style::new().blue().bold()),
                    Span::styled(format!(" {}", action), Style::new()),
                ]),
            );
        }
        let block = Paragraph::new(text)
            .block(Block::new())
            .style(Style::new().white().on_black())
            .wrap(Wrap { trim: true });
        f.render_widget(block, area);
        Ok(())
    }
}