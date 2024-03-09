use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::DagRuns;

#[derive(Default)]
pub struct TableDagRuns {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    columns: Vec<&'static str>,
    dag_runs: DagRuns,
    table_state: TableState,
}

impl TableDagRuns {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            columns: vec!["DAG ID", "STATE", "START DATE", "END DATE", "RUN TYPE", "EXTERNAL TRIGGER"],
            dag_runs: DagRuns::default(),
            table_state: TableState::default(),
        }
    }

    pub(crate) fn set_dag_runs(&mut self, dag_runs: DagRuns) {
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

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Next => {
                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                } else {
                    if let Some(selected_index) = self.table_state.selected() {
                        if selected_index < self.dag_runs.dag_runs.len() - 1 {
                            self.table_state.select(Some(selected_index + 1));
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
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let rows = self.dag_runs.get_dag_runs_rows_context();
        // Set the width of the columns
        let widths = self.columns.iter().map(|_| {
            Constraint::Percentage((100 / self.columns.len() as u16).into())
        }).collect::<Vec<_>>();
        let title = Line::from(vec![
            Span::styled(" Context(", Style::new().light_cyan()),
            Span::styled("all", Style::new().magenta()),
            Span::styled(") ", Style::new().light_cyan()),
        ]);
        let table = Table::new(rows, widths)
            .header(
                Row::new(self.columns.iter().map(|&s| s).collect::<Vec<_>>())
                    .bottom_margin(0)
            )
            .block(Block::default().title(title).title_alignment(Alignment::Center).borders(Borders::ALL).border_style(Style::default().fg(Color::LightCyan)))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");

        f.render_stateful_widget(table, area, &mut self.table_state);
        Ok(())
    }
}