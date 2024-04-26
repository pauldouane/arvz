use crate::config::Airflow;
use crate::models::conf::Conf;
use crate::models::model_airflow::Data;
use crate::models::tasks::Tasks;
use color_eyre::eyre::Result;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;

use super::dag::Dag;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct DagRun {
    conf: Conf,
    pub(crate) dag_id: String,
    pub(crate) dag_run_id: String,
    pub(crate) data_interval_end: String,
    pub(crate) data_interval_start: String,
    end_date: Option<String>,
    pub(crate) external_trigger: bool,
    last_scheduling_decision: Option<String>,
    logical_date: String,
    note: Option<String>,
    pub(crate) run_type: String,
    start_date: Option<String>,
    pub(crate) state: String,
}

impl DagRun {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn clear(
        &mut self,
        client: &Client,
        cfg_airflow: &Airflow,
        username: &str,
        password: &str,
        url: &str,
    ) -> Result<()> {
        let mut map = HashMap::new();
        map.insert("dry_run", false);
        let task = client
            .post(format!(
                "{}/api/v1/dags/{}/dagRuns/{}/clear",
                &cfg_airflow.host, self.dag_id, self.dag_run_id
            ))
            .basic_auth(&cfg_airflow.username, Some(&cfg_airflow.password))
            .json(&map)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_source_code(&mut self, client: &Client, cfg: &Airflow) -> Result<String> {
        let res = client
            .get(format!("{}/api/v1/dags/{}/details", cfg.host, self.dag_id))
            .basic_auth(&cfg.username, Some(&cfg.password))
            .send()
            .await?;
        if res.status() == StatusCode::OK {
            let dag = res.json::<Dag>().await?;
            let source_code = client
                .get(format!("{}/api/v1/dagSources/{}", cfg.host, dag.file_token))
                .basic_auth(&cfg.username, Some(&cfg.password))
                .send()
                .await?
                .text()
                .await?;
            return Ok(source_code);
        }
        Ok(String::from(""))
    }
}

impl Data for DagRun {
    fn get_id(&self) -> &str {
        &self.dag_id
    }
}
