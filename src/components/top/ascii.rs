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
use crate::components::bottom::Bottom;
use crate::components::center::Center;
use crate::components::top::Top;

#[derive(Default)]
pub struct Ascii {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Ascii {
    pub fn new() -> Self {
        Self::default()
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
        let block = Block::default()
            .title("Ascii")
            .borders(Borders::ALL);
        f.render_widget(block, area);
        Ok(())
    }
}