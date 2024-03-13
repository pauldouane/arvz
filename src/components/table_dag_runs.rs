use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::title;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Str;
use tokio::sync::mpsc::UnboundedSender;
use tracing_subscriber::fmt::format;

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
    pub(crate) table_state: TableState,
    pub(crate) user_search: Option<String>,
    client: Client,
    pub(crate) tasks: Option<Tasks>,
    pub(crate) table_tasks_state: TableState,
    pub(crate) log: String,
    pub try_number: usize,
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
            tasks: None,
            table_tasks_state: TableState::default(),
            log: String::from(""),
            try_number: 1,
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
        let table_state = if self.mode == Mode::Task {
            &mut self.table_tasks_state
        } else {
            &mut self.table_state
        };
        match action {
            Action::Next => {
                if table_state.selected().is_none() {
                    table_state.select(Some(0));
                } else {
                    if let Some(selected_index) = table_state.selected() {
                       if self.mode == Mode::Task {
                           if selected_index < self.tasks.as_ref().unwrap().task_instances.len() - 1 {
                               table_state.select(Some(selected_index + 1));
                           }
                       } else {
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
                }
            },
            Action::Previous => {
                // If is the first element, don't do anything
                if let Some(selected_index) = table_state.selected() {
                    if selected_index > 0 {
                        table_state.select(Some(selected_index - 1));
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
            Span::styled(if self.mode == Mode::Task {
                format!("{}", self.dag_runs.dag_runs[self.table_state.selected().unwrap()].dag_run_id)
            } else {
                if self.mode == Mode::Log {
                    format!("{}", self.tasks.as_ref().unwrap().task_instances[self.table_tasks_state.selected().unwrap()].task_id)
                } else {
                    String::from("all")
                }
            }, Style::new().magenta()),
            Span::styled(")", Style::new().light_cyan()),
            Span::styled("[", Style::new().white()),
            Span::styled(format!("{}",
                                 if let Some(try_number) = self.try_number {
                                     try_number as u32
                                 } else {
                                     self.dag_runs.get_total_entries()
                                 }), Style::new().light_yellow()),
            Span::styled("] ", Style::new().white()),
        ];
        if let Some(search) = &self.user_search {
            title.push(Span::raw("<"));
            title.push(Span::styled(format!("/{}", search), Style::new().bg(Color::Green)));
            title.push(Span::raw("> "));
        }


        if self.mode == Mode::Log {
            let log = Paragraph::new(self.log.as_str())
                .block(Block::default().title(Line::from(title)).title_alignment(Alignment::Center).borders(Borders::ALL).border_style(Style::default().fg(Color::LightBlue)))
                .wrap(Wrap { trim: true });
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let vertical_scroll = 0; // from app state
            let mut scrollbar_state = ScrollbarState::new(self.log.len()).position(vertical_scroll);
            f.render_widget(log, area);
            f.render_stateful_widget(
                scrollbar,
                area.inner(&Margin {
                    // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
            return Ok(());
        } else {
            let table = Table::new(rows, widths)
                .header(
                    Row::new(self.columns.iter().map(|&s| s).collect::<Vec<_>>())
                        .bottom_margin(0)
                )
                .block(Block::default().title(Line::from(title)).title_alignment(Alignment::Center).borders(Borders::ALL).border_style(Style::default().fg(Color::LightBlue)))
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

            f.render_stateful_widget(table, area,
             if self.mode == Mode::Task {
                 &mut self.table_tasks_state
             } else {
                 &mut self.table_state
             }
            );
        }
        Ok(())
    }
}