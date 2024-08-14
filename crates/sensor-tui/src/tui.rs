use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::TableState,
};
use std::{collections::HashMap, io::stdout, net::SocketAddr, sync::mpsc, time::Duration};
use tui_tree_widget::TreeState;

mod reading;
use reading::Readings;
mod render;
use crate::Update;

mod history_len;
use history_len::HistoryLen;

use self::reading::TreeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ActiveTab {
    #[default]
    Readings,
    Affectors,
}

impl ActiveTab {
    fn swap(self) -> Self {
        match self {
            Self::Readings => Self::Affectors,
            Self::Affectors => Self::Readings,
        }
    }

    fn number(&self) -> usize {
        match self {
            ActiveTab::Readings => 0,
            ActiveTab::Affectors => 1,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum InputMode {
    #[default]
    Normal,
    EditingBounds,
}

pub fn run(
    rx: mpsc::Receiver<Update>,
    shutdown_tx: mpsc::Sender<color_eyre::Result<Update>>,
    data_store: SocketAddr,
    log_store: SocketAddr,
) -> Result<(), std::io::Error> {
    App::default().run(rx, shutdown_tx, data_store, log_store)
}

#[derive(Default)]
struct App {
    theme: render::Theme,
    input_mode: InputMode,
    active_tab: ActiveTab,
    show_histogram: bool,
    show_logs: bool,
    reading_tree_state: TreeState<TreeKey>,
    affector_tree_state: TreeState<TreeKey>,
    logs_table_state: TableState,
    history_length: HistoryLen,
}

impl App {
    pub fn run(
        &mut self,
        rx: mpsc::Receiver<Update>,
        shutdown_tx: mpsc::Sender<color_eyre::Result<Update>>,
        data_store: SocketAddr,
        log_store: SocketAddr,
    ) -> Result<(), std::io::Error> {
        let mut readings = Readings {
            ground: Vec::new(),
            data: HashMap::new(),
        };

        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        let affectors = Vec::new();
        let mut plot_buf = Vec::new();

        loop {
            let data = self
                .reading_tree_state
                .selected()
                .last() // unique leaf id
                .and_then(|key| readings.data.get_mut(key));

            let plot_open = data.is_some();
            let (chart, histogram, details, logs) = if let Some(data) = data {
                data.logs_from_store.update_if_needed(
                    log_store,
                    data.reading.clone(),
                    &mut self.history_length,
                );
                data.history_from_store.update_if_needed(
                    data_store,
                    data.reading.clone(),
                    &mut self.history_length,
                );
                data.percentiles_from_store.update_if_needed(
                    log_store,
                    data.reading.device(),
                    &mut self.history_length,
                );
                (
                    data.chart(&mut plot_buf),
                    data.histogram(),
                    Some(data.details()),
                    Some(data.logs()),
                )
            } else {
                (None, Vec::new(), None, None)
            };

            terminal.draw(|frame| {
                render::app(
                    frame,
                    self,
                    &readings.ground,
                    &affectors,
                    details,
                    chart,
                    logs,
                    &histogram,
                );
            })?;

            if self.reading_tree_state.selected().is_empty() {
                self.reading_tree_state.select_first();
            }

            if event::poll(Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    tracing::trace!("key pressed: {key:?}");
                    if key.kind == KeyEventKind::Press {
                        let res = self.handle_key_all_modes(key);
                        if let ShouldExit::Yes = res {
                            break;
                        }
                        let res = match self.input_mode {
                            InputMode::Normal => self.handle_key_normal_mode(key, plot_open),
                            InputMode::EditingBounds => self.handle_key_bounds_mode(key),
                        };

                        if let ShouldExit::Yes = res {
                            break;
                        }
                    }
                }
            };

            match rx.try_recv() {
                Ok(Update::Reading(reading)) => {
                    readings.add(reading);
                }
                Ok(Update::ReadingList(list)) => {
                    readings.populate_from_reading_list(list);
                }
                Ok(Update::DeviceList(list)) => {
                    readings.populate_from_device_list(list);
                }
                Ok(Update::Error(err)) => readings.add_error(err),
                _ => (),
            }
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        shutdown_tx.send(Ok(Update::Shutdown)).unwrap();
        Ok(())
    }

    fn handle_key_normal_mode(&mut self, key: KeyEvent, plot_open: bool) -> ShouldExit {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return ShouldExit::Yes;
            }
            KeyCode::Left => {
                self.active_tab = self.active_tab.swap();
            }
            KeyCode::Right => {
                self.active_tab = self.active_tab.swap();
            }
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
            _other => (),
        }
        ShouldExit::No
    }

    fn handle_key_bounds_mode(&mut self, key: KeyEvent) -> ShouldExit {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.history_length.exit_editing();
                self.input_mode = InputMode::Normal;
            }
            other => self.history_length.process(other),
        }
        ShouldExit::No
    }

    fn handle_key_all_modes(&mut self, key: KeyEvent) -> ShouldExit {
        match key.code {
            KeyCode::Down => {
                self.reading_tree_state.key_down();
            }
            KeyCode::Up => {
                self.reading_tree_state.key_up();
            }
            KeyCode::Enter => {
                self.reading_tree_state.toggle_selected();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return ShouldExit::Yes;
            }
            _other => (),
        }
        ShouldExit::No
    }
}

enum ShouldExit {
    Yes,
    No,
}
