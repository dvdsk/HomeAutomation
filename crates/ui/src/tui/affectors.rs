use std::collections::HashMap;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use protocol::{affector, Affector};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use tokio::sync::mpsc;
use tui_tree_widget::{TreeItem, TreeState};

use crate::control;
use crate::Update;

use super::Theme;
use protocol::affector::tree::Item;
use protocol::affector::tree::Tree as AffectorTree;

mod handle_key;
mod render;

pub type TreeKey = [u8; 6];

#[derive(Debug)]
struct AffectorState {
    affector: Affector,
    selected_control: usize,
    last_input: Option<Instant>,
    last_controlled_by: Option<String>,
    last_order_status: Option<control::AffectorStatus>,
    info: affector::Info,
    device_broken: DeviceBroken,
}

#[derive(Default)]
pub struct Tab {
    tree_state: TreeState<TreeKey>,
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    data: HashMap<TreeKey, AffectorState>,
}

impl Tab {
    pub fn render(&mut self, frame: &mut Frame, layout: Rect, theme: &Theme) {
        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(1)])
                .areas(layout);
        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                .areas(main);

        render::tree(frame, left, &self.ground, &mut self.tree_state);

        let mut data = self
            .tree_state
            .selected()
            .last() // unique leaf id
            .and_then(|key| self.data.get_mut(key));

        if let Some(ref mut data) = data {
            let [top, bottom] =
                Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)])
                    .areas(right);
            render::details(frame, data, top);
            render::controls(frame, data, bottom);
        };
        render::footer(frame, footer, data, theme)
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        control: &mut mpsc::Sender<protocol::Affector>,
    ) -> Option<KeyEvent> {
        let data = self
            .tree_state
            .selected()
            .last() // unique leaf id
            .and_then(|key| self.data.get_mut(key));

        match key.code {
            KeyCode::Down => {
                self.tree_state.key_down();
            }
            KeyCode::Up => {
                self.tree_state.key_up();
            }
            KeyCode::Enter => {
                self.tree_state.toggle_selected();
            }
            _ => {
                if let Some(state) = data {
                    handle_key::handle(key, state, control)?;
                }
                return Some(key);
            }
        }
        None
    }

    pub fn process_update(&mut self, update: Update) -> Option<Update> {
        let update = match update {
            Update::ReadingList(_)
            | Update::Fetched { .. }
            | Update::FetchError(_)
            | Update::SensorReading(_)
            | Update::DeviceList(_)
            | Update::PopulateError(_)
            | Update::SubscribeError(_) => Some(update),
            Update::AffectorControlled {
                affector,
                controlled_by,
            } => {
                self.mark_controlled(&affector, controlled_by);
                None
            }
            Update::SensorError(ref err) => {
                let broken = err.device().info().affectors;
                for affector in broken {
                    self.mark_broken(affector);
                }
                Some(update)
            }
            Update::AffectorList(affectors) => {
                for affector in affectors {
                    self.add(&affector);
                }
                None
            }
            Update::AffectorOrderStatus { affector, status } => {
                self.update_order_status(&affector, status);
                None
            }
        };

        if self.tree_state.selected().is_empty() {
            self.tree_state.select_first();
        }

        update
    }
}

#[derive(Debug)]
enum DeviceBroken {
    Yes,
    No,
}

impl Tab {
    fn mark_controlled(
        &mut self,
        affector: &protocol::Affector,
        controlled_by: String,
    ) {
        self.update_tree(affector, move |state| {
            state.last_controlled_by = Some(controlled_by);
        })
    }

    fn mark_broken(&mut self, affector: &protocol::Affector) {
        self.update_tree(affector, move |state| {
            state.device_broken = DeviceBroken::Yes;
        })
    }

    fn add(&mut self, affector: &protocol::Affector) {
        self.update_tree(affector, move |state| {
            state.affector = *affector;
        })
    }

    fn update_order_status(
        &mut self,
        affector: &protocol::Affector,
        status: control::AffectorStatus,
    ) {
        self.update_tree(affector, move |state| {
            state.last_order_status = Some(status);
        })
    }

    fn update_tree(
        &mut self,
        affector: &protocol::Affector,
        update: impl FnOnce(&mut AffectorState) -> (),
    ) {
        let key = tree_key(affector);

        let mut tree =
            add_root(affector as &dyn AffectorTree, &mut self.ground);
        let mut tree_node = match affector.inner() {
            Item::Leaf(_) => unreachable!("no values at level 0"),
            Item::Node(inner) => inner,
        };
        loop {
            match tree_node.inner() {
                Item::Leaf(info) => {
                    let text = tree_node.name();
                    add_leaf(text, tree, key);
                    let item = self.data.entry(key).or_insert(AffectorState {
                        affector: *affector,
                        info,
                        selected_control: 0,
                        last_controlled_by: None,
                        device_broken: DeviceBroken::No,
                        last_input: None,
                        last_order_status: None,
                    });
                    update(item);
                    return;
                }
                Item::Node(inner) => {
                    tree = add_node(tree_node, tree);
                    tree_node = inner;
                }
            };
        }
    }
}

fn add_leaf(text: String, tree: &mut TreeItem<'static, TreeKey>, key: TreeKey) {
    let new_item = TreeItem::new_leaf(key, text.clone());
    // todo is exists its fine handle that
    let _ignore_existing = tree.add_child(new_item); // errors when identifier already exists

    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    let existing = tree.child_mut(new_child).expect("just added it");
    existing.update_text(text);
}

fn add_root<'a>(
    tomato: &dyn AffectorTree,
    ground: &'a mut Vec<TreeItem<'static, TreeKey>>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.branch_id(); 6];
    let exists = ground.iter().any(|item| *item.identifier() == key);
    if !exists {
        let new_root = TreeItem::new(key, tomato.name(), vec![]).unwrap();
        ground.push(new_root);
    }

    ground
        .iter_mut()
        .find(|item| *item.identifier() == key)
        .expect("checked and added if missing")
}

fn add_node<'a>(
    tomato: &dyn AffectorTree,
    tree: &'a mut TreeItem<'static, TreeKey>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.branch_id(); 6];
    let new_item = TreeItem::new(key, tomato.name(), Vec::new()).unwrap();
    // add just in case it was not there yet
    let _ignore_existing = tree.add_child(new_item);
    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    tree.child_mut(new_child).expect("just added it")
}

pub(crate) fn tree_key(affector: &protocol::Affector) -> TreeKey {
    let mut key = [0u8; 6];
    key[0] = affector.branch_id();

    let mut reading = affector as &dyn AffectorTree;
    for byte in &mut key[1..] {
        reading = match reading.inner() {
            Item::Node(inner) => {
                *byte = inner.branch_id();
                inner
            }
            Item::Leaf(affector::Info { .. }) => {
                return key;
            }
        };
    }
    unreachable!("reading should not be deeper then key size")
}
