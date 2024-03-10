use ratatui::style::{Color, Style};

pub fn get_style_row_context(state: &str) -> Style {
    match state {
        "running" => Style::default().fg(Color::LightCyan),
        "failed" => Style::default().fg(Color::Red),
        "scheduled" => Style::default().fg(Color::Yellow),
        "success" => Style::default().fg(Color::Green),
        "queued" => Style::default().fg(Color::Gray),
        _ => Style::default().fg(Color::White),
    }
}