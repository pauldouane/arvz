use std::cell::RefCell;
use crate::tui::Event::Key;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration, vec};
use tokio::sync::MutexGuard;
use crate::tui::Event;
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

use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table::pool::Pool;
use crate::components::table::table::LinkedTable;
use crate::components::{Component, LinkedComponent};
use crate::config::key_event_to_string;
use crate::context_data::ContextData;
use crate::main_layout::Chunk;
use crate::mode::Mode;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct MainTable {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    tables: LinkedTable,
    state: TableState,
}

impl MainTable {
    pub fn new() -> Self {
        let mut linked_tables = LinkedTable::new();
        linked_tables.append(Rc::new(RefCell::new(Pool::new())));
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::Pool,
            tables: linked_tables,
            state: TableState::new(),
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

    fn handle_events(&mut self, event: Option<&Event>, context_data: &MutexGuard<'_, ContextData>, mode: Mode) -> Result<Option<Action>> {
        match event {
            Some(event) => {
                match event {
                    Key(key_event) => {
                        match key_event.code {
                            KeyCode::Down => {
                                if let Some(selected) = self.state.selected() {
                                    match &context_data.get_model_by_mode(mode).unwrap() {
                                        Some(data) => {
                                            if ((selected + 1) as i32) < data.get_total_entries() {
                                                self.state.select(Some(selected + 1))
                                            }
                                        }
                                        None => log::info!("No pool collection"),
                                    };
                                } else {
                                    self.state.select(Some(0))
                                }
                            },
                            KeyCode::Up => {
                                if let Some(selected) = self.state.selected() { 
                                    if selected > 0 { 
                                        self.state.select(Some(selected - 1)); 
                                    } 
                                } else { 
                                    self.state.select(Some(0)) 
                                }
                            },
                            KeyCode::Enter => {
                                self.tables.next();    
                            }
                            _ => {
                                log::info!("Key not implemented");
                            }
                        }
                        Ok(None)
                    },
                    _ => Ok(None)
                }
            },
            None => Ok(None)
        }
    }

    fn draw(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        context_data: &MutexGuard<'_, ContextData>,
    ) -> Result<()> {
        let current_table = self.tables.head.clone().unwrap();
        let node = current_table.borrow();
        let columns = node.value.borrow().get_columns();
        let total_entries = match &context_data.pool_collection {
            Some(pool_collection) => pool_collection.get_total_entries(),
            None => 0,
        };
        let rows = match &context_data.pool_collection {
            Some(pool_collection) => pool_collection.get_rows(),
            None => vec![],
        };
        let mut title = vec![
            Span::styled(format!(" {:?}", self.mode), Style::new().light_cyan()),
            Span::styled("[", Style::new().white()),
            Span::styled(total_entries.to_string(), Style::new().light_yellow()),
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
                    .border_style(node.value.borrow().get_border_style()),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }
}
