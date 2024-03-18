use std::{collections::HashMap, time::Duration, vec};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use log::Level::Trace;
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::config::key_event_to_string;
use crate::mode::Mode;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct Shortcut {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    area: usize,
    mode: Mode,
}

impl Shortcut {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::new().unwrap(),
            area: 1,
            mode: Mode::DagRun,
        }
    }

    pub fn register_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for Shortcut {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        {}
        Ok(None)
    }

    fn get_area(&self) -> usize {
        1
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Loop through keybindings by mode and display them
        // Get the number of keybindings for the current mode
        let num_keybindings = self.config.keybindings.get(&self.mode).unwrap().len() as f64;

        let number_of_columns = (num_keybindings / 6f64).ceil() as u16;

        let mut constraints = vec![];

        for _ in 0..number_of_columns {
            constraints.push(Constraint::Length(21));
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let mut text: Vec<Line> = vec![];
        let mut cnt_keybindings = 1;
        let max_shortcut_len = self
            .config
            .keybindings
            .get(&self.mode)
            .unwrap()
            .iter()
            .map(|(shortcut, _)| key_event_to_string(&shortcut[0]).len())
            .max()
            .unwrap();
        for (shortcut, action) in self.config.keybindings.get(&self.mode).unwrap() {
            text.push(Line::from(vec![
                Span::styled(
                    format!("<{}>", key_event_to_string(&shortcut[0])),
                    Style::new().blue().bold(),
                ),
                Span::raw(" ".repeat(max_shortcut_len - key_event_to_string(&shortcut[0]).len())),
                Span::styled(format!(" {}", action), Style::new()),
            ]));
            if cnt_keybindings % 6 == 0 || cnt_keybindings as f64 == num_keybindings {
                f.render_widget(
                    Paragraph::new(text),
                    layout[(cnt_keybindings as f64 / 6f64).ceil() as usize - 1],
                );
                text = vec![];
            }
            cnt_keybindings += 1;
        }
        Ok(())
    }
}
