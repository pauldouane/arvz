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
pub struct TableDagRuns {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    columns: Vec<&'static str>,
}

impl TableDagRuns {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            columns: vec!["DAG ID", "STATE", "EXECUTION DATE", "START DATE", "END DATE", "DURATION", "EXTERNAL TRIGGER"],
        }
    }
}

impl Component for TableDagRuns {
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
        Ok(())
    }
}