use histogram::Histogram;
use protocol::reading_tree::ReadingInfo;
use protocol::reading_tree::{Item, Tree};
use protocol::Error;
use protocol::Reading;

use ratatui::{text::Line, widgets::Bar};
use std::collections::HashMap;
use std::sync::TryLockError;
use std::time::Instant;
use time::OffsetDateTime;
use tui_tree_widget::TreeItem;

mod fetch;
use fetch::StoredHistory;

#[derive(Debug)]
pub struct SensorInfo {
    timing: Histogram,
    pub reading: Reading,
    recent_history: Vec<(OffsetDateTime, f32)>,
    pub stored_history: StoredHistory,
    condition: Result<(), Box<Error>>,
    log: Vec<(Instant, Error)>,
}

impl SensorInfo {
    fn last_at(&self) -> Result<OffsetDateTime, Box<Error>> {
        self.condition.clone()?;

        let last = self
            .recent_history
            .last()
            .expect("Items are put in the map when they arrive with a value");
        Ok(last.0)
    }

    pub fn histogram(&self) -> Vec<Bar> {
        histogram_bars(&self.timing)
    }

    pub fn chart<'a>(&mut self, plot_buf: &'a mut Vec<(f64, f64)>) -> Option<ChartParts<'a>> {
        let guard = match self.stored_history.data.try_lock() {
            Ok(list) => Some(list),
            Err(TryLockError::WouldBlock) => None,
            Err(other) => panic!(
                "fetching sensor history from data-store \
                panicked: {other}"
            ),
        };
        let empty = Vec::new();
        let old_history = guard.as_deref().unwrap_or(&empty);

        let reference = old_history
            .first()
            .map(|(t, _)| t)
            .or_else(|| self.recent_history.first().map(|(t, _)| t))?;

        let first_recent = self
            .recent_history
            .first()
            .map(|(t, _)| t)
            .cloned()
            .unwrap_or(OffsetDateTime::from_unix_timestamp(0).unwrap());
        plot_buf.clear();

        for xy in old_history
            .iter()
            .take_while(|(t, _)| *t < first_recent)
            .chain(self.recent_history.iter())
            .map(|(x, y)| ((*x - *reference).as_seconds_f64(), *y as f64))
        {
            plot_buf.push(xy);
        }

        Some(ChartParts {
            reading: self.reading.clone(),
            data: plot_buf,
        })
    }
}

/// Guaranteed to be unique for a leaf,
/// the path to the leaf (through branch-id's) is
/// encoded with the last byte byte being the leaf's id
pub type TreeKey = [u8; 6];
pub struct Readings {
    // in the ground there are multiple trees
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    pub data: HashMap<TreeKey, SensorInfo>,
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
    // add just in case it was not there yet
    let _ignore_existing = tree.add_child(new_item);
    let new_child = tree
        .children()
        .iter()
        .position(|item| *item.identifier() == key)
        .expect("just added it");
    tree.child_mut(new_child).expect("just added it")
}

fn extract_leaf_info(reading: &Reading) -> (TreeKey, String, f32) {
    let mut key = [0u8; 6];
    key[0] = reading.branch_id();

    let mut reading = reading as &dyn Tree;
    for byte in &mut key[1..] {
        reading = match reading.inner() {
            Item::Node(inner) => {
                *byte = inner.branch_id();
                inner
            }
            Item::Leaf(ReadingInfo { val, .. }) => {
                let name = reading.name();
                return (key, name, val);
            }
        };
    }
    unreachable!("reading should not be deeper then key size")
}

impl Readings {
    pub fn add(&mut self, reading: Reading) {
        self.update_tree(&reading);
        self.record_data(reading);
    }

    pub fn add_error(&mut self, error: Box<Error>) {
        self.update_tree_err(&error);
        self.record_error(error);
    }

    fn record_error(&mut self, error: Box<Error>) {
        for broken in error.device().affected_readings() {
            let (key, _, _) = extract_leaf_info(broken);

            if let Some(info) = self.data.get_mut(&key) {
                info.condition = Err(error.clone());
                info.log.push((Instant::now(), (*error).clone()));
            } else {
                self.data.insert(
                    key,
                    SensorInfo {
                        reading: broken.clone(),
                        timing: Histogram::new(4, 24).unwrap(),
                        recent_history: Vec::new(),
                        stored_history: StoredHistory::new(),
                        condition: Err(error.clone()),
                        log: vec![(Instant::now(), (*error).clone())],
                    },
                );
            }
        }
    }

    fn record_data(&mut self, reading: Reading) {
        let (key, _, val) = extract_leaf_info(&reading);
        let time = OffsetDateTime::now_utc();

        if let Some(info) = self.data.get_mut(&key) {
            if let Ok(last_reading) = info.last_at() {
                info.timing
                    .increment((time - last_reading).whole_milliseconds() as u64)
                    .unwrap();
            }
            info.recent_history.push((time, val));
            info.condition = Ok(());
        } else {
            let history = vec![(time, val)];
            self.data.insert(
                key,
                SensorInfo {
                    reading,
                    timing: Histogram::new(4, 24).unwrap(),
                    recent_history: history,
                    stored_history: StoredHistory::new(),
                    condition: Ok(()),
                    log: Vec::new(),
                },
            );
        }
    }

    fn update_tree(&mut self, reading: &Reading) {
        let (key, _, _) = extract_leaf_info(reading);

        let mut tree = add_root(reading as &dyn Tree, &mut self.ground);
        let mut tomato = match reading.inner() {
            Item::Leaf(_) => unreachable!("no values at level 0"),
            Item::Node(inner) => inner,
        };
        loop {
            match tomato.inner() {
                Item::Leaf(info) => {
                    let text = format!(
                        "{0}: {1:.2$} {3}",
                        tomato.name(),
                        info.val,
                        info.precision(),
                        info.unit
                    );
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

    fn update_tree_err(&mut self, error: &Error) {
        for broken in error.device().affected_readings() {
            let (key, _, _) = extract_leaf_info(broken);

            let mut tree = add_root(broken as &dyn Tree, &mut self.ground);
            let mut tomato = match broken.inner() {
                Item::Leaf(_) => unreachable!("no values at level 0"),
                Item::Node(inner) => inner,
            };
            loop {
                match tomato.inner() {
                    Item::Leaf(_) => {
                        let text = format!("{}: {:?}", tomato.name(), error);
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

    pub fn histogram_all(&self) -> Vec<Bar> {
        let mut all = Histogram::new(4, 24).unwrap();
        for (_, val) in self.data.iter() {
            all = all.checked_add(&val.timing).unwrap();
        }
        histogram_bars(&all)
    }
}

pub struct ChartParts<'a> {
    pub reading: Reading,
    pub data: &'a [(f64, f64)],
}

fn histogram_bars(hist: &Histogram) -> Vec<Bar<'static>> {
    let Some(percentiles) = hist
        .percentiles(&[25.0, 50.0, 75.0, 90.0, 95.0, 100.0])
        .unwrap()
    else {
        return Vec::new();
    };

    percentiles
        .into_iter()
        .map(|(p, bucket)| {
            Bar::default()
                .value(bucket.count())
                .text_value(format!("p{p}: {}", bucket.count()))
                .label(Line::from(format!("{}..{}", bucket.start(), bucket.end())))
        })
        .collect()
}
