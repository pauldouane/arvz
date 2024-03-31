use color_eyre::Result;
use reqwest::{Error, Response};
use std::future::Future;

pub trait ModelAirflow {
    fn get_endpoint(&self) -> &str;
    fn deserialize(&self, res: Result<Response, Error>);
}
