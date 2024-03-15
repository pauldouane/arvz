use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Dag {
    pub(crate) dag_id: String,
    default_view: Option<String>,
    description: Option<String>,
    pub(crate) file_token: String,
    fileloc: String,
    has_import_errors: Option<bool>,
    has_task_concurrency_limits: Option<bool>,
    is_active: Option<bool>,
    is_paused: Option<bool>,
    is_subdag: bool,
    last_expired: Option<String>,
    last_parsed_time: Option<String>,
    last_pickled: Option<String>,
    max_active_runs: Option<i32>,
    max_active_tasks: Option<i32>,
    next_dagrun: Option<i32>,
    next_dagrun_create_after: Option<String>,
    next_dagrun_data_interval_end: Option<String>,
    next_dagrun_data_interval_start: Option<String>,
    owners: Vec<String>,
    pickle_id: Option<String>,
    root_dag_id: Option<String>,
    schedule_interval: Option<ScheduleInterval>,
    scheduler_lock: Option<bool>,
    tags: Option<Vec<Tag>>,
    timetable_description: Option<String>,
    catchup: bool,
    concurrency: i32,
    dag_run_timeout: Option<DagRunTimeout>,
    doc_md: Option<String>,
    end_date: Option<String>,
    is_paused_upon_creation: Option<bool>,
    last_parsed: Option<String>,
    orientation: String,
    params: Params,
    render_template_as_native_obj: Option<bool>,
    start_date: Option<String>,
    template_search_path: Option<Vec<String>>,
    timezone: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct ScheduleInterval {}

#[derive(Debug, Default, Deserialize)]
pub struct Tag {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct DagRunTimeout {
    __type: String,
    days: i32,
    microseconds: i32,
    seconds: i32,
}

#[derive(Debug, Default, Deserialize)]
pub struct Params {}
