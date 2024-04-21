use crate::config::Airflow;
use crate::models::dag_runs::DagRuns;
use crate::models::model_airflow::Data;
use crate::models::model_airflow::ModelAirflow;
use crate::models::task::Task;
use crate::style;
use color_eyre::eyre::Result;
use ratatui::widgets::Row;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Tasks {
    pub task_instances: Vec<Task>,
    pub total_entries: i32,
}

impl Tasks {
    pub fn new() -> Tasks {
        Tasks {
            task_instances: vec![],
            total_entries: 0,
        }
    }
}

impl ModelAirflow for Tasks {
    fn get_endpoint(&self, params: Option<String>) -> String {
        let mut endpoint = String::from("/api/v1/dags/~/dagRuns/~/taskInstances");
        if let Some(params) = params {
            endpoint.push_str(format!("?pool={}", params).as_str());
        }
        endpoint
    }

    fn deserialize(&mut self, res: &str) {
        *self = serde_json::from_str::<Tasks>(res).expect("rgge")
    }

    fn get_total_entries(&self) -> i32 {
        self.total_entries
    }

    fn get_rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();

        for task in &self.task_instances {
            rows.push(
                Row::new(vec![
                    task.operator.clone().unwrap_or("n/a".to_string()),
                    task.task_id.clone(),
                    task.try_number.to_string(),
                    task.state.clone().unwrap_or("n/a".to_string()),
                    format!(
                        "{:.2} seconds",
                        if let Some(duration) = task.duration {
                            if duration > 0.0 {
                                duration
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    ),
                ])
                .style(style::get_style_row(
                    &task.state.clone().unwrap_or_default(),
                )),
            );
        }
        rows
    }

    fn get_element(&self, id: usize) -> Option<Box<&dyn Data>> {
        Some(Box::new(&self.task_instances[id]))
    }
}
