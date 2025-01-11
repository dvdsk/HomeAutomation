use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::KeyEvent;
use log_store::api::Percentile;
use protocol::IsSameAs;
use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use ratatui::Frame;
use tui_tree_widget::TreeState;

mod handle_key;
pub(crate) mod plot_range;
pub mod render;
pub mod sensor_info;
use plot_range::PlotRange;

use crate::{fetch::Fetch, Update};
use sensor_info::{is_leaf_id, ChartParts, Readings};

use super::Theme;

#[derive(Debug, Default, Clone, Copy)]
struct InputMode {
    editing_bounds: bool,
    chart_cursor: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ChartCursor {
    enabled: bool,
    steps: i32,
    chart_width: Option<u16>,
}

impl Default for ChartCursor {
    fn default() -> Self {
        Self {
            enabled: false,
            steps: 0,
            chart_width: None,
        }
    }
}

impl ChartCursor {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    fn shift(&mut self, offset: i32) {
        self.steps += offset;
    }

    fn get(&mut self, chart_width: u16) -> Option<u16> {
        self.chart_width = Some(chart_width);
        if !self.enabled {
            return None;
        }

        self.steps = if self.steps >= chart_width as i32 {
            0
        } else if self.steps < 0 {
            chart_width as i32 - 1
        } else {
            self.steps
        };

        Some(self.steps as u16)
    }

    fn zoom_in(&self) -> [f64; 2] {
        let Some(chart_width) = self.chart_width else {
            return [1.0, 1.0];
        };

        let pos = self.steps as f64 / chart_width as f64;
        [pos - 0.1, pos + 0.1]
    }

    fn zoom_out(&self) -> [f64; 2] {
        let Some(chart_width) = self.chart_width else {
            return [1.0, 1.0];
        };

        let pos = self.steps as f64 / chart_width as f64;
        [pos - 10.0, pos + 10.0]
    }
}

#[derive(Default)]
pub struct UiState {
    show_histogram: bool,
    show_logs: bool,
    show_complete_help: bool,
    chart_cursor: ChartCursor,
    plot_range: PlotRange,
    input_mode: InputMode,
    tree_state: TreeState<u16>,
    logs_table_state: TableState,
    reading_selected: bool,
    comparing: HashSet<u16>,
}

pub struct Tab {
    ui_state: UiState,
    readings: Readings,
    plot_bufs: Vec<Vec<(f64, f64)>>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            ui_state: UiState::default(),
            readings: Readings::new(),
            plot_bufs: Vec::new(),
        }
    }
}

#[derive(Default)]
struct DataToDisplay<'a> {
    chart_parts: Vec<ChartParts<'a>>,
    precentiles: Vec<Percentile>,
    details: Option<sensor_info::Details>,
    logs: Option<sensor_info::LogList>,
}

fn fill_data<'a>(
    to_display: Vec<u16>,
    readings: &mut Readings,
    plot_bufs: &'a mut Vec<Vec<(f64, f64)>>,
    history_len: &plot_range::Range,
) -> DataToDisplay<'a> {
    for buf in plot_bufs.iter_mut() {
        buf.clear();
    }
    plot_bufs.resize_with(to_display.len().max(plot_bufs.len()), Vec::new);
    let mut res = DataToDisplay::default();
    for (i, (ui_id, buf)) in to_display.into_iter().zip(plot_bufs).enumerate() {
        let Some(info) = readings.get_by_ui_id(ui_id) else {
            continue;
        };

        if i == 0 {
            // TODO merge? if possible;
            res.precentiles = info.percentiles();
            // TODO merge to show details from all plots
            res.details = Some(info.details());
            // TODO merge to show logs interleaved
            res.logs = Some(info.logs());
        }

        res.chart_parts.push(info.chart(buf, &history_len));
    }
    res
}

impl Tab {
    pub fn render(&mut self, frame: &mut Frame, layout: Rect, theme: &Theme) {
        let Self {
            ui_state,
            readings,
            plot_bufs,
        } = self;

        let DataToDisplay {
            chart_parts,
            precentiles,
            details,
            logs,
        } = {
            let selected = ui_state.tree_state.selected().last();
            ui_state.reading_selected = selected.copied().is_some_and(is_leaf_id);

            let mut to_display: Vec<_> = selected
                .into_iter()
                .chain(ui_state.comparing.iter())
                .copied()
                .collect();
            to_display.sort();
            to_display.dedup();
            fill_data(to_display, readings, plot_bufs, &ui_state.plot_range.range)
        };

        let [top, bottom, footer] = render::layout(
            frame,
            layout,
            readings,
            !chart_parts.is_empty(),
            logs.is_some(),
            ui_state,
        );

        let have_details = details.is_some();
        render::readings_and_details(frame, top, ui_state, readings, details);
        if have_details {
            render::graph_hist_logs(
                frame,
                bottom,
                ui_state,
                &precentiles,
                logs,
                chart_parts,
                theme,
            );
        }
        render::footer(frame, footer, ui_state, theme);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        self.ui_state.handle_key_all(key)?;
        if self.ui_state.input_mode.editing_bounds {
            self.ui_state.handle_key_bounds(key)?;
        } else {
            self.ui_state.handle_key_normal_mode(key)?;
        };
        if self.ui_state.input_mode.chart_cursor {
            self.ui_state.handle_key_cursor(key)?;
        }

        Some(key)
    }

    pub fn process_update(&mut self, update: Update) {
        let data = self
            .ui_state
            .tree_state
            .selected()
            .last() // Unique leaf id
            .and_then(|key| self.readings.get_by_ui_id(*key));

        match update {
            Update::SensorReading(reading) => {
                self.readings.update(reading);
            }
            Update::ReadingList(list) => {
                self.readings.populate_from_reading_list(list);
            }
            Update::DeviceList(list) => {
                self.readings.populate_from_device_list(list);
            }
            Update::SensorError(err) => self.readings.add_error(err),
            Update::Fetched {
                reading,
                thing: fetched,
            } => {
                if data
                    .as_ref()
                    .is_some_and(|i| i.reading.is_same_as(&reading))
                {
                    self.ui_state.plot_range.state = plot_range::State::Fetched;
                }
                self.readings.add_fetched(reading, fetched);
            }

            _ => (),
        }

        if self.ui_state.tree_state.selected().is_empty() {
            self.ui_state.tree_state.select_first();
        }
    }

    pub(crate) fn fetch_if_needed(&mut self, fetcher: &mut Fetch) {
        let selected = self.ui_state.tree_state.selected().last();
        let needed = selected
            .into_iter()
            .chain(self.ui_state.comparing.iter())
            .copied();

        let history_len = &mut self.ui_state.plot_range.state;
        for data in needed {
            let Some(data) = self.readings.get_by_ui_id(data) else {
                continue;
            };

            fetcher.assure_up_to_date(
                self.ui_state.plot_range.range,
                || *history_len = plot_range::State::Fetching(Instant::now()),
                data.reading.clone(),
                data.covers(),
                data.logs.covers(),
                data.histogram_range.clone(),
            );
        }
    }
}
