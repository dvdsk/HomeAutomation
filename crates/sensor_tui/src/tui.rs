use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::ListState,
};
use tui_tree_widget::TreeState;
use std::{
    collections::HashMap, io::stdout, sync::mpsc, time::{Duration, Instant}
};

mod reading;
use reading::Readings;
mod render;
use crate::Update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveList {
    Readings,
    Actuators,
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
            Self::Readings => Self::Actuators,
            Self::Actuators => Self::Readings,
        }
    }
}

struct ConnInfo {
    last_msg: Instant,
    addr: std::net::SocketAddr,
}

impl ConnInfo {
    fn conn_status(&self) -> String {
        let Self { last_msg, addr } = self;
        let elapsed = last_msg.elapsed().as_secs();
        if elapsed < 2 {
            format!("client: {addr}\nlast message less then 2 seconds ago")
        } else {
            format!("client: {addr}\nlast message {elapsed}s ago")
        }
    }
    fn got_msg(&mut self) {
        self.last_msg = Instant::now();
    }
}

pub fn run(rx: mpsc::Receiver<Update>) -> Result<(), std::io::Error> {
    let mut readings = Readings { ground: Vec::new(), data: HashMap::new() };
    let mut conn_info = None;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut active_list = ActiveList::Readings;
    let mut reading_list_state = TreeState::default();
    let mut actuator_list_state = ListState::default();
    let actuators = vec![
        "placeholder 1".to_owned(),
        "placeholder 2".to_owned(),
        "placeholder 3".to_owned(),
    ];
    let mut plot_buf = Vec::new();

    loop {
        let conn_status = conn_info
            .as_ref()
            .map(ConnInfo::conn_status)
            .unwrap_or_else(|| format!("no connected client"));

        let chart = if !reading_list_state.selected().is_empty() {
            readings.chart(reading_list_state.selected(), &mut plot_buf)
        } else {
            None
        };

        let histogram = if !reading_list_state.selected().is_empty() {
            let key = reading_list_state.selected().first().unwrap();
            readings.histogram(*key)
        } else {
            readings.histogram_all()
        };

        terminal.draw(|frame| {
            render::all(
                frame,
                &readings.ground,
                &mut reading_list_state,
                actuators.clone(),
                &mut actuator_list_state,
                active_list,
                &histogram,
                chart,
                &conn_status,
            );
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    if key.code == KeyCode::Left {
                        active_list = active_list.swap();
                    }
                    if key.code == KeyCode::Right {
                        active_list = active_list.swap();
                    }
                    if key.code == KeyCode::Down {
                        reading_list_state.key_down();
                    }
                    if key.code == KeyCode::Up {
                        reading_list_state.key_up();
                    }
                }
            }
        }

        match rx.try_recv() {
            Ok(Update::Reading(reading)) => {
                conn_info.as_mut().unwrap().got_msg();
                readings.add(reading);
            }
            Ok(Update::Error(err)) => {
                tracing::warn!("sensor error: {err:?}");
            }
            _ => (),
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
