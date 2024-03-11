use serde::Deserialize;
use crate::models::conf::Conf;

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
    pub(crate) state: String
}

impl DagRun {
    pub fn new() -> Self {
        Self::default()
    }
}