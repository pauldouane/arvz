use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct Ascii {
    command_tx: Option<UnboundedSender<Action>>,
    ascii: &'static str,
    config: Config,
}

impl Ascii {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            // Store ascii art to display
            ascii: r"
  __ _ _ ____   ______
 / _` | '__\ \ / /_  /
| (_| | |   \ V / / /
 \__,_|_|    \_/ /___|
            ",
            config: Config::default(),
        }
    }
}

impl Component for Ascii {
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
        let block = Paragraph::new(self.ascii)
            .block(Block::new());
        f.render_widget(block, area);
        Ok(())
    }
}