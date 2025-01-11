use std::collections::HashSet;

use indextree::Arena;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use tui_tree_widget::TreeItem;

use crate::tui::readings::sensor_info::{Node, Readings, SensorInfo};

#[derive(Default)]
struct IdGen(u16);

impl IdGen {
    fn next(&mut self) -> u16 {
        self.0 += 1;
        assert!(self.0 < 10_000, "to many id's generated");
        self.0
    }
}

pub(crate) fn build_ui<'a>(
    readings: &'a Readings,
    comparing: &HashSet<u16>,
) -> Vec<TreeItem<'a, u16>> {
    let mut id_gen = IdGen::default();

    readings
        .root
        .children(&readings.arena)
        .map(
            |subroot| match readings.arena.get(subroot).expect("child exists").get() {
                Node::Sensor(info) => new_leaf(info, comparing),
                Node::Branch(name) => TreeItem::new(
                    id_gen.next(),
                    name.as_str(),
                    recusive_call(subroot, &readings.arena, &mut id_gen, comparing),
                )
                .expect("no duplicate ids"),
                Node::Root => unreachable!("Root is not a descendant"),
            },
        )
        .collect()
}

fn recusive_call<'a>(
    root: indextree::NodeId,
    arena: &'a Arena<Node>,
    id_gen: &mut IdGen,
    comparing: &HashSet<u16>,
) -> Vec<TreeItem<'a, u16>> {
    root.children(arena)
        .map(|child_id| {
            let child_node = arena.get(child_id).expect("child exists").get();
            match child_node {
                Node::Sensor(info) => new_leaf(info, comparing),
                Node::Branch(name) => TreeItem::new(
                    id_gen.next(),
                    name.as_str(),
                    recusive_call(child_id, arena, id_gen, comparing),
                )
                .expect("no duplicate ids"),
                Node::Root => unreachable!("Root is not a descendant"),
            }
        })
        .collect()
}

fn new_leaf<'a>(info: &'a SensorInfo, comparing: &HashSet<u16>) -> TreeItem<'a, u16> {
    use protocol::reading::tree::{Item, Tree};

    let mut node = &info.reading as &dyn Tree;
    while let Item::Node(inner) = node.inner() {
        node = inner;
    }

    let style = if comparing.contains(&info.ui_id) {
        Style::new().italic().blue()
    } else {
        Style::new()
    };

    let text = if info.is_placeholder {
        node.name()
    } else {
        format!(
            "{0}: {1:.2$} {3}",
            node.name(),
            info.info.val,
            info.info.precision(),
            info.info.unit
        )
    };

    TreeItem::new_leaf(info.ui_id, Text::styled(text, style))
}
