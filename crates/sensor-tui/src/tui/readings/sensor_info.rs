use hdrhistogram::Histogram;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use jiff::Unit;
use log_store::api::{self, Percentile};
use protocol::reading;
use protocol::reading::tree::{Item, Tree};
use protocol::Reading;
use protocol::{Device, Error};

use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::time::Duration;
use tui_tree_widget::TreeItem;

use crate::Fetchable;

mod logs;
pub(crate) use logs::List as LogList;
pub(crate) use logs::LogSource;

#[derive(Debug)]
pub enum Node {
    Sensor(SensorInfo),
    Branch(String),
    Root,
}

#[derive(Debug)]
pub struct SensorInfo {
    pub id: u16,
    pub info: reading::Info,
    /// This value is not up to date, only use for requesting
    /// data use the last element of recent_history for printing
    pub reading: Reading,
    timing: Histogram<u64>,
    pub percentiles_from_store: Vec<Percentile>,
    recent_history: Vec<(jiff::Timestamp, f32)>,
    pub histogram_range: Option<RangeInclusive<jiff::Timestamp>>,
    pub history_from_store: Vec<(jiff::Timestamp, f32)>,
    condition: Result<(), Box<Error>>,
    pub(crate) logs: logs::Logs,
    pub placeholder: bool,
}

pub struct ErrorDensity {
    pub t5_min: f32,
    pub t15_min: f32,
    pub t30_min: f32,
    pub t45_min: f32,
    pub t60_min: f32,
}

impl ErrorDensity {
    fn from_log(log: &logs::Logs) -> Self {
        let buckets = [5, 15, 30, 45, 60].map(|min| jiff::Span::new().minutes(min));
        let counts = log.density(buckets);

        Self {
            t5_min: counts[0],
            t15_min: counts[1],
            t30_min: counts[2],
            t45_min: counts[3],
            t60_min: counts[4],
        }
    }
}

pub struct Details {
    pub last_reading: Option<(jiff::Timestamp, String)>,
    pub condition: Result<(), Box<Error>>,
    pub description: String,
    pub errors_since: ErrorDensity,
}

impl SensorInfo {
    fn last_at(&self) -> Option<jiff::Timestamp> {
        self.condition.clone().ok();
        self.recent_history.last().map(|(ts, _)| ts).copied()
    }

    pub fn details(&self) -> Details {
        let last_reading = self.recent_history.last().copied().map(|(ts, val)| {
            let val = format!("{0:.1$} {2}", val, self.info.precision(), self.info.unit);
            (ts, val)
        });
        Details {
            last_reading,
            condition: self.condition.clone(),
            description: self.info.description.to_owned(),
            errors_since: ErrorDensity::from_log(&self.logs),
        }
    }

    pub fn percentiles(&self) -> Vec<api::Percentile> {
        let older_then_15s = |range: &RangeInclusive<jiff::Timestamp>| {
            jiff::Timestamp::now()
                .since(*range.end())
                .unwrap()
                .get_seconds()
                > 15
        };

        if self.histogram_range.is_none()
            || self.histogram_range.as_ref().is_some_and(older_then_15s)
        {
            self.fallback_local_hist()
        } else {
            self.percentiles_from_store.clone()
        }
    }

    pub fn fallback_local_hist(&self) -> Vec<log_store::api::Percentile> {
        self.timing
            .iter_quantiles(1)
            .map(|it| log_store::api::Percentile {
                percentile: it.percentile(),
                bucket_ends: it.value_iterated_to(),
                count_in_bucket: it.count_at_value(),
            })
            .dedup_by(|a, b| {
                a.bucket_ends == b.bucket_ends
                    && a.percentile.total_cmp(&b.percentile).is_eq()
                    && a.count_in_bucket == b.count_in_bucket
            })
            .collect_vec()
    }

    pub fn chart<'a>(
        &mut self,
        plot_buf: &'a mut Vec<(f64, f64)>,
        history_len: Duration,
    ) -> Option<ChartParts<'a>> {
        let reference = jiff::Timestamp::now() - history_len;

        let first_recent = self
            .recent_history
            .first()
            .map(|(t, _)| t)
            .cloned()
            .unwrap_or(jiff::Timestamp::MAX);

        plot_buf.clear();
        for xy in self
            .history_from_store
            .iter()
            .skip_while(|(t, _)| *t < reference)
            .take_while(|(t, _)| *t < first_recent)
            .chain(self.recent_history.iter())
            .map(|(x, y)| {
                (
                    (*x - reference)
                        .total(jiff::Unit::Second)
                        .expect("unit is not a calander unit"),
                    *y as f64,
                )
            })
            .skip_while(|(x, _)| *x > history_len.as_secs_f64())
        {
            plot_buf.push(xy);
        }

        assert!(
            plot_buf.iter().all(|(x, _)| *x > 0.0),
            "negative x is not allowed. Arguments: 
            \thistory_len: {history_len:?}
            \tfirst_recent: {first_recent}
            \reference: {reference}"
        );

        Some(ChartParts {
            reading: self.info.clone(),
            data: plot_buf,
        })
    }

    pub(crate) fn logs(&self) -> logs::List {
        self.logs.list()
    }

    pub(crate) fn oldest_in_history(&self) -> jiff::Timestamp {
        jiff::Timestamp::min(
            self.history_from_store
                .first()
                .map(|(ts, _)| ts)
                .copied()
                .unwrap_or(jiff::Timestamp::MAX),
            self.recent_history
                .first()
                .map(|(ts, _)| ts)
                .copied()
                .unwrap_or(jiff::Timestamp::MAX),
        )
    }
}

pub struct IdGen(u16);
impl IdGen {
    fn new() -> Self {
        Self(10_000)
    }
    fn next(&mut self) -> u16 {
        self.0 += 1;
        self.0
    }
}

/// Guaranteed to be unique for a leaf,
/// the path to the leaf (through branch-id's) is
/// encoded with the last byte byte being the leaf's id
pub type TreeKey = [u8; 6];
pub struct Readings {
    // new tree, we are porting to this
    pub lookup: Vec<(Reading, indextree::NodeId)>,
    pub arena: indextree::Arena<Node>,
    pub root: NodeId,
    pub idgen: IdGen,
    // In the ground there are multiple trees
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    pub data: HashMap<TreeKey, SensorInfo>,
}

fn add_leaf(text: String, tree: &mut TreeItem<'static, TreeKey>, key: TreeKey) {
    let new_item = TreeItem::new_leaf(key, text.clone());
    // Todo is exists its fine handle that
    let _ignore_existing = tree.add_child(new_item); // Errors when identifier already exists

    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    let existing = tree.child_mut(new_child).expect("just added it");
    existing.update_text(text);
}

fn add_root<'a>(
    tomato: &dyn Tree,
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
    tomato: &dyn Tree,
    tree: &'a mut TreeItem<'static, TreeKey>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.branch_id(); 6];
    let new_item = TreeItem::new(key, tomato.name(), Vec::new()).unwrap();
    // Add just in case it was not there yet
    let _ignore_existing = tree.add_child(new_item);
    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    tree.child_mut(new_child).expect("just added it")
}

pub(crate) fn tree_key(reading: &Reading) -> TreeKey {
    let mut key = [0u8; 6];
    key[0] = reading.branch_id();

    let mut reading = reading as &dyn Tree;
    for byte in &mut key[1..] {
        reading = match reading.inner() {
            Item::Node(inner) => {
                *byte = inner.branch_id();
                inner
            }
            Item::Leaf(reading::Info { .. }) => {
                return key;
            }
        };
    }
    unreachable!("reading should not be deeper then key size")
}

#[derive(Debug, PartialEq, Eq)]
enum IsPlaceholder {
    Yes,
    No,
}

impl Readings {
    pub fn update(&mut self, reading: Reading) {
        self.update_tree(&reading, IsPlaceholder::No);
        self.update_tree2(&reading, IsPlaceholder::No);
        self.record_data(reading);
    }

    pub(crate) fn populate_from_reading_list(&mut self, list: Vec<Reading>) {
        for reading in list {
            self.update_tree(&reading, IsPlaceholder::Yes);
            self.record_missing_data(reading);
        }
    }

    pub(crate) fn populate_from_device_list(&mut self, list: Vec<Device>) {
        for reading in list.iter().flat_map(|d| d.info().affects_readings) {
            self.update_tree(reading, IsPlaceholder::Yes);
            self.record_missing_data(reading.clone());
        }
    }

    pub fn add_error(&mut self, error: Box<Error>) {
        self.update_tree_err(&error);
        self.update_tree_err2(&error);
        self.record_error(error);
    }

    fn record_error(&mut self, error: Box<Error>) {
        for broken in error.device().info().affects_readings {
            let key = tree_key(broken);

            if let Some(info) = self.data.get_mut(&key) {
                info.condition = Err(error.clone());
                info.logs.add(&error);
            } else {
                let logs = logs::Logs::new_from(&error);
                self.data.insert(
                    key,
                    SensorInfo {
                        info: broken.leaf(),
                        reading: broken.clone(),
                        timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                        percentiles_from_store: Vec::new(),
                        histogram_range: None,

                        recent_history: Vec::new(),
                        history_from_store: Vec::new(),

                        condition: Err(error.clone()),
                        logs,
                        placeholder: true,
                        id: self.idgen.next(),
                    },
                );
            }
        }
    }

    fn record_data(&mut self, reading: Reading) {
        let key = tree_key(&reading);
        let time = jiff::Timestamp::now();

        if let Some(info) = self.data.get_mut(&key) {
            if let Some(last_reading) = info.last_at() {
                info.timing += (time - last_reading)
                    .total(Unit::Millisecond)
                    .expect("no calander units involved") as u64
            }
            info.recent_history.push((time, reading.leaf().val));
            info.condition = Ok(());
        } else {
            let history = vec![(time, reading.leaf().val)];
            self.data.insert(
                key,
                SensorInfo {
                    info: reading.leaf(),
                    reading,
                    timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                    percentiles_from_store: Vec::new(),
                    histogram_range: None,

                    recent_history: history,
                    history_from_store: Vec::new(),

                    condition: Ok(()),
                    logs: logs::Logs::new_empty(),
                    placeholder: true,
                    id: self.idgen.next(),
                },
            );
        }
    }

    fn record_missing_data(&mut self, reading: Reading) {
        let key = tree_key(&reading);
        if self.data.contains_key(&key) {
            return;
        }

        self.data.insert(
            key,
            SensorInfo {
                info: reading.leaf(),
                reading,
                timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                percentiles_from_store: Vec::new(),
                histogram_range: None,

                recent_history: Vec::new(),
                history_from_store: Vec::new(),

                condition: Ok(()),
                logs: logs::Logs::new_empty(),
                placeholder: true,
                id: self.idgen.next(),
            },
        );
    }

    fn update_tree2(&mut self, reading: &Reading, placeholder: IsPlaceholder) {
        let existing = self
            .lookup
            .iter()
            .find(|(r, _)| r.is_same_as(reading))
            .map(|(_, id)| id)
            .and_then(|id| self.arena.get_mut(*id))
            .map(indextree::Node::get_mut);

        let time = jiff::Timestamp::now();
        if let Some(node) = existing {
            let Node::Sensor(ref mut info) = node else {
                panic!("got other node then Sensor for reading");
            };
            assert_eq!(
                IsPlaceholder::No,
                placeholder,
                "cant be a placeholder if there is already a valid node"
            );
            info.reading = reading.clone();
            if let Some(last_reading) = info.last_at() {
                info.timing += (time - last_reading)
                    .total(Unit::Millisecond)
                    .expect("no calander units involved") as u64
            }
            info.recent_history.push((time, reading.leaf().val));
            info.condition = Ok(());
        } else {
            let history = vec![(time, reading.leaf().val)];
            let info = SensorInfo {
                info: reading.leaf(),
                reading: reading.clone(),
                timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                percentiles_from_store: Vec::new(),
                histogram_range: None,

                recent_history: history,
                history_from_store: Vec::new(),

                condition: Ok(()),
                logs: logs::Logs::new_empty(),
                placeholder: true,
                id: self.idgen.next(),
            };

            let node_id = self.arena.new_node(Node::Sensor(info));
            let parent = build_parents(&mut self.arena, self.root, &reading);
            parent.append(node_id, &mut self.arena);
            self.lookup.push((reading.clone(), node_id));
        }
    }

    fn update_tree(&mut self, reading: &Reading, placeholder: IsPlaceholder) {
        let key = tree_key(reading);
        tracing::trace!("reading: {reading:?}");

        let mut tree = add_root(reading as &dyn Tree, &mut self.ground);
        let mut node = match reading.inner() {
            Item::Leaf(_) => unreachable!("no values at level 0"),
            Item::Node(inner) => inner,
        };
        loop {
            match node.inner() {
                Item::Leaf(info) => {
                    let text = if let IsPlaceholder::Yes = placeholder {
                        node.name()
                    } else {
                        format!(
                            "{0}: {1:.2$} {3}",
                            node.name(),
                            info.val,
                            info.precision(),
                            info.unit
                        )
                    };
                    add_leaf(text, tree, key);
                    return;
                }
                Item::Node(inner) => {
                    tree = add_node(node, tree);
                    node = inner;
                }
            };
        }
    }

    fn update_tree_err(&mut self, error: &Error) {
        for broken in error.device().info().affects_readings {
            let key = tree_key(broken);

            let mut tree = add_root(broken as &dyn Tree, &mut self.ground);
            let mut tomato = match broken.inner() {
                Item::Leaf(_) => unreachable!("no values at level 0"),
                Item::Node(inner) => inner,
            };
            loop {
                match tomato.inner() {
                    Item::Leaf(_) => {
                        let text = format!("{}: {}", tomato.name(), error);
                        add_leaf(text, tree, key);
                        break;
                    }
                    Item::Node(inner) => {
                        tree = add_node(tomato, tree);
                        tomato = inner;
                    }
                };
            }
        }
    }

    fn update_tree_err2(&mut self, error: &Error) {
        for broken in error.device().info().affects_readings {
            let existing = self
                .lookup
                .iter()
                .find(|(r, _)| r.is_same_as(broken))
                .map(|(_, id)| id)
                .and_then(|id| self.arena.get_mut(*id))
                .map(indextree::Node::get_mut);

            if let Some(node) = existing {
                let Node::Sensor(ref mut info) = node else {
                    panic!("got other node then Sensor for Error");
                };
                info.condition = Err(Box::new(error.clone()));
                info.logs.add(&error);
            } else {
                let logs = logs::Logs::new_from(&error);
                let info = SensorInfo {
                    info: broken.leaf(),
                    reading: broken.clone(),
                    timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
                    percentiles_from_store: Vec::new(),
                    histogram_range: None,

                    recent_history: Vec::new(),
                    history_from_store: Vec::new(),

                    condition: Err(Box::new(error.clone())),
                    logs,
                    placeholder: true,
                    id: self.idgen.next(),
                };
                let node_id = self.arena.new_node(Node::Sensor(info));
                self.lookup.push((broken.clone(), node_id));
            }
        }
    }

    pub(crate) fn add_fetched(&mut self, reading: Reading, fetched: Fetchable) {
        let sensorinfo = self
            .data
            .get_mut(&tree_key(&reading))
            .expect("data is never removed");
        match fetched {
            Fetchable::Data { timestamps, data } => {
                sensorinfo.history_from_store = timestamps.into_iter().zip(data).collect();
            }
            Fetchable::Logs { logs, start_at } => {
                sensorinfo.logs.from_store = Some(logs::FromStore {
                    list: logs,
                    since: start_at,
                })
            }
            Fetchable::Hist { percentiles, range } => {
                sensorinfo.percentiles_from_store = percentiles;
                sensorinfo.histogram_range = Some(range);
            }
        }
    }

    pub(crate) fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(Node::Root);
        Self {
            lookup: Vec::new(),
            arena,
            root,
            ground: Vec::new(),
            data: HashMap::new(),
            idgen: IdGen::new(),
        }
    }
}

fn build_parents(arena: &mut Arena<Node>, root: NodeId, reading: &Reading) -> NodeId {
    let mut curr = match reading.inner() {
        Item::Leaf(_) => unreachable!("no leafs without parents"),
        Item::Node(inner) => inner,
    };

    let mut parent = get_branch_by_name(arena, root, curr.name())
        .unwrap_or_else(|| arena.new_node(Node::Branch(curr.name())));

    loop {
        match curr.inner() {
            Item::Leaf(_) => return parent,
            Item::Node(inner) => {
                parent = get_branch_by_name(arena, root, inner.name()).unwrap_or_else(|| {
                    let new = arena.new_node(Node::Branch(curr.name()));
                    parent.append(new, arena);
                    new
                });
                curr = inner;
            }
        }
    }
}

fn get_branch_by_name(
    arena: &mut Arena<Node>,
    root: NodeId,
    name_to_find: String,
) -> Option<NodeId> {
    root.children(arena).find(|node| {
        match arena
            .get(*node)
            .expect("no changes since .children()")
            .get()
        {
            Node::Root => unreachable!("root node is never a child"),
            Node::Sensor(_) => false,
            Node::Branch(name) => *name == name_to_find,
        }
    })
}

pub struct ChartParts<'a> {
    pub reading: reading::Info,
    /// array of time, y
    pub data: &'a [(f64, f64)],
}
