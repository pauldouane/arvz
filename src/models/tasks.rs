use reqwest::Client;
use serde::Deserialize;
use crate::models::dag_runs::DagRuns;
use crate::models::task::Task;
use color_eyre::eyre::Result;
use ratatui::widgets::Row;
use crate::config::Airflow;
use crate::style;

#[derive(Debug, Default, Deserialize)]
pub struct Tasks {
    pub task_instances: Vec<Task>,
    pub total_entries: u32,
}

impl Tasks {
    pub fn new() -> Result<Self> {
        Ok(Self {
            task_instances: vec![],
            total_entries: 0,
        })
    }

    pub fn get_total_entries(&self) -> u32 {
        self.total_entries
    }

    pub fn get_tasks_row(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();

        for task in &self.task_instances {
            rows.push(Row::new(vec![
                task.operator.clone().unwrap_or("n/a".to_string()),
                task.task_id.clone(),
                task.try_number.to_string(),
                task.state.clone().unwrap_or("n/a".to_string()),
                format!("{:.2} seconds",
                    if let Some(duration) = task.duration {
                        if duration > 0.0 { duration }
                        else { 0.0 }
                    }
                    else { 0.0 }
                ),
            ]).style(style::get_style_row(&task.state.clone().unwrap_or_default())));
        }
        rows
    }
}