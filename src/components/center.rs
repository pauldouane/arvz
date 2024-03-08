use std::time::Instant;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, tui::Frame};
use crate::config::Config;

#[derive(Default)]
pub struct Center {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Center {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Center {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let block = Block::default()
            .title("Center")
            .borders(Borders::ALL);
        f.render_widget(block, rect);
        Ok(())
    }
}