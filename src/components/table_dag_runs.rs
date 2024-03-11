use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::title;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use crate::mode::Mode;
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::DagRuns;
use crate::models::tasks::Tasks;
use crate::utils::get_user_input_by_key;

#[derive(Default)]
pub struct TableDagRuns {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    columns: Vec<&'static str>,
    dag_runs: DagRuns,
    table_state: TableState,
    user_search: Option<String>,
    client: Client,
    pub(crate) tasks: Option<Tasks>
}

impl TableDagRuns {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            mode: Mode::DagRun,
            columns: vec!["DAG ID", "STATE", "START DATE", "END DATE", "RUN TYPE", "EXTERNAL TRIGGER"],
            dag_runs: DagRuns::default(),
            table_state: TableState::default(),
            user_search: None,
            client: Client::new(),
            tasks: None
        }
    }

    pub fn set_dag_runs(&mut self, dag_runs: DagRuns) {
        self.dag_runs = dag_runs;
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

    fn handle_mode(&mut self, mode: Mode) -> Result<()> {
        if self.mode == Mode::Search {
            self.table_state.select(None);
        }
        self.mode = mode;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.mode == Mode::Search {
            get_user_input_by_key(key.code, &mut self.user_search);
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Next => {
                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                } else {
                    if let Some(selected_index) = self.table_state.selected() {
                        if let Some(search) = &self.user_search {
                            let filtered = self.dag_runs.get_dag_runs_rows_filtered(search);
                            if selected_index < filtered.len() - 1 {
                                self.table_state.select(Some(selected_index + 1));
                            }
                        } else {
                            if selected_index < self.dag_runs.dag_runs.len() - 1 {
                                self.table_state.select(Some(selected_index + 1));
                            }
                        }
                    }
                }
            },
            Action::Previous => {
                // If is the first element, don't do anything
                if let Some(selected_index) = self.table_state.selected() {
                    if selected_index > 0 {
                        self.table_state.select(Some(selected_index - 1));
                    }
                }
            },
            Action::DagRun => {
                self.handle_mode(Mode::DagRun)?;
                self.columns = vec!["DAG ID", "STATE", "START DATE", "END DATE", "RUN TYPE", "EXTERNAL TRIGGER"];
            },
            Action::Task => {
                self.handle_mode(Mode::Task)?;
                self.columns = vec![
                    "OPERATOR",
                    "TASK ID",
                    "TRY NUMBER",
                    "STATE",
                    "DURATION",
                ];
            },
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let rows: Vec<Row> = if self.mode == Mode::Task {
            if let Some(tasks) = &self.tasks {
                tasks.get_tasks_row()
            } else {
                vec![]
            }
        } else {
            if let Some(search) = &self.user_search {
                self.dag_runs.get_dag_runs_rows_filtered(search)
            } else {
                self.dag_runs.get_dag_runs_rows_context()
            }
        };
        // Set the width of the columns
        let widths = self.columns.iter().map(|_| {
            Constraint::Percentage((100 / self.columns.len() as u16).into())
        }).collect::<Vec<_>>();
        let mut title = vec![
            Span::styled(format!(" {:?}(", self.mode), Style::new().light_cyan()),
            Span::styled("all", Style::new().magenta()),
            Span::styled(")", Style::new().light_cyan()),
            Span::styled("[", Style::new().white()),
            Span::styled(format!("{}", rows.len()), Style::new().light_yellow()),
            Span::styled("] ", Style::new().white()),
        ];
        if let Some(search) = &self.user_search {
            title.push(Span::raw("<"));
            title.push(Span::styled(format!("/{}", search), Style::new().bg(Color::Green)));
            title.push(Span::raw("> "));
        }
        let table = Table::new(rows, widths)
            .header(
                Row::new(self.columns.iter().map(|&s| s).collect::<Vec<_>>())
                    .bottom_margin(0)
            )
            .block(Block::default().title(Line::from(title)).title_alignment(Alignment::Center).borders(Borders::ALL).border_style(Style::default().fg(Color::LightBlue)))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(table, area, &mut self.table_state);
        Ok(())
    }
}