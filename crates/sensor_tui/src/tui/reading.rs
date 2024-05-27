use histogram::Histogram;
use protocol::large_bedroom::Error;
use protocol::{Reading, Tomato, TomatoItem};
use ratatui::{text::Line, widgets::Bar};
use std::collections::HashMap;
use std::{collections::VecDeque, time::Instant};
use tui_tree_widget::TreeItem;

pub struct SensorInfo {
    timing: Histogram,
    history: VecDeque<(Instant, f32)>,
    condition: Result<(), Error>,
}

impl SensorInfo {
    fn last_at(&self) -> Result<Instant, Error> {
        self.condition.clone()?;

        let last = self
            .history
            .front()
            .expect("Items are put in the map when they arrive with a value");
        Ok(last.0)
    }
}

pub type TreeKey = [u8; 6];
pub struct Readings {
    // in the ground there are multiple trees
    pub ground: Vec<TreeItem<'static, TreeKey>>,
    pub data: HashMap<TreeKey, SensorInfo>,
}

fn add_leaf(name: &'static str, val: f32, tree: &mut TreeItem<'static, TreeKey>, key: TreeKey) {
    let text = format!("{}: {}", name, val);
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
    tomato: &dyn Tomato,
    ground: &'a mut Vec<TreeItem<'static, TreeKey>>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.id(); 6];
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
    tomato: &dyn Tomato,
    tree: &'a mut TreeItem<'static, TreeKey>,
) -> &'a mut TreeItem<'static, TreeKey> {
    let key = [tomato.id(); 6];
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

fn extract_keyval(reading: &Reading) -> (TreeKey, f32) {
    let mut key = [0u8; 6];
    key[0] = reading.id();

    let mut reading = reading as &dyn Tomato;
    for byte in &mut key[1..] {
        reading = match reading.inner() {
            TomatoItem::Node(inner) => {
                *byte = inner.id();
                inner
            }
            TomatoItem::Leaf(val) => {
                *byte = reading.id();
                return (key, val);
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

    fn record_data(&mut self, reading: Reading) {
        let (key, val) = extract_keyval(&reading);

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
                    timing: Histogram::new(4, 24).unwrap(),
                    history,
                    condition: Ok(()),
                },
            );
        }
    }

    fn update_tree(&mut self, reading: &Reading) {
        let (key, _) = extract_keyval(reading);

        let mut tomato = reading as &dyn Tomato;
        let mut tree = add_root(tomato, &mut self.ground);
        loop {
            match tomato.inner() {
                TomatoItem::Leaf(val) => {
                    add_leaf(tomato.name(), val, tree, key);
                    return;
                }
                TomatoItem::Node(inner) => {
                    tomato = inner;
                    tree = add_node(tomato, tree);
                }
            };
        }
    }

    pub fn histogram_all(&self) -> Vec<Bar> {
        let mut all = Histogram::new(4, 24).unwrap();
        for (_, val) in self.data.iter() {
            all = all.checked_add(&val.timing).unwrap();
        }
        histogram_bars(&all)
    }

    pub fn histogram(&self, key: TreeKey) -> Vec<Bar> {
        let hist = &self.data.get(&key).unwrap().timing;
        histogram_bars(hist)
    }

    pub fn chart<'a>(
        &mut self,
        selected: &[TreeKey],
        plot_buf: &'a mut Vec<(f64, f64)>,
    ) -> Option<ChartParts<'a>> {
        plot_buf.clear();

        let key = selected.first().unwrap();
        let data = &self.data.get(key).expect("data is never removed");

        for xy in data
            .history
            .iter()
            .map(|(x, y)| (x.elapsed().as_secs_f64(), *y as f64))
        {
            plot_buf.push(xy);
        }

        Some(ChartParts {
            name: format!("{key:?}"),
            data: plot_buf,
        })
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
