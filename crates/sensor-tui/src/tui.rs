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
use std::{collections::HashMap, io::stdout, sync::mpsc, time::Duration};
use tui_tree_widget::TreeState;

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

    // fn swap(self) -> Self {
    //     match self {
    //         Self::Readings => Self::Actuators,
    //         Self::Actuators => Self::Readings,
    //     }
    // }
}

pub fn run(rx: mpsc::Receiver<Update>) -> Result<(), std::io::Error> {
    let mut readings = Readings {
        ground: Vec::new(),
        data: HashMap::new(),
    };

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let active_list = ActiveList::Readings;
    let mut reading_list_state = TreeState::default();
    let mut actuator_list_state = ListState::default();
    let actuators = vec![
        "placeholder 1".to_owned(),
        "placeholder 2".to_owned(),
        "placeholder 3".to_owned(),
    ];
    let mut plot_buf = Vec::new();

    loop {
        let data = reading_list_state
            .selected()
            .first()
            .map(|key| readings.data.get(key))
            .flatten();

        let (chart, histogram) = if let Some(data) = data {
            (data.chart(&mut plot_buf), data.histogram())
        } else {
            (None, readings.histogram_all())
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
            );
        })?;

        if reading_list_state.selected().is_empty() {
            reading_list_state.select_first();
        }

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                tracing::trace!("key pressed: {key:?}");
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
                        reading_list_state.key_left();
                        // active_list = active_list.swap();
                    }
                    if key.code == KeyCode::Right {
                        reading_list_state.key_right();
                        // active_list = active_list.swap();
                    }
                    if key.code == KeyCode::Down {
                        reading_list_state.key_down();
                    }
                    if key.code == KeyCode::Up {
                        reading_list_state.key_up();
                    }
                    // if key.code == KeyCode::Enter {
                    // reading_list_state.
                    // }
                }
            }
        }

        match rx.try_recv() {
            Ok(Update::Reading(reading)) => {
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
