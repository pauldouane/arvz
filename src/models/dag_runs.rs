use serde::Deserialize;
use crate::models::dag_run::DagRun;

#[derive(Deserialize, Debug)]
pub struct DagRuns{
    dag_runs: Vec<DagRun>,
    total_entries: u32,
}