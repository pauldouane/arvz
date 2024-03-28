use std::{collections::HashMap, time::Duration, vec};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use libc::bind;
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing_subscriber::fmt::format;

use crate::config::key_event_to_string;
use crate::mode::Mode;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use crate::components::{Component, LinkedComponent};
use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table::pool::Pool;
use crate::components::table::table::LinkedTable;
use crate::main_layout::Chunk;

#[derive(Default)]
pub struct MainTable {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    tables: LinkedTable,
}

impl MainTable {
    pub fn new() -> Self {
        let mut linked_tables = LinkedTable::new();
        linked_tables.append(
            Rc::new(RefCell::new(Pool::new())),
        );
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::Pool,
            tables: linked_tables,
        }
    }

    pub fn register_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for MainTable {
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

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let current_table = self.tables.head.clone().unwrap();
        let node = current_table.borrow();
        let columns = node.value.borrow().get_columns();
        let rows: Vec<Row> = vec![];
        let mut title = vec![
            Span::styled(format!(" {:?}", self.mode), Style::new().light_cyan()),
            Span::styled("[", Style::new().white()),
            Span::styled(
                "0".to_string(),
                Style::new().light_yellow(),
            ),
            Span::styled("] ", Style::new().white()),
        ];
        let widths = columns
            .iter()
            .map(|_| Constraint::Percentage(100 / columns.len() as u16))
            .collect::<Vec<_>>();

        let table = Table::new(rows, widths)
            .header(Row::new(columns.to_vec()).bottom_margin(0))
            .block(
                Block::default()
                    .title(Line::from(title))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        f.render_widget(table, area);
        Ok(())
    }
}
