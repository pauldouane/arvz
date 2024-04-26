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
    Pool,
}

pub type RefreshLayoutFnType = Option<RefCell<Box<dyn FnMut(Mode)>>>;

pub struct ObservableMode {
    mode: Mode,
    refresh_layout_fn: RefreshLayoutFnType,
}

impl Default for ObservableMode {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservableMode {
    pub fn new() -> Self {
        ObservableMode {
            mode: Mode::Pool,
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

    pub fn set_next_mode(&mut self, mode: Mode) {
        self.mode = self.get_next_mode(mode);
        if let Some(refresh_fn) = &self.refresh_layout_fn {
            (refresh_fn.borrow_mut())(mode);
        }
    }

    pub fn get_next_mode(&self, mode: Mode) -> Mode {
        match mode {
            Mode::Pool => Mode::Task,
            _ => self.get(),
        }
    }
}
