use std::collections::HashMap;
use reqwest::Client;
use serde::Deserialize;
use crate::config::Airflow;
use color_eyre::eyre::Result;
use serde_json::json;
use serde_json::Value;
use crate::models::log::Log;

#[derive(Debug, Default, Deserialize)]
pub struct Task {
    dag_id: String,
    dag_run_id: String,
    pub(crate) duration: Option<f64>,
    end_date: Option<String>,
    execution_date: String,
    executor_config: String,
    hostname: String,
    map_index: i8,
    max_tries: i8,
    note: Option<String>,
    pub(crate) operator: Option<String>,
    pid: Option<i32>,
    pool: String,
    pool_slots: i16,
    priority_weight: Option<i32>,
    queue: Option<String>,
    queued_when: Option<String>,
    rendered_fields: Option<RenderedFields>,
    sla_miss: Option<SlaMiss>,
    start_date: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) task_id: String,
    trigger: Option<Trigger>,
    triggerer_job: Option<TriggerJob>,
    pub(crate) try_number: f64,
    unixname: String
}

#[derive(Debug, Default, Deserialize)]
pub struct SlaMiss {
    dag_id: String,
    description: String,
    email_sent: bool,
    execution_date: String,
    notification_sent: bool,
    task_id: String,
    timestamp: String
}

#[derive(Debug, Default, Deserialize)]
pub struct Trigger {
    classpath: String,
    created_date: String,
    id: i32,
    kwargs: String,
    triggerer_id: i32
}

#[derive(Debug, Default, Deserialize)]
pub struct TriggerJob {
    dag_id: String,
    end_date: String,
    executor_class: String,
    hostname: String,
    id: i32,
    job_type: String,
    latest_heartbeat: String,
    start_date: String,
    state: String,
    unixname: String
}

#[derive(Debug, Default, Deserialize)]
pub struct RenderedFields {}

impl Task {
    pub async fn clear(&mut self, client: &Client, cfg_airflow: &Airflow, username: &str, password: &str, url: &str) -> Result<()> {
        let body = json!({
            "dry_run": true,
            "task_ids": [self.task_id],
            "only_failed": true,
            "only_running": false,
            "include_subdags": true,
            "include_parentdag": true,
            "reset_dag_runs": true,
            "dag_run_id": self.dag_run_id,
            "include_upstream": false,
            "include_downstream": false,
            "include_future": false,
            "include_past": false
        });
        let task = client
            .post(format!("{}/api/v1/dags/{}/clearTaskInstance", &cfg_airflow.host, self.dag_id))
            .basic_auth(&cfg_airflow.username, Some(&cfg_airflow.password))
            .json(&body)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_logs(&mut self, client: &Client, cfg_airflow: &Airflow, username: &str, password: &str, url: &str, try_number: usize) -> Result<String> {
        let logs = client
            .get(format!("{}/api/v1/dags/{}/dagRuns/{}/taskInstances/{}/logs/{}", &cfg_airflow.host, self.dag_id, self.dag_run_id, self.task_id, try_number))
            .basic_auth(&cfg_airflow.username, Some(&cfg_airflow.password))
            .send()
            .await?
            .text()
            .await?;
        Ok(logs)
    }
}