use serde::Deserialize;
use crate::models::conf::Conf;

#[derive(Deserialize, Debug)]
pub struct DagRun {
    conf: Conf,
    dag_id: String,
    dag_run_id: String,
    data_interval_end: String,
    data_interval_start: String,
    end_date: String,
    external_trigger: bool,
    last_scheduling_decision: String,
    logical_date: String,
    note: Option<String>,
    run_type: String,
    start_date: String,
    state: String
}