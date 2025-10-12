use hdrhistogram::Histogram;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use jiff::Unit;
use log_store::api::{self, Percentile};
use protocol::{reading, IsSameAs};
use protocol::reading::tree::{Item, Tree};
use protocol::Reading;
use protocol::{Device, Error};

use std::ops::RangeInclusive;

use crate::Fetchable;

mod logs;
pub(crate) use logs::List as LogList;
pub(crate) use logs::LogSource;

use super::plot_range;

#[derive(Debug)]
pub(crate) enum Cover {
    Overlapping {
        store: RangeInclusive<jiff::Timestamp>,
        local: RangeInclusive<jiff::Timestamp>,
    },
    OnlyLocal(RangeInclusive<jiff::Timestamp>),
    OnlyStore(RangeInclusive<jiff::Timestamp>),
    Distinct {
        store: RangeInclusive<jiff::Timestamp>,
        local: RangeInclusive<jiff::Timestamp>,
    },
    None,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Stored on the heap through Vec, mem waste of Branches is \
    worth the perf of not boxing SensorInfo"
)]
#[derive(Debug)]
pub enum Node {
    Sensor(SensorInfo),
    Branch(String),
    Root,
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct SensorInfo {
    /// unique id used by the rendered tree to refer back to
    /// this specific item.
    pub ui_id: u16,
    #[derivative(Debug = "ignore")]
    pub info: reading::Info,
    /// This value is not up to date, only use for requesting
    /// data use the last element of recent_history for printing
    pub reading: Reading,
    #[derivative(Debug = "ignore")]
    timing: Histogram<u64>,
    #[derivative(Debug = "ignore")]
    pub percentiles_from_store: Vec<Percentile>,
    #[derivative(Debug = "ignore")]
    recent_history: Vec<(jiff::Timestamp, f32)>,
    pub histogram_range: Option<RangeInclusive<jiff::Timestamp>>,
    #[derivative(Debug = "ignore")]
    pub history_from_store: Vec<(jiff::Timestamp, f32)>,
    condition: Result<(), Box<Error>>,
    #[derivative(Debug = "ignore")]
    pub(crate) logs: logs::Logs,
    pub is_placeholder: bool,
}

impl SensorInfo {
    fn new(reading: &Reading, is_placeholder: bool, ui_id: u16) -> Self {
        let time = jiff::Timestamp::now();
        let recent_history = if is_placeholder {
            Vec::new()
        } else {
            vec![(time, reading.info().val)]
        };

        Self {
            info: reading.info(),
            reading: reading.clone(),
            timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
            percentiles_from_store: Vec::new(),
            histogram_range: None,

            recent_history,
            history_from_store: Vec::new(),

            condition: Ok(()),
            logs: logs::Logs::new_empty(),
            is_placeholder,
            ui_id,
        }
    }

    fn new_err(error: &Error, broken: &Reading, ui_id: u16) -> Self {
        let logs = logs::Logs::new_from(error);
        SensorInfo {
            info: broken.info(),
            reading: broken.clone(),
            timing: Histogram::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap(),
            percentiles_from_store: Vec::new(),
            histogram_range: None,

            recent_history: Vec::new(),
            history_from_store: Vec::new(),

            condition: Err(Box::new(error.clone())),
            logs,
            is_placeholder: true,
            ui_id,
        }
    }

    fn update(&mut self, reading: &Reading) {
        let time = jiff::Timestamp::now();
        self.info = reading.info();
        if let Some(last_reading) = self.last_at() {
            self.timing += (time - last_reading)
                .total(Unit::Millisecond)
                .expect("no calander units involved") as u64
        }
        self.recent_history.push((time, reading.info().val));
        self.is_placeholder = false;
        self.condition = Ok(());
    }
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
        range: &plot_range::Range,
    ) -> ChartParts<'a> {
        let start = *range.range_inclusive().start();
        let before_start = |t: &jiff::Timestamp| t < &start;

        let first_recent = self.recent_history.first().map(|(t, _)| t).cloned();
        let not_recent = |t: &jiff::Timestamp| !first_recent.is_some_and(|recent| t > &recent);

        let end = *range.range_inclusive().end();
        let before_end = |t: &jiff::Timestamp| t < &end;

        plot_buf.clear();
        let plot = self
            .history_from_store
            .iter()
            .take_while(|(t, _)| not_recent(t))
            .chain(self.recent_history.iter())
            .skip_while(|(t, _)| before_start(t))
            .take_while(|(t, _)| before_end(t))
            .collect_vec();

        for (x, y) in plot {
            plot_buf.push((
                (*x - start)
                    .total(jiff::Unit::Second)
                    .expect("unit is not a calander unit"),
                *y as f64,
            ))
        }

        for (i, (x, _)) in plot_buf.iter().enumerate() {
            assert!(
                *x >= 0.0,
                "negative x is not allowed. Info:
            \t x: {x},
            \t index: {i},
            \t range: {range:?}
            \t first_recent: {first_recent:?}
            \t start: {start}"
            );
        }

        ChartParts {
            info: self.info.clone(),
            reading: self.reading.clone(),
            data: plot_buf,
        }
    }

    pub(crate) fn logs(&self) -> logs::List {
        self.logs.list()
    }

    pub(crate) fn covers(&self) -> Cover {
        use jiff::Timestamp as Ts;
        let are_overlapping =
            |store: &RangeInclusive<Ts>, local: &RangeInclusive<Ts>| local.start() <= store.end();

        match (
            self.history_from_store
                .first()
                .map(|(ts, _)| ts)
                .zip(self.history_from_store.last().map(|(ts, _)| ts))
                .map(|(a, b)| *a..=*b),
            self.recent_history
                .first()
                .map(|(ts, _)| ts)
                .zip(self.recent_history.last().map(|(ts, _)| ts))
                .map(|(a, b)| *a..=*b),
        ) {
            (None, None) => Cover::None,
            (None, Some(local)) => Cover::OnlyLocal(local),
            (Some(store), None) => Cover::OnlyStore(store),
            (Some(store), Some(local)) if are_overlapping(&store, &local) => {
                Cover::Overlapping { store, local }
            }
            (Some(store), Some(local)) => Cover::Distinct { store, local },
        }
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

pub fn is_leaf_id(id: u16) -> bool {
    id > 10_000
}

/// Guaranteed to be unique for a leaf,
/// the path to the leaf (through branch-id's) is
/// encoded with the last byte byte being the leaf's id
pub type TreeKey = [u8; 6];
pub struct Readings {
    pub lookup_by_reading: Vec<(Reading, indextree::NodeId)>,
    pub lookup_by_ui_id: Vec<(u16, indextree::NodeId)>,
    pub arena: indextree::Arena<Node>,
    pub root: NodeId,
    pub idgen: IdGen,
}

impl Readings {
    pub fn get_by_ui_id(&mut self, id: u16) -> Option<&mut SensorInfo> {
        self.lookup_by_ui_id
            .iter_mut()
            .find(|(ui_id, _)| id == *ui_id)
            .map(|(_, node_id)| node_id)
            .and_then(|id| self.arena.get_mut(*id))
            .map(indextree::Node::get_mut)
            .map(|node| match node {
                Node::Sensor(info) => info,
                Node::Branch(_) | Node::Root => {
                    unreachable!("Only Sensor nodes are put in the lookup table")
                }
            })
    }

    pub fn update(&mut self, reading: Reading) {
        self.update_tree(&reading, false);
    }

    pub(crate) fn populate_from_reading_list(&mut self, list: Vec<Reading>) {
        for reading in list {
            self.update_tree(&reading, true);
        }
    }

    pub(crate) fn populate_from_device_list(&mut self, list: Vec<Device>) {
        for reading in list.iter().flat_map(|d| d.info().affects_readings) {
            self.update_tree(reading, true);
        }
    }

    pub fn add_error(&mut self, error: Box<Error>) {
        self.update_tree_err(&error);
    }

    fn update_tree(&mut self, reading: &Reading, is_placeholder: bool) {
        let existing = self
            .lookup_by_reading
            .iter()
            .find(|(r, _)| r.is_same_as(reading))
            .map(|(_, id)| id)
            .and_then(|id| self.arena.get_mut(*id))
            .map(indextree::Node::get_mut);

        if let Some(node) = existing {
            let Node::Sensor(ref mut info) = node else {
                panic!("got other node then Sensor for reading");
            };

            if is_placeholder {
                return; // can receive sensor update before populating from list
            }
            info.update(reading);
        } else {
            let ui_id = self.idgen.next();
            let info = SensorInfo::new(reading, is_placeholder, ui_id);
            let node_id = self.arena.new_node(Node::Sensor(info));
            let parent = build_parents(&mut self.arena, self.root, reading);
            parent.append(node_id, &mut self.arena);
            self.lookup_by_reading.push((reading.clone(), node_id));
            self.lookup_by_ui_id.push((ui_id, node_id));
        }
    }

    fn update_tree_err(&mut self, error: &Error) {
        for broken in error.device().info().affects_readings {
            let existing = self
                .lookup_by_reading
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
                info.logs.add(error);
            } else {
                let ui_id = self.idgen.next();
                let info = SensorInfo::new_err(error, broken, ui_id);
                let node_id = self.arena.new_node(Node::Sensor(info));
                self.lookup_by_reading.push((broken.clone(), node_id));
                self.lookup_by_ui_id.push((ui_id, node_id));
            }
        }
    }

    pub(crate) fn add_fetched(&mut self, reading: Reading, fetched: Fetchable) {
        let Node::Sensor(info) = self
            .lookup_by_reading
            .iter()
            .find(|(r, _)| r.is_same_as(&reading))
            .map(|(_, id)| id)
            .and_then(|id| self.arena.get_mut(*id))
            .map(indextree::Node::get_mut)
            .expect("Data can only be fetched if it exists in the tree")
        else {
            panic!("Node for a reading should always be a sensornode");
        };

        match fetched {
            Fetchable::Data { timestamps, data } => {
                info.history_from_store = timestamps.into_iter().zip(data).collect();
            }
            Fetchable::Logs { logs, start_at } => {
                info.logs.from_store = Some(logs::FromStore {
                    list: logs,
                    since: start_at,
                })
            }
            Fetchable::Hist { percentiles, range } => {
                info.percentiles_from_store = percentiles;
                info.histogram_range = Some(range);
            }
        }
    }

    pub(crate) fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(Node::Root);
        Self {
            lookup_by_reading: Vec::new(),
            lookup_by_ui_id: Vec::new(),
            arena,
            root,
            idgen: IdGen::new(),
        }
    }
}

fn build_parents(arena: &mut Arena<Node>, root: NodeId, reading: &Reading) -> NodeId {
    let mut curr = reading as &dyn Tree;
    let mut parent = root;
    loop {
        match curr.inner() {
            Item::Leaf(_) => return parent,
            Item::Node(inner) => {
                parent = get_child_by_name(arena, parent, curr.name()).unwrap_or_else(|| {
                    let new = arena.new_node(Node::Branch(curr.name()));
                    parent.append(new, arena);
                    new
                });
                curr = inner;
            }
        }
    }
}

fn get_child_by_name(
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
    pub info: reading::Info,
    pub reading: Reading,
    /// array of time, y
    pub data: &'a mut [(f64, f64)],
}
