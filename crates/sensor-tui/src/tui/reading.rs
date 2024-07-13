use histogram::Histogram;
use protocol::reading_tree::ReadingInfo;
use protocol::reading_tree::{Item, Tree};
use protocol::Error;
use protocol::Reading;

use ratatui::{text::Line, widgets::Bar};
use std::collections::HashMap;
use std::{collections::VecDeque, time::Instant};
use tui_tree_widget::TreeItem;

pub struct SensorInfo {
    name: String,
    timing: Histogram,
    history: VecDeque<(Instant, f32)>,
    condition: Result<(), Box<Error>>,
    log: Vec<(Instant, Error)>,
}

impl SensorInfo {
    fn last_at(&self) -> Result<Instant, Box<Error>> {
        self.condition.clone()?;

        let last = self
            .history
            .front()
            .expect("Items are put in the map when they arrive with a value");
        Ok(last.0)
    }

    pub fn histogram(&self) -> Vec<Bar> {
        histogram_bars(&self.timing)
    }

    pub fn chart<'a>(&self, plot_buf: &'a mut Vec<(f64, f64)>) -> Option<ChartParts<'a>> {
        plot_buf.clear();

        for xy in self
            .history
            .iter()
            .map(|(x, y)| (x.elapsed().as_secs_f64(), *y as f64))
        {
            plot_buf.push(xy);
        }

        Some(ChartParts {
            name: self.name.clone(),
            data: plot_buf,
        })
    }
}

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
            let (key, name, _) = extract_leaf_info(&broken);

            if let Some(info) = self.data.get_mut(&key) {
                info.condition = Err(error.clone());
                info.log.push((Instant::now(), (*error).clone()));
            } else {
                self.data.insert(
                    key,
                    SensorInfo {
                        name,
                        timing: Histogram::new(4, 24).unwrap(),
                        history: VecDeque::new(),
                        condition: Err(error.clone()),
                        log: vec![(Instant::now(), (*error).clone())],
                    },
                );
            }
        }
    }

    fn record_data(&mut self, reading: Reading) {
        let (key, name, val) = extract_leaf_info(&reading);

        if let Some(info) = self.data.get_mut(&key) {
            if let Ok(last_reading) = info.last_at() {
                info.timing
                    .increment(last_reading.elapsed().as_millis() as u64)
                    .unwrap();
            }
            info.history.push_front((Instant::now(), val));
            info.condition = Ok(());
        } else {
            let mut history = VecDeque::new();
            history.push_front((Instant::now(), val));
            self.data.insert(
                key,
                SensorInfo {
                    name,
                    timing: Histogram::new(4, 24).unwrap(),
                    history,
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
                Item::Leaf(ReadingInfo { val, .. }) => {
                    let text = format!("{}: {}", tomato.name(), val);
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
            let (key, _, _) = extract_leaf_info(&broken);

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

    pub fn histogram_all(&self) -> Vec<Bar> {
        let mut all = Histogram::new(4, 24).unwrap();
        for (_, val) in self.data.iter() {
            all = all.checked_add(&val.timing).unwrap();
        }
        histogram_bars(&all)
    }
}

pub struct ChartParts<'a> {
    pub name: String,
    pub data: &'a [(f64, f64)],
}

fn histogram_bars(hist: &Histogram) -> Vec<Bar<'static>> {
    let percentiles = hist
        .percentiles(&[25.0, 50.0, 75.0, 90.0, 95.0, 100.0])
        .unwrap();
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
