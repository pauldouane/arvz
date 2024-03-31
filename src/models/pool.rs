use crate::models::model_airflow::ModelAirflow;
use reqwest::{Error, Response};
use serde::Deserialize;
use std::future::Future;

#[derive(Debug, Deserialize)]
pub struct Pool {
    deferred_slots: i32,
    description: Option<String>,
    include_deferred: bool,
    name: String,
    occupied_slots: i32,
    open_slots: i32,
    queued_slots: i32,
    running_slots: i32,
    scheduled_slots: i32,
    slots: i32,
}

#[derive(Debug, Deserialize)]
pub struct PoolCollection {
    pools: Vec<Pool>,
    total_entries: i32,
}

impl PoolCollection {
    pub fn new() -> PoolCollection {
        PoolCollection {
            pools: vec![],
            total_entries: 0,
        }
    }
}

impl ModelAirflow for PoolCollection {
    fn get_endpoint(&self) -> &str {
        "/api/v1/pools"
    }

    fn deserialize(&self, res: color_eyre::Result<Response, Error>) {
        panic!("{:?}", res);
    }
}
