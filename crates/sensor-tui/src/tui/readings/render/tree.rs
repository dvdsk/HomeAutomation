use indextree::Arena;
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

pub(crate) fn build_ui<'a>(readings: &'a Readings) -> Vec<TreeItem<'a, u16>> {
    let mut id_gen = IdGen::default();

    readings
        .root
        .children(&readings.arena)
        .map(
            |subroot| match readings.arena.get(subroot).expect("child exists").get() {
                Node::Sensor(info) => new_leaf(info),
                Node::Branch(name) => TreeItem::new(
                    id_gen.next(),
                    name.as_str(),
                    recusive_call(subroot, &readings.arena, &mut id_gen),
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
) -> Vec<TreeItem<'a, u16>> {
    root.children(arena)
        .map(|child_id| {
            let child_node = arena.get(child_id).expect("child exists").get();
            match child_node {
                Node::Sensor(info) => new_leaf(info),
                Node::Branch(name) => TreeItem::new(
                    id_gen.next(),
                    name.as_str(),
                    recusive_call(root, arena, id_gen),
                )
                .expect("no duplicate ids"),
                Node::Root => unreachable!("Root is not a descendant"),
            }
        })
        .collect()
}

fn new_leaf(info: &SensorInfo) -> TreeItem<u16> {
    use protocol::reading::tree::Tree as _;

    let text = format!(
        "{0}: {1:.2$} {3}",
        info.reading.name(),
        info.info.val,
        info.info.precision(),
        info.info.unit
    );
    TreeItem::new_leaf(info.id, text)
}
