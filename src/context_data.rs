use crate::config::{Airflow, Config};
use crate::mode::Mode;
use crate::models::dag_runs::DagRuns;
use crate::models::model_airflow::ModelAirflow;
use color_eyre::Result;
use reqwest::Client;
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::pool::{Pool, PoolCollection};

pub struct ContextData {
    pool_collection: Option<Rc<RefCell<dyn ModelAirflow>>>,
    airflow_config: Airflow,
    client: Client,
}

impl ContextData {
    pub(crate) fn new() -> ContextData {
        ContextData {
            pool_collection: Some(Rc::new(RefCell::new(PoolCollection::new()))),
            airflow_config: Airflow::default(),
            client: Client::default(),
        }
    }

    pub(crate) fn handle_airflow_config(&mut self, airflow_config: Airflow) {
        self.airflow_config = airflow_config.clone();
    }

    pub async fn refresh(&self, mode: Mode) {
        let model = self.get_model_by_mode(mode).clone().unwrap();
        let future = self
            .client
            .get(self.airflow_config.host.as_str().to_owned() + model.borrow_mut().get_endpoint())
            .basic_auth(
                &self.airflow_config.username,
                Some(&self.airflow_config.password),
            )
            .send()
            .await;
        model.borrow_mut().deserialize(future);
    }

    fn get_model_by_mode(&self, mode: Mode) -> &Option<Rc<RefCell<dyn ModelAirflow>>> {
        match mode {
            Mode::Pool => &self.pool_collection,
            _ => &self.pool_collection,
        }
    }
}
