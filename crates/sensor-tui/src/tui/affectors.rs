use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use tui_tree_widget::TreeState;

use crate::Update;

use super::{ShouldExit, Theme};

pub type TreeKey = [u8; 6];

pub struct Tab {
    affector_tree_state: TreeState<TreeKey>,
    // affectors
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            affector_tree_state: Default::default(),
        }
    }
}

impl Tab {
    pub fn render(&mut self, _frame: &mut Frame, _layout: Rect, _theme: &Theme) {
        // frame.render_stateful_widget(
        //     Tree::new(affectors)
        //         .expect("all item identifiers should be unique")
        //         .block(
        //             Block::default()
        //                 .title("Controllable affectors")
        //                 .borders(Borders::ALL),
        //         )
        //         .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        //         .highlight_symbol(">>"),
        //     layout,
        //     &mut app.affector_tree_state,
        // );
    }

    pub fn handle_key(&mut self, _key: KeyEvent) -> ShouldExit {
        todo!()
    }

    pub fn process_update(&mut self, _update: Update) {
        todo!()
    }
}
