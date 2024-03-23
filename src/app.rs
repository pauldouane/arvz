use std::cell::RefCell;
use std::rc::Rc;

use crate::main_layout::{self, Chunk};
use crate::mode::ObservableMode;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use log::log;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table_dag_runs::TableDagRuns;
use crate::main_layout::MainLayout;
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::DagRuns;
use crate::{
    action::Action,
    components::{fps::FpsCounter, Component},
    config::Config,
    mode::Mode,
    tui,
};

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub observable_mode: ObservableMode,
    pub main_layout: Rc<RefCell<MainLayout>>,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub last_dag_runs_call: Instant,
    pub last_task_call: Option<Instant>,
    pub dag_runs: DagRuns,
    client: Client,
    context_information: ContextInformation,
    shortcut: Shortcut,
    ascii: Ascii,
    table_dag_runs: TableDagRuns,
    status_bar: StatusBar,
    command_search: CommandSearch,
    command: Command,
}

impl App {
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
            observable_mode: ObservableMode::new(),
            config,
            main_layout: Rc::new(RefCell::new(MainLayout::new())),
            last_tick_key_events: Vec::new(),
            last_dag_runs_call: Instant::now(),
            last_task_call: None,
            dag_runs: DagRuns::new(),
            client,
            context_information: ContextInformation::new(),
            shortcut: Shortcut::new(),
            ascii: Ascii::new(),
            table_dag_runs: TableDagRuns::new(),
            status_bar: StatusBar::new(),
            command_search: CommandSearch::new(),
            command: Command::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        self.dag_runs
            .set_dag_runs(
                &self.client,
                &self.config.airflow.username,
                &self.config.airflow.password,
                &self.config.airflow.host,
            )
            .await?;
        self.last_dag_runs_call = Instant::now();
        // Set the initial dag_runs state for the table widget
        self.table_dag_runs.set_dag_runs(self.dag_runs.clone());
        let mut tui = tui::Tui::new()?;
        // tui.mouse(true);
        tui.enter()?;

        // Register action and config handlers for all components
        self.context_information
            .register_action_handler(action_tx.clone())?;
        self.context_information
            .register_config_handler(self.config.clone())?;

        self.shortcut.register_action_handler(action_tx.clone())?;
        self.shortcut.register_config_handler(self.config.clone())?;

        self.ascii.register_action_handler(action_tx.clone())?;
        self.ascii.register_config_handler(self.config.clone())?;

        self.table_dag_runs
            .register_action_handler(action_tx.clone())?;
        self.table_dag_runs
            .register_config_handler(self.config.clone())?;

        // Init the area for all components
        self.context_information.init(tui.size()?)?;
        self.shortcut.init(tui.size()?)?;
        self.ascii.init(tui.size()?)?;
        self.table_dag_runs.init(tui.size()?)?;

        self.main_layout
            .borrow_mut()
            .set_tui_size(Rc::new(RefCell::new(tui.size().unwrap())));
        self.main_layout
            .borrow_mut()
            .set_main_layout(&self.observable_mode.get());

        let main_layout_rc = Rc::clone(&self.main_layout);
        self.observable_mode
            .set_refresh_layout_fn(move |mode| main_layout_rc.borrow_mut().set_main_layout(&mode));

        loop {
            if self.last_dag_runs_call.elapsed().as_secs() == 3 {
                self.dag_runs
                    .set_dag_runs(
                        &self.client,
                        &self.config.airflow.username,
                        &self.config.airflow.password,
                        &self.config.airflow.host,
                    )
                    .await?;
                self.last_dag_runs_call = Instant::now();
                self.table_dag_runs.set_dag_runs(self.dag_runs.clone());
            }

            // If the task mode is selected, then fetch the tasks for the selected dag_run
            if self.observable_mode.get() == Mode::Task {
                if let Some(last_task_call) = self.last_task_call {
                    if last_task_call.elapsed().as_secs() >= 2 {
                        self.table_dag_runs.tasks = Some(
                            self.dag_runs
                                .get_task(
                                    &self.client,
                                    &self.config.airflow,
                                    &self.config.airflow.username,
                                    &self.config.airflow.password,
                                    &self.config.airflow.host,
                                    self.table_dag_runs.table_state.selected().unwrap_or(0),
                                )
                                .await?,
                        );
                        self.last_task_call = Some(Instant::now());
                    }
                } else {
                    self.last_task_call = Some(Instant::now());
                }
            }
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        if let Some(keymap) =
                            self.config.keybindings.get(&self.observable_mode.get())
                        {
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
                if let Some(action) = self.context_information.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
                if let Some(action) = self.shortcut.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
                if let Some(action) = self.ascii.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
                if let Some(action) = self.command_search.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
                if let Some(action) = self.command.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
                if let Some(action) = self.table_dag_runs.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            let r = self.context_information.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self.shortcut.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self.ascii.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self.table_dag_runs.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            let r = self
                                .context_information
                                .draw(f, self.main_layout.borrow().get_chunk(Chunk::Context(0)));
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            if self.observable_mode.get() == Mode::Search {
                                let r = self
                                    .command_search
                                    .draw(f, self.main_layout.borrow().get_chunk(Chunk::Command));
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }

                            if self.observable_mode.get() == Mode::Command {
                                let r = self
                                    .command
                                    .draw(f, self.main_layout.borrow().get_chunk(Chunk::Command));
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }

                            let r = self
                                .shortcut
                                .draw(f, self.main_layout.borrow().get_chunk(Chunk::Context(1)));
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self
                                .ascii
                                .draw(f, self.main_layout.borrow().get_chunk(Chunk::Context(2)));
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self
                                .table_dag_runs
                                .draw(f, self.main_layout.borrow().get_chunk(Chunk::Table));
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }

                            let r = self
                                .status_bar
                                .draw(f, self.main_layout.borrow().get_chunk(Chunk::Status));
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::Search => {
                        self.status_bar.mode_breadcrumb.push(Mode::DagRun);
                        self.observable_mode.set_mode(Mode::Search);
                        self.status_bar.register_mode(self.observable_mode.get());
                        self.observable_mode.set_mode(Mode::Search);
                    }
                    Action::Command => {
                        self.status_bar.mode_breadcrumb.push(Mode::DagRun);
                        self.observable_mode.set_mode(Mode::Command);
                        self.status_bar.register_mode(self.observable_mode.get());
                        self.observable_mode.set_mode(Mode::Command);
                    }
                    Action::DagRun => {
                        self.status_bar.mode_breadcrumb.clear();
                        self.command.command = None;
                        self.observable_mode.set_mode(Mode::DagRun);
                        self.status_bar.register_mode(self.observable_mode.get());
                        self.table_dag_runs.position = None;
                        self.observable_mode.set_mode(Mode::DagRun);
                    }
                    Action::Code => {
                        self.observable_mode.set_mode(Mode::Code);
                        self.status_bar.mode_breadcrumb.push(Mode::DagRun);
                        self.status_bar.register_mode(self.observable_mode.get());
                        self.table_dag_runs
                            .handle_mode(self.observable_mode.get())?;
                        let source_code = self.table_dag_runs.dag_runs.dag_runs
                            [self.table_dag_runs.table_state.selected().unwrap()]
                        .get_source_code(&self.client, &self.config.airflow)
                        .await?;
                        self.table_dag_runs.code = source_code.clone();
                        self.observable_mode.set_mode(Mode::Code);
                    }
                    Action::Clear => {
                        if self.observable_mode.get() == Mode::DagRun
                            && self.table_dag_runs.table_state.selected().is_some()
                        {
                            self.dag_runs.dag_runs
                                [self.table_dag_runs.table_state.selected().unwrap()]
                            .clear(
                                &self.client,
                                &self.config.airflow,
                                &self.config.airflow.username,
                                &self.config.airflow.password,
                                &self.config.airflow.host,
                            )
                            .await?;
                        }

                        if self.observable_mode.get() == Mode::Task
                            && self.table_dag_runs.table_state.selected().is_some()
                        {
                            self.table_dag_runs.tasks.as_mut().unwrap().task_instances
                                [self.table_dag_runs.table_tasks_state.selected().unwrap()]
                            .clear(
                                &self.client,
                                &self.config.airflow,
                                &self.config.airflow.username,
                                &self.config.airflow.password,
                                &self.config.airflow.host,
                            )
                            .await?;
                        }
                    }
                    Action::Task => {
                        self.status_bar.mode_breadcrumb.clear();
                        self.status_bar.mode_breadcrumb.push(Mode::DagRun);
                        if self.table_dag_runs.table_state.selected().is_none() {
                            self.observable_mode.set_mode(Mode::DagRun);
                            self.observable_mode.set_mode(Mode::DagRun);
                            break;
                        }
                        self.observable_mode.set_mode(Mode::Task);
                        self.table_dag_runs.table_tasks_state.select(Some(0));
                        self.table_dag_runs.tasks = Some(
                            self.dag_runs
                                .get_task(
                                    &self.client,
                                    &self.config.airflow,
                                    &self.config.airflow.username,
                                    &self.config.airflow.password,
                                    &self.config.airflow.host,
                                    self.table_dag_runs.table_state.selected().unwrap_or(0),
                                )
                                .await?,
                        );
                        self.status_bar.register_mode(self.observable_mode.get());
                    }
                    Action::Log => {
                        self.status_bar.mode_breadcrumb.clear();
                        self.status_bar.mode_breadcrumb.push(Mode::DagRun);
                        self.status_bar.mode_breadcrumb.push(Mode::Task);
                        self.observable_mode.set_mode(Mode::Log);
                        self.table_dag_runs
                            .handle_mode(self.observable_mode.get())?;
                        self.status_bar.register_mode(self.observable_mode.get());
                        if self.table_dag_runs.table_state.selected().is_some() {
                            let log = self.table_dag_runs.tasks.as_mut().unwrap().task_instances
                                [self.table_dag_runs.table_tasks_state.selected().unwrap()]
                            .get_logs(
                                &self.client,
                                &self.config.airflow,
                                &self.config.airflow.username,
                                &self.config.airflow.password,
                                &self.config.airflow.host,
                                self.table_dag_runs.try_number,
                            )
                            .await?;
                            self.table_dag_runs.log = log;
                        };
                    }
                    Action::NextTryNumber => {
                        if self.table_dag_runs.table_tasks_state.selected().is_some()
                            && self.observable_mode.get() == Mode::Log
                            && self.table_dag_runs.tasks.as_ref().unwrap().task_instances
                                [self.table_dag_runs.table_tasks_state.selected().unwrap()]
                            .try_number as usize
                                > self.table_dag_runs.try_number
                        {
                            self.table_dag_runs.try_number += 1;
                            log::info!("{}", format!("{}", self.table_dag_runs.try_number));
                            let log = self.table_dag_runs.tasks.as_mut().unwrap().task_instances
                                [self.table_dag_runs.table_tasks_state.selected().unwrap()]
                            .get_logs(
                                &self.client,
                                &self.config.airflow,
                                &self.config.airflow.username,
                                &self.config.airflow.password,
                                &self.config.airflow.host,
                                self.table_dag_runs.try_number,
                            )
                            .await?;
                            self.table_dag_runs.log = log;
                        }
                    }
                    Action::PreviousTryNumber => {
                        if self.table_dag_runs.table_tasks_state.selected().is_some()
                            && self.observable_mode.get() == Mode::Log
                            && self.table_dag_runs.try_number > 1
                        {
                            self.table_dag_runs.try_number -= 1;
                            log::info!("{}", format!("{}", self.table_dag_runs.try_number));
                            let log = self.table_dag_runs.tasks.as_mut().unwrap().task_instances
                                [self.table_dag_runs.table_tasks_state.selected().unwrap()]
                            .get_logs(
                                &self.client,
                                &self.config.airflow,
                                &self.config.airflow.username,
                                &self.config.airflow.password,
                                &self.config.airflow.host,
                                self.table_dag_runs.try_number,
                            )
                            .await?;
                            self.table_dag_runs.log = log;
                        }
                    }
                    Action::ClearSearch => {
                        self.table_dag_runs.user_search = None;
                    }
                    _ => {}
                }
                if let Some(action) = self.context_information.update(action.clone())? {
                    action_tx.send(action)?
                };
                self.context_information
                    .register_context_information(&self.dag_runs);

                if let Some(action) = self.shortcut.update(action.clone())? {
                    action_tx.send(action)?
                };
                self.shortcut.register_mode(self.observable_mode.get());

                if let Some(action) = self.ascii.update(action.clone())? {
                    action_tx.send(action)?
                };

                if let Some(action) = self.command_search.update(action.clone())? {
                    action_tx.send(action)?
                };
                self.command_search
                    .handle_mode(self.observable_mode.get())?;

                if let Some(action) = self.command.update(action.clone())? {
                    action_tx.send(action)?
                };
                self.command.handle_mode(self.observable_mode.get())?;

                if let Some(action) = self.table_dag_runs.update(action.clone())? {
                    action_tx.send(action)?
                };
                self.table_dag_runs
                    .handle_mode(self.observable_mode.get())?;
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
