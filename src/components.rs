use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::rc::Rc;

use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::main_layout::{Chunk, MainLayout};
use crate::mode::Mode;
use crate::{
    action::Action,
    config::Config,
    tui::{Event, Frame},
};

pub mod ascii;
pub mod command;
pub mod command_search;
pub mod context_informations;
pub mod fps;
pub mod shortcut;
pub mod status_bar;
pub mod table;
pub mod table_dag_runs;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to receive events,
/// update state, and be rendered on the screen.
pub trait Component {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused_variables)]
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        Ok(())
    }
    /// Initialize the component with a specified area if necessary.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn init(&mut self, area: Rect) -> Result<()> {
        Ok(())
    }
    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_events(&mut self, event: Option<&Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }
    /// Handle mode changes and update the state of the component if necessary.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode to be handled.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused_variables)]
    fn handle_mode(&mut self, mode: Mode) -> Result<()> {
        Ok(())
    }
    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: &KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: &MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}

impl Debug for dyn Component {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type NodeComponentRef = Option<Rc<RefCell<NodeComponent>>>;

#[derive(Debug)]
pub struct NodeComponent {
    pub value: Rc<RefCell<dyn Component>>,
    pub next: NodeComponentRef,
    pub chunk: Chunk,
    pub display_mode: Option<Mode>,
}

impl NodeComponent {
    pub fn new(
        component: Rc<RefCell<dyn Component>>,
        chunk: Chunk,
        display_mode: Option<Mode>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(NodeComponent {
            value: component,
            next: None,
            chunk,
            display_mode,
        }))
    }
}

#[derive(Debug)]
pub struct LinkedComponent {
    pub head: NodeComponentRef,
}

impl LinkedComponent {
    pub fn new() -> Self {
        LinkedComponent { head: None }
    }

    pub fn append(
        &mut self,
        component: Rc<RefCell<dyn Component>>,
        chunk: Chunk,
        display_mode: Option<Mode>,
    ) {
        let new_node: Rc<RefCell<NodeComponent>> =
            NodeComponent::new(component, chunk, display_mode);
        match self.head {
            None => {
                self.head = Some(new_node);
            }
            Some(ref head) => {
                let mut current = head.clone();
                loop {
                    let next = { current.borrow().next.clone() };
                    match next {
                        Some(n) => current = n,
                        None => {
                            current.borrow_mut().next = Some(new_node.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_components<F>(&mut self, f: &mut Frame<'_>, get_chunk: F, mode: Mode) -> Result<()>
    where
        F: Fn(&Chunk) -> Rect,
    {
        let mut current: Option<Rc<RefCell<NodeComponent>>> = self.head.clone();
        while let Some(node) = current {
            let node: Ref<NodeComponent> = node.borrow();
            let chunk: &Chunk = &node.chunk;
            let display_mode: Option<Mode> = node.display_mode;
            let area: Rect = get_chunk(chunk);
            if let Some(observable_mode) = display_mode {
                if observable_mode != mode {
                    current = node.next.clone();
                    continue;
                }
            }
            node.value.borrow_mut().draw(f, area)?;
            current = node.next.clone();
        }
        Ok(())
    }

    pub fn register_action_components(&self, tx: &UnboundedSender<Action>) {
        let mut current: Option<Rc<RefCell<NodeComponent>>> = self.head.clone();
        while let Some(node) = current {
            let node: Ref<NodeComponent> = node.borrow();
            let mut component = node.value.borrow_mut();
            component
                .register_action_handler(tx.clone())
                .expect("TODO: panic message");
            current = node.next.clone();
        }
    }

    pub fn register_config_components(&self, config: &Config) {
        let mut current: Option<Rc<RefCell<NodeComponent>>> = self.head.clone();
        while let Some(node) = current {
            let node: Ref<NodeComponent> = node.borrow();
            let mut component = node.value.borrow_mut();
            component
                .register_config_handler(config.clone())
                .expect("TODO: panic message");
            current = node.next.clone();
        }
    }

    pub fn handle_events(&self, option: Option<&Event>) -> Result<Option<Action>> {
        let mut current: Option<Rc<RefCell<NodeComponent>>> = self.head.clone();
        while let Some(node) = current {
            let node: Ref<NodeComponent> = node.borrow();
            let mut component = node.value.borrow_mut();
            let action = component.handle_events(option)?;
            current = node.next.clone();
            if current.is_none() && action.is_some() {
                return Ok(action);
            }
        }
        Ok(None)
    }

    pub fn get_component_by_idx(&self, usr_idx: usize) -> Option<Rc<RefCell<dyn Component>>> {
        let mut current: Option<Rc<RefCell<NodeComponent>>> = self.head.clone();
        let mut idx: usize = 0;
        while let Some(node) = current {
            let node: Ref<NodeComponent> = node.borrow();
            if idx == usr_idx {
                return Some(node.value.clone());
            }
            current = node.next.clone();
        }
        None
    }

    pub fn iter(&self) -> IterComponent {
        IterComponent {
            current: self.head.clone(),
        }
    }
}

pub struct IterComponent {
    pub(crate) current: NodeComponentRef,
}

impl Iterator for IterComponent {
    type Item = Rc<RefCell<NodeComponent>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current: Option<Rc<RefCell<NodeComponent>>> = self.current.take();
        if current.is_none() {
            return None;
        } else {
            let current: Rc<RefCell<NodeComponent>> = current.unwrap();
            let current: RefMut<NodeComponent> = current.borrow_mut();
            self.current = current.next.clone();
            current.next.clone()
        }
    }
}
