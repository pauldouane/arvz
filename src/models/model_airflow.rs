use color_eyre::Result;
use ratatui::layout::Rows;
use ratatui::widgets::Row;
use reqwest::{Error, Response};
use std::fmt::Debug;
use std::future::Future;

pub trait Data {
    fn get_id(&self) -> &str;
}

pub struct ModelView<'a> {
    pub(crate) rows: Vec<Row<'a>>,
}

pub trait ModelAirflow: Send {
    fn get_endpoint(&self, params: Option<String>) -> String;
    fn deserialize(&mut self, res: &str);
    fn get_total_entries(&self) -> i32;
    fn get_rows(&self) -> Vec<Row>;
    fn get_element(&self, id: usize) -> Option<Box<&dyn Data>>;
    fn get_view_model(&self) -> ModelView;
}

impl Debug for dyn ModelAirflow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
