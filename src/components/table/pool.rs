use crate::components::table::table::Table;

#[derive(Default)]
pub struct Pool {
}

impl Pool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Table for Pool {
    fn get_columns(&self) -> Vec<&'static str> {
        vec!["Pool"]
    }
}