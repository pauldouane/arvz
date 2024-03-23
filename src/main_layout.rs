use crate::mode::Mode;
use ratatui::prelude::Constraint;
use ratatui::prelude::Direction;
use ratatui::prelude::Layout;
use ratatui::prelude::Rect;
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Chunk {
    Context(usize),
    Command,
    Table,
    Status,
}

pub struct MainLayout {
    pub tui_size: Option<Rc<RefCell<Rect>>>,
    pub main_chunk: Option<Rc<[Rect]>>,
    pub context_chunk: Option<Rc<[Rect]>>,
    pub command_chunk: Option<Rc<[Rect]>>,
    pub table_chunk: Option<Rc<[Rect]>>,
    pub status_chunk: Option<Rc<[Rect]>>,
}

impl Default for MainLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl MainLayout {
    pub fn new() -> MainLayout {
        MainLayout {
            tui_size: None,
            main_chunk: None,
            context_chunk: None,
            command_chunk: None,
            table_chunk: None,
            status_chunk: None,
        }
    }

    pub fn get_constraints(&self, mode: &Mode) -> [Constraint; 4] {
        [
            Constraint::Length(6),
            if mode == &Mode::Search || mode == &Mode::Command {
                Constraint::Length(3)
            } else {
                Constraint::Percentage(0)
            },
            Constraint::Fill(1),
            Constraint::Length(1),
        ]
    }

    pub fn set_tui_size(&mut self, tui_size: Rc<RefCell<Rect>>) {
        self.tui_size = Some(tui_size);
    }

    pub fn set_main_layout(&mut self, mode: &Mode) {
        if let Some(tui_size) = &self.tui_size {
            self.main_chunk = Some(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(self.get_constraints(mode))
                    .margin(1)
                    .split(*tui_size.borrow_mut()),
            );
        }
        if let Some(main_chunk) = &self.main_chunk {
            self.context_chunk = Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Length(50),
                        Constraint::Fill(1),
                        Constraint::Length(22),
                    ])
                    .split(main_chunk[0]),
            );
            self.command_chunk = Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(100)])
                    .split(main_chunk[1]),
            );
            self.table_chunk = Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(100)])
                    .split(main_chunk[2]),
            );
            self.status_chunk = Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(100)])
                    .split(main_chunk[3]),
            );
        }
    }

    pub fn get_chunk(&self, chunk: Chunk) -> Rect {
        match chunk {
            Chunk::Context(area) => self.context_chunk.clone().unwrap()[area],
            Chunk::Command => self.command_chunk.clone().unwrap()[0],
            Chunk::Table => self.table_chunk.clone().unwrap()[0],
            Chunk::Status => self.status_chunk.clone().unwrap()[0],
        }
    }
}
