use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::KeyEvent;
use log_store::api::Percentile;
use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use ratatui::Frame;
use tui_tree_widget::TreeState;

mod handle_key;
pub(crate) mod history_len;
pub mod render;
pub mod sensor_info;
use history_len::HistoryLen;

use crate::{fetch::Fetch, Update};
use sensor_info::{is_leaf_id, ChartParts, Readings};

use super::Theme;

#[derive(Debug, Default, Clone, Copy)]
struct InputMode {
    editing_bounds: bool,
    chart_cursor: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ChartCursor {
    Disabled {
        // steps (characters) from the left where the cursor
        // will be shown again if its re-enabled
        steps: i32,
    },
    Enabled {
        // steps (characters) from the left
        steps: i32,
    },
}

impl Default for ChartCursor {
    fn default() -> Self {
        Self::Disabled { steps: 0 }
    }
}

impl ChartCursor {
    fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }
    fn toggle(&mut self) {
        *self = match *self {
            ChartCursor::Disabled { steps } => ChartCursor::Enabled { steps },
            ChartCursor::Enabled { steps } => ChartCursor::Disabled { steps },
        }
    }
    fn shift(&mut self, offset: i32) {
        let ChartCursor::Enabled { ref mut steps } = self else {
            return;
        };

        *steps += offset;
    }

    fn get(&mut self, chart_width: u16) -> Option<u16> {
        let steps = match self {
            ChartCursor::Disabled { .. } => return None,
            ChartCursor::Enabled { steps } => steps,
        };

        *steps = if *steps >= chart_width as i32 {
            0
        } else if *steps < 0 {
            chart_width as i32 - 1
        } else {
            *steps
        };

        Some(*steps as u16)
    }
}

#[derive(Default)]
pub struct UiState {
    show_histogram: bool,
    show_logs: bool,
    show_complete_help: bool,
    chart_cursor: ChartCursor,
    history_length: HistoryLen,
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
    history_len: std::time::Duration,
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

        res.chart_parts.push(info.chart(buf, history_len));
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
            fill_data(to_display, readings, plot_bufs, ui_state.history_length.dur)
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
                    self.ui_state.history_length.state = history_len::State::Fetched;
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

        let dur = self.ui_state.history_length.dur;
        let history_len = &mut self.ui_state.history_length.state;
        for data in needed {
            let Some(data) = self.readings.get_by_ui_id(data) else {
                continue;
            };

            fetcher.assure_up_to_date(
                dur,
                || *history_len = history_len::State::Fetching(Instant::now()),
                data.reading.clone(),
                data.oldest_in_history(),
                data.logs.covers_from(),
                data.histogram_range.clone(),
            );
        }
    }
}
