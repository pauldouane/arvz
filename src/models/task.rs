use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Task {
    dag_id: String,
    dag_run_id: String,
    pub(crate) duration: f64,
    end_date: String,
    execution_date: String,
    executor_config: String,
    hostname: String,
    map_index: i8,
    max_tries: i8,
    note: Option<String>,
    pub(crate) operator: String,
    pid: i32,
    pool: String,
    pool_slots: i16,
    priority_weight: i32,
    queue: String,
    queued_when: String,
    rendered_fields: Option<RenderedFields>,
    sla_miss: Option<SlaMiss>,
    start_date: String,
    pub(crate) state: String,
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