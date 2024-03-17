use std::collections::HashMap;
use std::rc::Rc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use log::log;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::chunk::Chunk;
use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::{self, ContextInformation};
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table_dag_runs::TableDagRuns;
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::DagRuns;
use crate::{
    action::Action,
    components::{fps::FpsCounter, Component},
    config::Config,
    mode::Mode,
    tui,
};

pub struct App<'a> {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub mode: Mode,
    pub components: HashMap<&'a Rc<[Rect]>, Box<dyn Component>>,
}

impl App<'_> {
    pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let mode = Mode::DagRun;
        let client = Client::new();
        Ok(Self {
            tick_rate,
            frame_rate,
            should_quit: false,
            should_suspend: false,
            last_tick_key_events: Vec::new(),
            config,
            mode,
            components: HashMap::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let mut tui = tui::Tui::new()?;
        // tui.mouse(true);
        tui.enter()?;

        // Get all components
        tui.set_components(&mut self.components);

        loop {
            // If the task mode is selected, then fetch the tasks for the selected dag_run
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                log::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single key action,
                                // then consider it for multi-key combinations.
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    log::info!("Got action: {action:?}");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        };
                    }
                    _ => {}
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {})?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            for (chunk, component) in self.components.iter_mut() {
                                component.draw(f, chunk[0]).unwrap();
                            }
                        })?;
                    }
                    _ => todo!(),
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
