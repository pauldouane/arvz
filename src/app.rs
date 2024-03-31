use std::cell::RefCell;
use std::ops::Deref;
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
use crate::components::table::main::MainTable;
use crate::components::table_dag_runs::TableDagRuns;
use crate::components::LinkedComponent;
use crate::context_data::ContextData;
use crate::main_layout::MainLayout;
use crate::mode::Mode::Search;
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::DagRuns;
use crate::models::pool::PoolCollection;
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
    pub context_data: Rc<RefCell<ContextData>>,
    pub dag_runs: DagRuns,
    client: Client,
    context_information: ContextInformation,
    shortcut: Shortcut,
    ascii: Ascii,
    table_dag_runs: TableDagRuns,
    status_bar: StatusBar,
    command_search: CommandSearch,
    command: Command,
    linked_components: LinkedComponent,
}

impl App {
    pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let mode = Mode::DagRun;
        let client = Client::new();

        let mut linked_components = LinkedComponent::new();
        linked_components.append(
            Rc::new(RefCell::new(ContextInformation::new())),
            Chunk::Context(0),
            None,
        );
        linked_components.append(
            Rc::new(RefCell::new(Shortcut::new())),
            Chunk::Context(1),
            None,
        );
        linked_components.append(Rc::new(RefCell::new(Ascii::new())), Chunk::Context(2), None);
        linked_components.append(
            Rc::new(RefCell::new(Command::new())),
            Chunk::CommandChunk,
            Some(Mode::Command),
        );
        linked_components.append(
            Rc::new(RefCell::new(CommandSearch::new())),
            Chunk::CommandChunk,
            Some(Mode::Search),
        );
        linked_components.append(Rc::new(RefCell::new(MainTable::new())), Chunk::Table, None);
        linked_components.append(Rc::new(RefCell::new(StatusBar::new())), Chunk::Status, None);
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
            context_data: Rc::new(RefCell::new(ContextData::new())),
            dag_runs: DagRuns::new(),
            client,
            context_information: ContextInformation::new(),
            shortcut: Shortcut::new(),
            ascii: Ascii::new(),
            table_dag_runs: TableDagRuns::new(),
            status_bar: StatusBar::new(),
            command_search: CommandSearch::new(),
            command: Command::new(),
            linked_components,
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

        // Init the area for all components
        self.context_information.init(tui.size()?)?;
        self.shortcut.init(tui.size()?)?;
        self.ascii.init(tui.size()?)?;
        self.table_dag_runs.init(tui.size()?)?;

        // Set tui_size for the main_layout
        self.main_layout
            .borrow_mut()
            .set_tui_size(Rc::new(RefCell::new(tui.size().unwrap())));

        // Set the default main layout
        self.main_layout
            .borrow_mut()
            .set_main_layout(&self.observable_mode.get());

        // Register the main layout refresh function for the observable mode
        let main_layout_rc = Rc::clone(&self.main_layout);
        self.observable_mode
            .set_refresh_layout_fn(move |mode| main_layout_rc.borrow_mut().set_main_layout(&mode));

        // Register configs for components
        self.linked_components
            .register_config_components(&self.config);

        // Register action handlers for components
        self.linked_components
            .register_action_components(&action_tx);

        self.context_data
            .borrow_mut()
            .handle_airflow_config(self.config.airflow.clone());

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
                    tui::Event::Refresh => match self.observable_mode.get() {
                        Mode::Pool => self.context_data.borrow_mut().refresh(Mode::Pool).await,
                        _ => {}
                    },
                    _ => {}
                }
                match self.linked_components.handle_events(Some(&e))? {
                    Some(action) => action_tx.send(action)?,
                    None => {}
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
                        self.main_layout
                            .borrow_mut()
                            .set_tui_size(Rc::new(RefCell::new(tui.size().unwrap())));
                        self.main_layout
                            .borrow_mut()
                            .set_main_layout(&self.observable_mode.get());
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                )
                                .expect("Failed to draw components");
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                )
                                .expect("Failed to draw components");
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
                    Action::Pool => {
                        self.observable_mode.set_mode(Mode::Pool);
                    }
                    _ => {}
                }
                if let Some(action) = self.context_information.update(action.clone())? {
                    action_tx.send(action)?
                };

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
