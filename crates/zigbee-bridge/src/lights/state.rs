#![allow(unused)]
#[derive(Debug, PartialEq)]
pub(crate) struct State {
    pub(crate) brightness: f64,
    pub(crate) color_temp: usize,
    pub(crate) color_xy: (f64, f64),
    pub(crate) on: bool,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            on: todo!(),
            brightness: todo!(),
            color_temp: todo!(),
            color_xy: todo!(),
        }
    }

    pub(crate) fn apply(&mut self, change: Option<Change>) -> State {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Change {
    pub(crate) friendly_name: String,
    pub(crate) on: Option<bool>,
    pub(crate) brightness: Option<f64>,
    pub(crate) color_temp: Option<usize>,
    pub(crate) color_xy: Option<(f64, f64)>,
}

