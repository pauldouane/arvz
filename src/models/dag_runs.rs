use reqwest::Client;
use serde::Deserialize;
use crate::models::dag_run::DagRun;
use color_eyre::eyre::Result;
use ratatui::widgets::Row;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct DagRuns{
    pub(crate) dag_runs: Vec<DagRun>,
    total_entries: u32,
}

impl DagRuns {
    pub async fn new(client: &Client) -> Result<Self> {
        let user_name = "root".to_string();
        let password: Option<String> = Some("root".to_string());

        let dag_runs: DagRuns = client
            .get("http://172.24.169.198:8080/api/v1/dags/~/dagRuns")
            .basic_auth(user_name, password)
            .send()
            .await?
            .json::<DagRuns>()
            .await?;

        Ok(dag_runs)
    }

    pub async fn set_dag_runs(&mut self, client: &Client) -> Result<()> {
        let user_name = "root".to_string();
        let password: Option<String> = Some("root".to_string());
        let dag_runs: DagRuns = client
            .get("http://172.24.169.198:8080/api/v1/dags/~/dagRuns")
            .basic_auth(user_name, password)
            .send()
            .await?
            .json::<DagRuns>()
            .await?;

        self.dag_runs = dag_runs.dag_runs;
        self.total_entries = dag_runs.total_entries;
        Ok(())
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
            ]));
        }
        rows
    }
}