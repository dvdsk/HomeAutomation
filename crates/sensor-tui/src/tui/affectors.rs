use std::collections::{HashMap, HashSet};

use crossterm::event::{KeyCode, KeyEvent};
use protocol::{affector, Device};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_tree_widget::Tree;
use tui_tree_widget::{TreeItem, TreeState};

use crate::Update;

use super::Theme;
use protocol::affector::tree::Item;
use protocol::affector::tree::Tree as AffectorTree;

pub type TreeKey = [u8; 6];

struct AffectorState;

#[derive(Default)]
pub struct Tab {
    tree_state: TreeState<TreeKey>,
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    data: HashMap<TreeKey, AffectorState>,
}

impl Tab {
    pub fn render(&mut self, frame: &mut Frame, layout: Rect, theme: &Theme) {
        frame.render_stateful_widget(
            Tree::new(&self.ground)
                .expect("all item identifiers should be unique")
                .block(
                    Block::default()
                        .title("Controllable affectors")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>"),
            layout,
            &mut self.tree_state,
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<KeyEvent> {
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
            _ => return Some(key),
        }
        None
    }

    pub fn process_update(&mut self, update: &Update) {
        let devices = match update {
            Update::ReadingList(_)
            | Update::Fetched { .. }
            | Update::FetchError(_)
            | Update::SubscribeError(_) => &Vec::new(),
            Update::SensorReading(r) => &vec![r.device()],
            Update::SensorError(err) => &vec![err.device()],
            Update::DeviceList(devices) => devices,
        };

        let affectors: HashSet<_> = devices
            .iter()
            .map(Device::info)
            .flat_map(|info| info.affectors)
            .collect();

        for affector in affectors {
            self.update_tree(affector);
        }

        if self.tree_state.selected().is_empty() {
            self.tree_state.select_first();
        }
    }
}

impl Tab {
    fn update_tree(&mut self, affector: &protocol::Affector) {
        let key = tree_key(affector);

        let mut tree = add_root(affector as &dyn AffectorTree, &mut self.ground);
        let mut tomato = match affector.inner() {
            Item::Leaf(_) => unreachable!("no values at level 0"),
            Item::Node(inner) => inner,
        };
        loop {
            match tomato.inner() {
                Item::Leaf(_) => {
                    let text = tomato.name();
                    add_leaf(text, tree, key);
                    return;
                }
                Item::Node(inner) => {
                    tree = add_node(tomato, tree);
                    tomato = inner;
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
