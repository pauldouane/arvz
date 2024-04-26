use crate::components::Table;
use crate::components::Tables;
use crate::mode::ObservableMode;
use crate::tui::Event;
use crate::tui::Event::Key;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use libc::bind;
use ratatui::symbols::border;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::Table as TableRatatui;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration, vec};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::MutexGuard;
use tracing_subscriber::fmt::format;

use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table::dag_run::DagRun;
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
    state: TableState,
    previous_mode: Option<Mode>,
    last_container: Option<String>,
    container: String,
    previous_table: bool,
    history: Vec<Mode>,
}

impl MainTable {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: TableState::new(),
            previous_mode: None,
            last_container: None,
            container: String::from("all"),
            previous_table: false,
            history: vec![Mode::Pool],
        }
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

    fn handle_events(
        &mut self,
        event: Option<&Event>,
        context_data: &mut MutexGuard<'_, ContextData>,
        mode: &ObservableMode,
        tables: &mut Tables,
    ) -> Result<Option<Action>> {
        match event {
            Some(event) => match event {
                Key(key_event) => {
                    match key_event.code {
                        KeyCode::Down => {
                            if let Some(selected) = self.state.selected() {
                                match &context_data.get_model_by_mode(mode.get()).unwrap() {
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
                        }
                        KeyCode::Up => {
                            if let Some(selected) = self.state.selected() {
                                if selected > 0 {
                                    self.state.select(Some(selected - 1));
                                }
                            } else {
                                self.state.select(Some(0))
                            }
                        }
                        KeyCode::Enter => {
                            self.command_tx
                                .as_ref()
                                .unwrap()
                                .send(Action::NextMode(mode.get()));
                            self.previous_mode = Some(mode.get());
                            self.history.push(mode.get_next_mode(mode.get()));
                            if let Some(binding) =
                                context_data.get_model_by_mode(mode.get()).unwrap()
                            {
                                if let Some(element) =
                                    binding.get_element(self.state.selected().expect("No selected"))
                                {
                                    self.container = element.get_id().to_string();
                                    context_data.params = Some(element.get_id().to_string());
                                }
                            }
                            self.state.select(None);
                        }
                        KeyCode::Esc => {
                            if mode.get() != self.history[0] {
                                if let Some(previous) = self.previous_mode {
                                    self.history.pop();
                                    self.command_tx
                                        .as_ref()
                                        .unwrap()
                                        .send(Action::PreviousMode(previous));
                                    self.state.select(None);
                                }
                            }
                        }
                        _ => {
                            log::info!("Key not implemented");
                        }
                    }
                    Ok(None)
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn draw(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        context_data: &MutexGuard<'_, ContextData>,
        table: &MutexGuard<'_, dyn Table>,
        mode: Mode,
    ) -> Result<()> {
        let columns = &table.get_columns();
        let binding = context_data
            .get_model_by_mode(mode)
            .unwrap()
            .as_ref()
            .expect("");
        let container = if self.history.len() == 1 {
            String::from("all")
        } else {
            self.container.clone()
        };
        let mut title = vec![
            Span::styled(format!(" {:?}", mode), Style::new().light_cyan()),
            Span::styled(format!("("), Style::default().fg(Color::LightCyan)),
            Span::styled(
                format!("{}", container),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(format!(")"), Style::default().fg(Color::LightCyan)),
            Span::styled("[", Style::new().white()),
            Span::styled(
                binding.get_total_entries().to_string(),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled("] ", Style::new().white()),
        ];
        let rows: Vec<Row> = binding.get_rows();
        let widths = columns
            .iter()
            .map(|_| Constraint::Percentage(100 / columns.len() as u16))
            .collect::<Vec<_>>();
        let table = TableRatatui::new(rows, widths)
            .header(Row::new(columns.to_vec()).bottom_margin(0))
            .block(
                Block::default()
                    .title(Line::from(title))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightCyan)),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }
}
