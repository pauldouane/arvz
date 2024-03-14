use std::{collections::HashMap, time::Duration, vec};

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
use crate::mode::Mode;
use crate::utils::get_user_input_by_key;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct CommandSearch {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    user_search: Option<String>,
}

impl CommandSearch {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::DagRun,
            user_search: None,
        }
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

    fn handle_mode(&mut self, mode: Mode) -> Result<()> {
        self.mode = mode;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.mode == Mode::Search {
            get_user_input_by_key(key.code, &mut self.user_search);
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // draw the search bar
        let line = Line::from(vec![if let Some(search) = &self.user_search {
            Span::raw(search)
        } else {
            Span::raw("")
        }]);
        let search_bar = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(search_bar, area);
        Ok(())
    }
}
