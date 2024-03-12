use reqwest::Client;
use serde::Deserialize;
use crate::models::dag_run::DagRun;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Row;
use crate::config::Airflow;
use crate::models::tasks::Tasks;
use crate::style;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct DagRuns{
    pub(crate) dag_runs: Vec<DagRun>,
    total_entries: u32,
}

impl DagRuns {
    pub fn new() -> Self {
        Self {
            dag_runs: vec![],
            total_entries: 0,
        }
    }

    pub async fn set_dag_runs(&mut self, client: &Client, username: &str, password: &str, url: &str) -> Result<()> {
        let dag_runs: DagRuns = client
            .get(format!("{}/api/v1/dags/~/dagRuns?order_by=-start_date", url))
            .basic_auth(username, Some(password))
            .send()
            .await?
            .json::<DagRuns>()
            .await?;

        self.dag_runs = dag_runs.dag_runs;
        self.total_entries = dag_runs.total_entries;
        Ok(())
    }

    pub async fn get_task(&mut self, client: &Client, cfg_airflow: &Airflow, username: &str, password: &str, url: &str, index: usize) -> Result<Tasks> {
        let task: Tasks = client
            .get(format!(
                "{}/api/v1/dags/{}/dagRuns/{}/taskInstances", &cfg_airflow.host,
                self.dag_runs[index].dag_id, self.dag_runs[index].dag_run_id))
            .basic_auth(&cfg_airflow.username, Some(&cfg_airflow.password))
            .send()
            .await?
            .json::<Tasks>()
            .await?;
        Ok(task)
    }

    pub fn get_total_entries(&self) -> u32 {
        self.total_entries
    }

    pub fn get_count_dag_run_running(&self) -> u32 {
        self.dag_runs.iter().filter(|dag_run| dag_run.state == "running").count() as u32
    }

    pub fn get_count_dag_run_failed(&self) -> u32 {
        self.dag_runs.iter().filter(|dag_run| dag_run.state == "failed").count() as u32
    }

    pub fn get_count_dag_run_scheduled(&self) -> u32 {
        self.dag_runs.iter().filter(|dag_run| dag_run.state == "scheduled").count() as u32
    }

    pub fn get_count_dag_run_success(&self) -> u32 {
        self.dag_runs.iter().filter(|dag_run| dag_run.state == "success").count() as u32
    }

    pub fn get_count_dag_run_queued(&self) -> u32 {
        self.dag_runs.iter().filter(|dag_run| dag_run.state == "queued").count() as u32
    }

    pub fn filter_runs_by_dag_id<'a>(&'a self, dag_id: &str) -> Vec<&'a DagRun> {
        self.dag_runs.iter().filter(|dag_run| dag_run.dag_id.contains(dag_id)).collect::<Vec<&'a DagRun>>()
    }

    pub fn get_dag_runs_rows_filtered(&self, dag_id: &str) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        let filtered_dag_runs: Vec<&DagRun>= self.filter_runs_by_dag_id(dag_id);

        for dag_run in filtered_dag_runs {
            rows.push(Row::new(vec![
                dag_run.dag_id.clone(),
                dag_run.state.clone(),
                dag_run.data_interval_start.clone(),
                dag_run.data_interval_end.clone(),
                dag_run.run_type.clone(),
                dag_run.external_trigger.to_string().clone(),
            ]).style(
                style::get_style_row(&dag_run.state)
            ));
        }
        rows
    }

    pub fn get_dag_runs_rows_context(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();

        for dag_run in &self.dag_runs {
            rows.push(Row::new(vec![
                dag_run.dag_id.clone(),
                dag_run.state.clone(),
                dag_run.data_interval_start.clone(),
                dag_run.data_interval_end.clone(),
                dag_run.run_type.clone(),
                dag_run.external_trigger.to_string().clone(),
            ]).style(
                style::get_style_row(&dag_run.state)
            ));
        }
        rows
    }
}