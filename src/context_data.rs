use crate::config::{Airflow, Config};
use crate::mode::Mode;
use crate::models::dag_runs::DagRuns;
use crate::models::model_airflow::ModelAirflow;
use color_eyre::Result;
use reqwest::Client;
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::pool::{Pool, PoolCollection};

#[derive(Debug)]
pub struct ContextData {
    pub pool_collection: Option<Box<dyn ModelAirflow>>,
    pub airflow_config: Airflow,
    client: Client,
}

impl ContextData {
    pub(crate) fn new() -> ContextData {
        ContextData {
            pool_collection: Some(Box::new(PoolCollection::new())),
            airflow_config: Airflow::default(),
            client: Client::default(),
        }
    }

    pub(crate) fn handle_airflow_config(&mut self, airflow_config: Airflow) {
        self.airflow_config = airflow_config.clone();
        log::info!("oui");
    }

    pub async fn log_test(&mut self) {
        log::info!("refresh");
    }

    pub fn get_model_by_mode(&self, mode: Mode) -> Result<&Option<Box<dyn ModelAirflow>>> {
        match mode {
            Mode::Pool => Ok(&self.pool_collection),
            _ => todo!(),
        }
    }

    pub fn get_dese(&mut self, mode: Mode, data: &str) {
        match mode {
            Mode::Pool => self.pool_collection.as_mut().unwrap().deserialize(&data),
            _ => log::error!("Mode not known"),
        };
    }

    pub async fn refresh(&mut self, mode: Mode) -> Result<()> {
        let model = self.get_model_by_mode(mode).unwrap();
        let data = self
            .client
            .get(
                self.airflow_config.host.as_str().to_owned()
                    + model
                        .as_ref()
                        .expect("Unable to get ref of the model")
                        .get_endpoint(),
            )
            .basic_auth(
                &self.airflow_config.username,
                Some(&self.airflow_config.password),
            )
            .send()
            .await?
            .text()
            .await?;
        self.get_dese(mode, &data);
        /*
        match mode {
            Mode::Pool => self.pool_collection.as_mut().unwrap().deserialize(&data),
            _ => log::error!("Mode not known")
        }
        */
        Ok(())
    }
}
