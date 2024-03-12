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
    pub(crate) mode_breadcrumb: Vec<Mode>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::DagRun,
            mode_breadcrumb: vec![],
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
        // Create a new block with self.mode_breadcrumb.len() + 1 columns
        // Space between each column is 10
        self.mode_breadcrumb.push(self.mode);
        let constraints = self.mode_breadcrumb.iter().map(|_| Constraint::Length(10)).collect::<Vec<_>>();

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                constraints
            )
            .split(area);

        if self.mode_breadcrumb.len() > 0 {
            for (i, mode) in self.mode_breadcrumb.iter().enumerate() {
                let para = Paragraph::new(format!("<{:?}>", mode))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray).bg(Color::Yellow));
                f.render_widget(para, block[i]);
            }
        }

        let para = Paragraph::new(format!("<{:?}>", self.mode))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray).bg(Color::LightCyan));
        f.render_widget(para, block[self.mode_breadcrumb.len() - 1]);
        self.mode_breadcrumb.pop();
        Ok(())
    }
}