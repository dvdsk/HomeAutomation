use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use ratatui::Frame;
use tui_tree_widget::TreeState;

pub(crate) mod history_len;
mod render;
pub mod sensor_info;
use history_len::HistoryLen;

use crate::{fetch::Fetch, Update};
use sensor_info::{Readings, TreeKey};

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
    history_length: HistoryLen,
    input_mode: InputMode,
    tree_state: TreeState<TreeKey>,
    logs_table_state: TableState,
}

pub struct Tab {
    ui_state: UiState,
    readings: Readings,
    plot_buf: Vec<(f64, f64)>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            ui_state: UiState::default(),
            readings: Readings::new(),
            plot_buf: Vec::new(),
        }
    }
}

impl Tab {
    pub fn render(&mut self, frame: &mut Frame, layout: Rect, theme: &Theme) {
        let Self {
            ui_state,
            readings,
            plot_buf,
        } = self;

        let (chart, percentiles, details, logs) = {
            let data = ui_state
                .tree_state
                .selected()
                .last() // Unique leaf id
                .and_then(|key| readings.data.get_mut(key));

            if let Some(data) = data {
                (
                    data.chart(plot_buf),
                    data.percentiles(),
                    Some(data.details()),
                    Some(data.logs()),
                )
            } else {
                plot_buf.clear();
                (None, Vec::new(), None, None)
            }
        };

        let [top, bottom, footer] = render::layout(
            frame,
            layout,
            ui_state,
            readings,
            chart.is_some(),
            logs.is_some(),
        );

        let have_details = details.is_some();
        render::readings_and_details(frame, top, ui_state, readings, details);
        if have_details {
            render::graph_hist_logs(frame, bottom, ui_state, &percentiles, logs, chart, theme);
        }
        render::footer(frame, footer, ui_state, theme);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        self.ui_state.handle_key_all_modes(key)?;
        let plot_open = !self.plot_buf.is_empty();
        match self.ui_state.input_mode {
            InputMode::Normal => self.ui_state.handle_key_normal_mode(key, plot_open)?,
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
            .and_then(|key| self.readings.data.get_mut(key));

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
            .and_then(|key| self.readings.data.get_mut(key))
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
    fn handle_key_normal_mode(&mut self, key: KeyEvent, plot_open: bool) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('b') => {
                if plot_open {
                    self.history_length.start_editing();
                    self.input_mode = InputMode::EditingBounds;
                }
            }
            KeyCode::Char('h') => {
                self.show_histogram = !self.show_histogram;
            }
            KeyCode::Char('l') => {
                self.show_logs = !self.show_logs;
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
