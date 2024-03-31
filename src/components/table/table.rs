use ratatui::prelude::Style;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Table {
    fn get_columns(&self) -> Vec<&'static str>;
    fn get_border_style(&self) -> Style;
}

pub type TableNodeComponentRef = Option<Rc<RefCell<TableNodeComponent>>>;

pub struct TableNodeComponent {
    pub value: Rc<RefCell<dyn Table>>,
    pub next: TableNodeComponentRef,
    pub previous: TableNodeComponentRef,
}

impl TableNodeComponent {
    pub fn new(component: Rc<RefCell<dyn Table>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TableNodeComponent {
            value: component,
            next: None,
            previous: None,
        }))
    }
}

#[derive(Default)]
pub struct LinkedTable {
    pub head: TableNodeComponentRef,
}

impl LinkedTable {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn append(&mut self, table: Rc<RefCell<dyn Table>>) {
        let new_node = TableNodeComponent::new(table);
        match &self.head {
            Some(head) => {
                let mut current = head.clone();
                loop {
                    match &current.borrow().next {
                        Some(next) => {
                            let current = next.clone();
                        }
                        None => {
                            current.borrow_mut().next = Some(new_node.clone());
                            new_node.borrow_mut().previous = Some(current.clone());
                            break;
                        }
                    }
                }
            }
            None => {
                self.head = Some(new_node.clone());
            }
        }
    }
}
