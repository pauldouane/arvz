use crate::config::{Airflow, Config};
use crate::mode::Mode;
use crate::models::dag_runs::DagRuns;
use crate::models::model_airflow::ModelAirflow;
use crate::models::tasks::Tasks;
use color_eyre::Result;
use reqwest::Client;
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::pool::{Pool, PoolCollection};

#[derive(Debug)]
pub struct ContextData {
    pub pool_collection: Option<Box<dyn ModelAirflow>>,
    pub task_instances: Option<Box<dyn ModelAirflow>>,
    pub airflow_config: Airflow,
    pub params: Option<String>,
    client: Client,
}

impl ContextData {
    pub(crate) fn new() -> ContextData {
        ContextData {
            pool_collection: Some(Box::new(PoolCollection::new())),
            task_instances: Some(Box::new(Tasks::new())),
            airflow_config: Airflow::default(),
            params: None,
            client: Client::default(),
        }
    }

    pub(crate) fn handle_airflow_config(&mut self, airflow_config: Airflow) {
        self.airflow_config = airflow_config.clone();
    }

    pub fn get_model_by_mode(&self, mode: Mode) -> Result<&Option<Box<dyn ModelAirflow>>> {
        match mode {
            Mode::Pool => Ok(&self.pool_collection),
            Mode::Task => Ok(&self.task_instances),
            _ => todo!(),
        }
    }

    pub fn get_mut_model_by_mode(
        &mut self,
        mode: Mode,
    ) -> Result<&mut Option<Box<dyn ModelAirflow>>> {
        match mode {
            Mode::Pool => Ok(&mut self.pool_collection),
            Mode::Task => Ok(&mut self.task_instances),
            _ => todo!(),
        }
    }

    pub fn get_deserialize(&mut self, mode: Mode, data: &str) {
        match mode {
            Mode::Pool => self.pool_collection.as_mut().unwrap().deserialize(&data),
            Mode::Task => self.task_instances.as_mut().unwrap().deserialize(&data),
            _ => log::error!("Mode not known"),
        };
    }

    pub async fn refresh(&mut self, mode: Mode) -> Result<()> {
        let model = self.get_model_by_mode(mode).unwrap();
        let data = self
            .client
            .get(
                self.airflow_config.host.as_str().to_owned()
                    + &model
                        .as_ref()
                        .expect("Unable to get ref of the model")
                        .get_endpoint(self.params.clone()),
            )
            .basic_auth(
                &self.airflow_config.username,
                Some(&self.airflow_config.password),
            )
            .send()
            .await?
            .text()
            .await?;
        log::info!("{:?}", mode);
        self.get_deserialize(mode, &data);
        Ok(())
    }
}
