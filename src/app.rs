use crate::components::table::table::LinkedTable;
use crate::components::table::table::Tables;
use crossterm::event::KeyCode::Char;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::components::table::pool::Pool;
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
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::components::ascii::Ascii;
use crate::components::command::Command;
use crate::components::command_search::CommandSearch;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::status_bar::StatusBar;
use crate::components::table::dag_run::DagRun as TableDagRun;
use crate::components::table::main::MainTable;
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
    client: Client,
    linked_components: LinkedComponent,
    tables: Tables,
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

        let mut tables = Tables::new();

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
            client,
            linked_components,
            tables,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let (data_tx, mut data_rx) = mpsc::unbounded_channel::<String>();
        let mut tui = tui::Tui::new()?;
        // tui.mouse(true);
        tui.enter()?;

        let context_data_ref = Arc::clone(&self.context_data);
        let airflow_config = self.config.airflow.clone();
        let mode = self.observable_mode.get().clone();
        tokio::spawn(async move {
            let mut lock = context_data_ref.lock().await;
            lock.refresh(mode).await;
        });

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

        let context_data_ref = Arc::clone(&self.context_data);
        let airflow_config = self.config.airflow.clone();
        let mode = self.observable_mode.get().clone();
        tokio::spawn(async move {
            let mut lock = context_data_ref.lock().await;
            lock.refresh(mode).await;
        });

        tokio::spawn(async move {
            data_tx.send(String::from("TEST"));
        });

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => match key.code {
                        Char('q') => self.should_quit = true,
                        _ => {}
                    },
                    tui::Event::Refresh => {
                        let context_data_ref = Arc::clone(&self.context_data);
                        let airflow_config = self.config.airflow.clone();
                        let mode = self.observable_mode.get().clone();
                        let test: String = tokio::spawn(async move {
                            let mut lock = context_data_ref.lock().await;
                            let test = lock.refresh(mode).await;
                        })
                        .await
                        .unwrap();
                        panic!("{} test);
                    }
                    _ => {}
                }
                let context_data_ref = Arc::clone(&self.context_data);
                let mut lock = context_data_ref.lock().await;
                match self.linked_components.handle_events(
                    Some(&e),
                    &mut lock,
                    &self.observable_mode,
                    &mut self.tables,
                )? {
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
                        let context_data_ref = Arc::clone(&self.context_data);
                        let context_data_lock = context_data_ref.lock().await;
                        let table_ref = Arc::clone(
                            &self
                                .tables
                                .get_table_by_mode(self.observable_mode.get())
                                .unwrap(),
                        );
                        let table = table_ref.lock().await;
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                    &context_data_lock,
                                    &table,
                                )
                                .expect("Failed to draw components");
                        })?;
                    }
                    Action::Render => {
                        let context_data_ref = Arc::clone(&self.context_data);
                        let context_data_lock = context_data_ref.lock().await;
                        let table_ref = Arc::clone(
                            &self
                                .tables
                                .get_table_by_mode(self.observable_mode.get())
                                .unwrap(),
                        );
                        let table = table_ref.lock().await;
                        tui.draw(|f| {
                            self.linked_components
                                .draw_components(
                                    f,
                                    |chunk| self.main_layout.borrow().get_chunk(chunk),
                                    self.observable_mode.get(),
                                    &context_data_lock,
                                    &table,
                                )
                                .expect("Failed to draw components");
                        })?;
                    }
                    Action::Mode(mode) => {
                        self.observable_mode.set_mode(mode);
                    }
                    Action::NextMode(current_mode) => {
                        self.observable_mode.set_next_mode(current_mode);
                        let context_data_ref = Arc::clone(&self.context_data);
                        let airflow_config = self.config.airflow.clone();
                        let mode = self.observable_mode.get().clone();
                        tokio::spawn(async move {
                            let mut lock = context_data_ref.lock().await;
                            lock.refresh(mode).await;
                        });
                    }
                    Action::PreviousMode(mode) => {
                        self.observable_mode.set_mode(mode);
                    }
                    Action::Pool => {
                        self.observable_mode.set_mode(Mode::Pool);
                    }
                    _ => {}
                }

                let context_data_ref = Arc::clone(&self.context_data);
                let mut lock = context_data_ref.lock().await;
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
