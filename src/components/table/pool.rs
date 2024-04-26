use crate::components::table::table::Table;
use ratatui::prelude::{Color, Style};

#[derive(Default)]
pub struct Pool {}

impl Pool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Table for Pool {
    fn get_columns(&self) -> Vec<&'static str> {
        vec![
            "deferred_slots",
            "include_deferred",
            "name",
            "occupied_slots",
            "open_slots",
            "queued_slots",
            "running_slots",
            "scheduled_slots",
            "slots",
        ]
    }

    fn get_border_style(&self) -> Style {
        Style::default().fg(Color::LightCyan)
    }
}
