use crate::components::table::table::Table;
use ratatui::prelude::{Color, Style};

#[derive(Default)]
pub struct DagRun {}

impl DagRun {
    pub fn new() -> Self {
        Self {}
    }
}

impl Table for DagRun {
    fn get_columns(&self) -> Vec<&'static str> {
        vec![
            "DAG ID",
            "STATE",
            "START DATE",
            "END DATE",
            "RUN TYPE",
            "EXTERNAL TRIGGER",
        ]
    }

    fn get_border_style(&self) -> Style {
        Style::default().fg(Color::LightCyan)
    }
}
