use crate::components::table::pool::Pool as TablePool;
use crate::components::table::task::Task as TableTask;
use crate::mode::Mode;
use crate::mode::Mode::Pool;
use crate::mode::Mode::Task;
use core::fmt::Debug;
use ratatui::prelude::Style;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard};
use tokio::sync::Mutex;

pub trait Table {
    fn get_columns(&self) -> Vec<&'static str>;
    fn get_border_style(&self) -> Style;
}

pub type TableNodeComponentRef = Option<Arc<Mutex<TableNodeComponent>>>;

pub struct TableNodeComponent {
    pub value: Arc<Mutex<dyn Table>>,
    pub next: TableNodeComponentRef,
    pub previous: TableNodeComponentRef,
}

impl TableNodeComponent {
    pub fn new(component: Arc<Mutex<dyn Table>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(TableNodeComponent {
            value: component,
            next: None,
            previous: None,
        }))
    }
}

pub struct LinkedTable {
    pub head: TableNodeComponentRef,
}

impl LinkedTable {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub async fn append(&mut self, table: Arc<Mutex<dyn Table>>) {
        let new_node = TableNodeComponent::new(table);

        match &self.head {
            Some(head) => {
                let mut current = Arc::clone(head);

                loop {
                    let current_ref = current.lock().await;

                    match &current_ref.next {
                        Some(next) => {
                            let current = Arc::clone(next);
                        }
                        None => {
                            break;
                        }
                    }
                }

                let mut current_mut = current.lock().await;
                current_mut.next = Some(Arc::clone(&new_node));
                new_node.lock().await.previous = Some(Arc::clone(&current));
            }
            None => {
                self.head = Some(Arc::clone(&new_node));
            }
        }
    }

    pub async fn next(&mut self) {
        if let Some(head) = self.head.take() {
            if let Some(next) = head.lock().await.next.clone() {
                self.head = Some(next);
            }
        }
    }
}

pub struct Tables {
    tables: [(Mode, Arc<Mutex<dyn Table>>); 2],
}

impl Tables {
    pub fn new() -> Tables {
        Tables {
            tables: [
                (Pool, Arc::new(Mutex::new(TablePool::default()))),
                (Task, Arc::new(Mutex::new(TableTask::default()))),
            ],
        }
    }

    pub fn get_table_by_mode(&self, mode: Mode) -> Option<&Arc<Mutex<dyn Table>>> {
        for (table_mode, table) in &self.tables {
            if table_mode == &mode {
                return Some(table);
            }
        }
        None
    }
}
