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
use std::sync::Arc;
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
    pub context_data: Arc<tokio::sync::Mutex<ContextData>>,
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
            context_data: Arc::new(tokio::sync::Mutex::new(ContextData::new())),
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

        // Register airflow config for context_data
        let context_data_ref = Arc::clone(&self.context_data);
        let airflow_config = self.config.airflow.clone();
        tokio::spawn(async move {
            let mut lock = context_data_ref.lock().await;
            lock.handle_airflow_config(airflow_config);
        });

        loop {
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
                    tui::Event::Refresh => {
                        let context_data_ref = Arc::clone(&self.context_data);
                        let airflow_config = self.config.airflow.clone();
                        let mode = self.observable_mode.get().clone();
                        tokio::spawn(async move {
                            let mut lock = context_data_ref.lock().await;
                            lock.refresh(mode).await;
                        });
                    }
                    _ => {}
                }
                let context_data_ref = Arc::clone(&self.context_data);
                let mut lock = context_data_ref.lock().await;
                match self.linked_components.handle_events(Some(&e), &lock, self.observable_mode.get())? {
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
                        let context_data_ref = Arc::clone(&self.context_data);
                        let context_data_lock = context_data_ref.lock().await;
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                    &context_data_lock,
                                )
                                .expect("Failed to draw components");
                        })?;
                    }
                    Action::Render => {
                        let context_data_ref = Arc::clone(&self.context_data);
                        let context_data_lock = context_data_ref.lock().await;
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                    &context_data_lock,
                                )
                                .expect("Failed to draw components");
                        })?;
                    }
                    Action::Pool => {
                        self.observable_mode.set_mode(Mode::Pool);
                    }
                    _ => {}
                }

                let context_data_ref = Arc::clone(&self.context_data);
                let mut lock = context_data_ref.lock().await;
                self.linked_components.handle_actions(action.clone(), &lock);
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
