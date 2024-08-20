use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
};
use render::render_tab;
use std::{
    io::{stdout, Stdout},
    sync::mpsc,
    time::Duration,
};

use crate::{Fetch, Update, UserIntent};

mod affectors;
mod readings;
mod render;

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
    fetcher: Fetch,
) -> Result<(), std::io::Error> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let res = App::default().run(terminal, rx, fetcher);
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
}

impl App {
    pub fn run(
        &mut self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
        rx: mpsc::Receiver<Update>,
        mut fetcher: Fetch,
    ) -> Result<(), std::io::Error> {
        loop {
            terminal.draw(|frame| {
                let layout = render_tab(frame, self);
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
                        match key.code {
                            KeyCode::Left => {
                                self.active_tab = self.active_tab.swap();
                            }
                            KeyCode::Right => {
                                self.active_tab = self.active_tab.swap();
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(());
                            }
                            _ => (),
                        }

                        let res = match self.active_tab {
                            ActiveTab::Readings => self.readings_tab.handle_key(key),
                            ActiveTab::Affectors => self.affectors_tab.handle_key(key),
                        };
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

            self.affectors_tab.process_update(&update);
            self.readings_tab.process_update(update);
        }
    }
}
