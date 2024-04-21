use crate::components::table::table::LinkedTable;
use crate::components::Table;
use std::{collections::HashMap, time::Duration, vec};
use tokio::sync::MutexGuard;

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::config::key_event_to_string;
use crate::context_data::ContextData;
use crate::mode::Mode;
use crate::utils::get_user_input_by_key;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct Command {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    pub command: Option<String>,
}

impl Command {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::Command,
            command: None,
        }
    }
}

impl Component for Command {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_mode(&mut self, mode: Mode) -> Result<()> {
        self.mode = mode;
        Ok(())
    }

    fn handle_key_events(&mut self, key: &KeyEvent) -> Result<Option<Action>> {
        if self.mode == Mode::Command {
            get_user_input_by_key(key.code, &mut self.command);
        }
        Ok(None)
    }

    fn update(
        &mut self,
        action: Action,
        context_data: &MutexGuard<'_, ContextData>,
        tables: &MutexGuard<'_, LinkedTable>,
    ) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        context_data: &MutexGuard<'_, ContextData>,
        table: &MutexGuard<'_, dyn Table>,
        mode: Mode,
    ) -> Result<()> {
        // draw the search bar
        let line = Line::from(vec![if let Some(command) = &self.command {
            Span::raw(command)
        } else {
            Span::raw("")
        }]);
        let command_bar = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        );
        f.render_widget(command_bar, area);
        Ok(())
    }
}
