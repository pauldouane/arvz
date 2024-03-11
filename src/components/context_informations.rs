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
use crate::models::dag_runs::DagRuns;

#[derive(Default)]
pub struct ContextInformation {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    total_dag_runs: u32,
    total_dag_runs_running: u32,
    total_dag_runs_failed: u32,
    total_dag_runs_scheduled: u32,
    total_dag_runs_success: u32,
    total_dag_runs_queued: u32
}

impl ContextInformation {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            total_dag_runs: 0,
            total_dag_runs_running: 0,
            total_dag_runs_failed: 0,
            total_dag_runs_scheduled: 0,
            total_dag_runs_success: 0,
            total_dag_runs_queued: 0
        }
    }

    pub fn register_context_information(&mut self, dag_runs: &DagRuns) {
        self.total_dag_runs = dag_runs.get_total_entries();
        self.total_dag_runs_running = dag_runs.get_count_dag_run_running();
        self.total_dag_runs_failed = dag_runs.get_count_dag_run_failed();
        self.total_dag_runs_scheduled = dag_runs.get_count_dag_run_scheduled();
        self.total_dag_runs_success = dag_runs.get_count_dag_run_success();
        self.total_dag_runs_queued = dag_runs.get_count_dag_run_queued();
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
        // Align the value on the right to the same starting point
        let text = vec![
            Line::from(vec![
                Span::styled("Dag Runs Number    : ", Style::new().yellow()),
                Span::raw(format!("{}", self.total_dag_runs)),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Running   : ", Style::new().yellow()),
                Span::raw(format!("{}", self.total_dag_runs_running)),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Failed    : ", Style::new().yellow()),
                Span::raw(format!("{}", self.total_dag_runs_failed)),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Scheduled : ", Style::new().yellow()),
                Span::raw(format!("{}", self.total_dag_runs_scheduled)),
            ]),
            Line::from(vec![
                Span::styled("Dag Runs Queued    : ", Style::new().yellow()),
                Span::raw(format!("{}", self.total_dag_runs_queued)),
            ]),
            Line::from(vec![
                Span::styled("ARVZ version       : ", Style::new().yellow()),
                Span::raw(format!("{}", "0.1.0")),
            ]),
        ];
        let block = Paragraph::new(text)
            .block(Block::new());
        f.render_widget(block, area);
        Ok(())
    }
}