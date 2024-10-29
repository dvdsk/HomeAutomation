#![allow(unused)]

use crate::lights::conversion::temp_to_xy;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct State {
    pub(crate) brightness: f64,
    pub(crate) color_temp: usize,
    pub(crate) color_xy: (f64, f64),
    pub(crate) on: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            color_temp: 2700,
            color_xy: temp_to_xy(2700),
            on: false,
        }
    }
}

impl State {
    pub(crate) fn apply(&mut self, change: Change) -> State {
        todo!()
    }
}

#[derive(Debug)]
// TODO: make enum
pub(crate) struct Change {
    pub(crate) friendly_name: String,
    pub(crate) on: Option<bool>,
    pub(crate) brightness: Option<f64>,
    pub(crate) color_temp: Option<usize>,
    pub(crate) color_xy: Option<(f64, f64)>,
}
