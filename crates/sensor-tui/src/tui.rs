use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use protocol::Affector;
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
};
use render::render;
use std::{
    io::{stdout, Stdout},
    sync::mpsc,
    time::Duration,
};

use crate::{Fetch, Update, UserIntent};

mod affectors;
mod readings;
mod render;
mod reports;

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

pub(crate) struct Theme {
    bars: Style,
    centered_text: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bars: Style::new().bg(Color::Gray).fg(Color::Black),
            centered_text: Style::new(),
        }
    }
}

pub fn run(
    rx: mpsc::Receiver<Update>,
    shutdown_tx: mpsc::Sender<UserIntent>,
    control_tx: tokio::sync::mpsc::Sender<Affector>,
    fetcher: Fetch,
) -> Result<(), std::io::Error> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let res = App::default().run(terminal, rx, control_tx, fetcher);
    shutdown_tx.send(UserIntent::Shutdown).unwrap();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    res
}

#[derive(Default)]
struct App {
    theme: Theme,
    active_tab: ActiveTab,
    readings_tab: readings::Tab,
    affectors_tab: affectors::Tab,
    reports: reports::Reports,
}

impl App {
    pub fn run(
        &mut self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
        rx: mpsc::Receiver<Update>,
        mut control_tx: tokio::sync::mpsc::Sender<Affector>,
        mut fetcher: Fetch,
    ) -> Result<(), std::io::Error> {
        loop {
            terminal.draw(|frame| {
                let layout = render(frame, self);
                match self.active_tab {
                    ActiveTab::Readings => {
                        self.readings_tab
                            .render(&mut fetcher, frame, layout, &self.theme)
                    }
                    ActiveTab::Affectors => self.affectors_tab.render(frame, layout, &self.theme),
                }
            })?;

            if event::poll(Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    tracing::trace!("key pressed: {key:?}");
                    if key.kind == KeyEventKind::Press {
                        let res = match key.code {
                            KeyCode::Left => {
                                self.active_tab = self.active_tab.swap();
                                None
                            }
                            KeyCode::Right => {
                                self.active_tab = self.active_tab.swap();
                                None
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(());
                            }
                            _ => Some(key),
                        }
                        .and_then(|key| match self.active_tab {
                            ActiveTab::Readings => self.readings_tab.handle_key(key),
                            ActiveTab::Affectors => {
                                self.affectors_tab.handle_key(key, &mut control_tx)
                            }
                        })
                        .and_then(|key| self.reports.handle_key(key));

                        if let Some(unhandled_key) = res {
                            match unhandled_key.code {
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                _ => (),
                            }
                        }
                    }
                }
            }

            let Ok(update) = rx.try_recv() else {
                continue;
            };

            let Some(update) = self.register_errors(update) else {
                continue;
            };
            let Some(update) = self.affectors_tab.process_update(update) else {
                continue;
            };
            self.readings_tab.process_update(update);
        }
    }

    fn register_errors(&mut self, update: Update) -> Option<Update> {
        match update {
            Update::AffectorControlled { .. }
            | Update::AffectorList(_)
            | Update::DeviceList(_)
            | Update::Fetched { .. }
            | Update::ReadingList(_)
            | Update::SensorError(_)
            | Update::SensorReading(_) => return Some(update),
            Update::FetchError(e) => self.reports.add(e.wrap_err("Error fetching data")),
            Update::SubscribeError(e) => self
                .reports
                .add(e.wrap_err("Error while subscribing to data-server")),
        }
        None
    }
}
