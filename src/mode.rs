use ratatui::prelude::Rect;
use std::borrow::Borrow;
use std::cell::RefCell;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    DagRun,
    Search,
    Task,
    Log,
    Code,
    Command,
}

pub struct ObservableMode {
    mode: Mode,
    refresh_layout_fn: Option<RefCell<Box<dyn FnMut(Mode)>>>,
}

impl ObservableMode {
    pub fn new() -> Self {
        ObservableMode {
            mode: Mode::DagRun,
            refresh_layout_fn: None,
        }
    }

    pub fn get(&self) -> Mode {
        self.mode
    }

    pub fn set_refresh_layout_fn<F>(&mut self, callback: F)
    where
        F: FnMut(Mode) + 'static,
    {
        self.refresh_layout_fn = Some(RefCell::new(Box::new(callback)));
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        if let Some(refresh_fn) = &self.refresh_layout_fn {
            (refresh_fn.borrow_mut())(mode);
        }
    }
}
