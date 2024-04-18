use crate::models::model_airflow::ModelAirflow;
use ratatui::widgets::Row;
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

    fn deserialize(&mut self, res: &str) {
        *self = serde_json::from_str::<PoolCollection>(res).expect("rgge")
    }

    fn get_total_entries(&self) -> i32 {
        self.total_entries
    }

    fn get_rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        for pool in &self.pools {
            rows.push(Row::new(vec![
                pool.deferred_slots.to_string(),
                pool.include_deferred.to_string(),
                pool.name.to_string(),
                pool.occupied_slots.to_string(),
                pool.open_slots.to_string(),
                pool.queued_slots.to_string(),
                pool.running_slots.to_string(),
                pool.scheduled_slots.to_string(),
                pool.slots.to_string(),
            ]));
        }
        rows
    }
}
