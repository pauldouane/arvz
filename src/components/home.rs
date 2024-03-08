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
pub struct Home {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  top: Top,
  center: Center,
  bottom: Bottom,
  str: String,
  count: u64,
}

impl Home {
  pub fn new() -> Self {
    // Set default values for all attributes without dags_runs_information
    Self {
      command_tx: None,
      config: Config::default(),
      top: Top::new(),
      center: Center::new(),
      bottom: Bottom::new(),
      str: "Hello, World!".to_string(),
      count: 0,
    }
  }
}

impl Component for Home {
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
    let main_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
          Constraint::Percentage(25),
          Constraint::Percentage(70),
          Constraint::Percentage(5),
        ])
        .split(area);
    self.top.draw(f, main_chunk[0])?;
    self.center.draw(f, main_chunk[1])?;
    self.bottom.draw(f, main_chunk[2])?;
    Ok(())
  }
}

