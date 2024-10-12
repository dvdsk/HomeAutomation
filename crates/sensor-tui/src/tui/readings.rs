use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use log_store::api::Percentile;
use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use ratatui::Frame;
use tui_tree_widget::TreeState;

pub(crate) mod history_len;
pub mod render;
pub mod sensor_info;
use history_len::HistoryLen;

use crate::{fetch::Fetch, Update};
use sensor_info::{ChartParts, Readings};

use super::Theme;

#[derive(Debug, Default, Clone, Copy)]
enum InputMode {
    #[default]
    Normal,
    EditingBounds,
}

#[derive(Default)]
pub struct UiState {
    show_histogram: bool,
    show_logs: bool,
    show_complete_help: bool,
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
            ui_state.reading_selected = selected.is_some();

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
        self.ui_state.handle_key_all_modes(key)?;
        match self.ui_state.input_mode {
            InputMode::Normal => self.ui_state.handle_key_normal_mode(key)?,
            InputMode::EditingBounds => self.ui_state.handle_key_bounds_mode(key)?,
        };

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
        let Some(data) = self
            .ui_state
            .tree_state
            .selected()
            .last() // Unique leaf id
            .and_then(|key| self.readings.get_by_ui_id(*key))
        else {
            return; // Nothing selected
        };

        let dur = self.ui_state.history_length.dur;
        let history_len = &mut self.ui_state.history_length.state;
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

impl UiState {
    fn handle_key_normal_mode(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('b') if self.reading_selected => {
                self.history_length.start_editing();
                self.input_mode = InputMode::EditingBounds;
            }
            KeyCode::Char('h') if self.reading_selected => {
                self.show_histogram = !self.show_histogram;
            }
            KeyCode::Char('l') if self.reading_selected => {
                self.show_logs = !self.show_logs;
            }
            KeyCode::Char('c') if self.reading_selected => {
                let id = *self
                    .tree_state
                    .selected()
                    .last()
                    .expect("reading_selected is true");
                if !self.comparing.remove(&id) {
                    self.comparing.insert(id);
                }
            }
            KeyCode::Char('?') => {
                self.show_complete_help = !self.show_complete_help;
            }
            _ => return Some(key),
        }

        None
    }

    fn handle_key_bounds_mode(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.history_length.exit_editing();
                self.input_mode = InputMode::Normal;
                None
            }
            _ => self.history_length.process(key),
        }
    }

    fn handle_key_all_modes(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Down => {
                self.tree_state.key_down();
            }
            KeyCode::Up => {
                self.tree_state.key_up();
            }
            KeyCode::Enter => {
                self.tree_state.toggle_selected();
            }
            _ => return Some(key),
        }
        None
    }
}
