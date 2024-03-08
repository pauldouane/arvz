mod context_informations;
mod ascii;
mod shortcut;

use std::time::Instant;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, tui::Frame};
use crate::components::top::ascii::Ascii;
use crate::components::top::context_informations::ContextInformation;
use crate::components::top::shortcut::Shortcut;
use crate::config::Config;

#[derive(Default)]
pub struct Top {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    context_information: ContextInformation,
    shortcut: Shortcut,
    ascii: Ascii,
}

impl Top {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            context_information: ContextInformation::new(),
            shortcut: Shortcut::new(),
            ascii: Ascii::new(),
        }
    }
}

impl Component for Top {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let main_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area);
        self.context_information.draw(f, main_chunk[0])?;
        self.shortcut.draw(f, main_chunk[1])?;
        self.ascii.draw(f, main_chunk[2])?;
        Ok(())
    }
}