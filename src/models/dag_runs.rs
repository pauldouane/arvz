use crate::config::Airflow;
use crate::models::dag_run::DagRun;
use crate::models::model_airflow::Data;
use crate::models::model_airflow::ModelAirflow;
use crate::models::tasks::Tasks;
use crate::style;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Row;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct DagRuns {
    pub(crate) dag_runs: Vec<DagRun>,
    total_entries: i32,
}

impl ModelAirflow for DagRuns {
    fn get_endpoint(&self, params: Option<String>) -> String {
        String::from("/taskinstance/list/?_flt_3_pool=default_pool")
    }

    fn deserialize(&mut self, res: &str) {
        *self = serde_json::from_str::<DagRuns>(res).expect("rgge")
    }

    fn get_total_entries(&self) -> i32 {
        self.total_entries
    }

    fn get_rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        for dag_run in &self.dag_runs {
            rows.push(Row::new(vec![
                dag_run.dag_id.to_string(),
                dag_run.state.to_string(),
                dag_run.data_interval_start.to_string(),
                dag_run.data_interval_end.to_string(),
                dag_run.run_type.to_string(),
                dag_run.external_trigger.to_string(),
            ]));
        }
        rows
    }

    fn get_element(&self, id: usize) -> Option<Box<&dyn Data>> {
        Some(Box::new(&self.dag_runs[id]))
    }
}
