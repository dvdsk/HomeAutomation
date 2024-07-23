use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::ListState,
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
enum ActiveList {
    #[default]
    Readings,
    Affectors,
}

impl ActiveList {
    fn style(&self, list: Self) -> Style {
        if *self == list {
            Style::default().fg(Color::Black)
        } else {
            Style::default().fg(Color::Indexed(242))
        }
    }

    fn swap(self) -> Self {
        match self {
            Self::Readings => Self::Affectors,
            Self::Affectors => Self::Readings,
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
) -> Result<(), std::io::Error> {
    App::default().run(rx, shutdown_tx, data_store)
}

#[derive(Default)]
struct App {
    input_mode: InputMode,
    active_list: ActiveList,
    show_histogram: bool,
    reading_list_state: TreeState<TreeKey>,
    history_length: HistoryLen,
}

impl App {
    pub fn run(
        &mut self,
        rx: mpsc::Receiver<Update>,
        shutdown_tx: mpsc::Sender<color_eyre::Result<Update>>,
        data_store: SocketAddr,
    ) -> Result<(), std::io::Error> {
        let mut readings = Readings {
            ground: Vec::new(),
            data: HashMap::new(),
        };

        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        let mut affectors_list_state = ListState::default();
        let affectors = vec![
            "placeholder 1".to_owned(),
            "placeholder 2".to_owned(),
            "placeholder 3".to_owned(),
        ];
        let mut plot_buf = Vec::new();

        loop {
            let data = self
                .reading_list_state
                .selected()
                .last() // unique leaf id
                .and_then(|key| readings.data.get_mut(key));

            let plot_open = data.is_some();
            let (chart, histogram) = if let Some(data) = data {
                data.stored_history.update_if_needed(
                    data_store,
                    data.reading.clone(),
                    &mut self.history_length,
                );
                (data.chart(&mut plot_buf), data.histogram())
            } else {
                (None, readings.histogram_all())
            };

            terminal.draw(|frame| {
                render::app(frame, self, &readings.ground, chart, &histogram);
            })?;

            if self.reading_list_state.selected().is_empty() {
                self.reading_list_state.select_first();
            }

            if event::poll(Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    tracing::trace!("key pressed: {key:?}");
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_all_modes(key);
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
            KeyCode::Char('q') => {
                return ShouldExit::Yes;
            }
            KeyCode::Left => {
                self.active_list = self.active_list.swap();
            }
            KeyCode::Right => {
                self.active_list = self.active_list.swap();
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
            _other => (),
        }
        ShouldExit::No
    }

    fn handle_key_bounds_mode(&mut self, key: KeyEvent) -> ShouldExit {
        match key.code {
            KeyCode::Esc => {
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
                self.reading_list_state.key_down();
            }
            KeyCode::Up => {
                self.reading_list_state.key_up();
            }
            KeyCode::Enter => {
                self.reading_list_state.toggle_selected();
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
