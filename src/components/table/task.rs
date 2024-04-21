use crate::components::table::table::Table;
use ratatui::prelude::{Color, Style};

#[derive(Default)]
pub struct Task {}

impl Table for Task {
    fn get_columns(&self) -> Vec<&'static str> {
        vec!["OPERATOR", "TASK ID", "TRY NUMBER", "STATE", "DURATION"]
    }

    fn get_border_style(&self) -> Style {
        Style::default().fg(Color::LightCyan)
    }
}
