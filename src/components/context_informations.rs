use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::io::split;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct ContextInformation {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl ContextInformation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for ContextInformation {
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
        let text = vec![
            Line::from(vec![
                Span::styled("Dag Runs Number : ", Style::new().yellow()),
                Span::raw("n/a"),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Running : ", Style::new().yellow()),
                Span::raw("n/a"),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Failed : ", Style::new().yellow()),
                Span::raw("n/a"),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Scheduled : ", Style::new().yellow()),
                Span::raw("n/a"),
            ]),
        ];
        let block = Paragraph::new(text)
            .block(Block::new())
            .style(Style::new().white().on_black())
            .wrap(Wrap { trim: true });
        f.render_widget(block, area);
        Ok(())
    }
}