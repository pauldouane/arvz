use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  action::Action,
  components::{fps::FpsCounter, Component},
  config::Config,
  mode::Mode,
  tui,
};
use crate::components::ascii::Ascii;
use crate::components::context_informations::ContextInformation;
use crate::components::shortcut::Shortcut;
use crate::components::table_dags_runs::{TableDagRuns};
use crate::models::dag_run::DagRun;
use crate::models::dag_runs::{DagRuns};

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub dag_runs: DagRuns,
  client: Client,
  context_information: ContextInformation,
  shortcut: Shortcut,
  ascii: Ascii,
  table_dag_runs: TableDagRuns,
}

impl App {
  pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let fps = FpsCounter::default();
    let config = Config::new()?;
    let mode = Mode::Home;
    let client = Client::new();

    let user_name = "root".to_string();
    let password: Option<String> = Some("root".to_string());

    let dag_runs: DagRuns = client
        .get("http://172.24.164.162:8080/api/v1/dags/~/dagRuns")
        .basic_auth(user_name, password)
        .send()
        .await?
        .json::<DagRuns>()
        .await?;


    Ok(Self {
      tick_rate,
      frame_rate,
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      dag_runs,
      client,
      context_information: ContextInformation::new(),
      shortcut: Shortcut::new(),
      ascii: Ascii::new(),
      table_dag_runs: TableDagRuns::new(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?;
    // tui.mouse(true);
    tui.enter()?;

    self.context_information.register_action_handler(action_tx.clone())?;
    self.context_information.register_config_handler(self.config.clone())?;

    self.shortcut.register_action_handler(action_tx.clone())?;
    self.shortcut.register_config_handler(self.config.clone())?;

    self.ascii.register_action_handler(action_tx.clone())?;
    self.ascii.register_config_handler(self.config.clone())?;

    self.table_dag_runs.register_action_handler(action_tx.clone())?;
    self.table_dag_runs.register_config_handler(self.config.clone())?;

    self.context_information.init(tui.size()?)?;
    self.shortcut.init(tui.size()?)?;
    self.ascii.init(tui.size()?)?;
    self.table_dag_runs.init(tui.size()?)?;

    loop {
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
          },
          _ => {},
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
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
                let r = self.context_information.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.shortcut.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.ascii.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.table_dag_runs.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              let main_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                  Constraint::Percentage(25),
                  Constraint::Percentage(70),
                  Constraint::Percentage(5),
                ])
                .split(f.size());
              let top_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                  Constraint::Percentage(40),
                  Constraint::Percentage(40),
                  Constraint::Percentage(20),
                ])
                .split(main_chunk[0]);
              let center_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                  Constraint::Percentage(100),
                ])
                .split(main_chunk[1]);

                let r = self.context_information.draw(f, top_chunk[0]);
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.shortcut.draw(f, top_chunk[1]);
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.ascii.draw(f, top_chunk[2]);
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }

                let r = self.table_dag_runs.draw(f, center_chunk[0]);
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
            })?;
          },
          _ => {},
        }
        if let Some(action) = self.context_information.update(action.clone())? {
          action_tx.send(action)?
        };

        if let Some(action) = self.shortcut.update(action.clone())? {
          action_tx.send(action)?
        };
        self.shortcut.register_mode(self.mode);

        if let Some(action) = self.ascii.update(action.clone())? {
          action_tx.send(action)?
        };

        if let Some(action) = self.table_dag_runs.update(action.clone())? {
          action_tx.send(action)?
        };
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
