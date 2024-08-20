use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::time::{Duration, Instant};

use crate::time::format::fmt_seconds;
use crate::time::parse::parse_duration;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Empty,
    Invalid,
    Valid,
    Fetching(Instant),
    Fetched,
}

pub struct HistoryLen {
    pub text_input: String,
    pub editing: bool,
    pub state: State,
    pub dur: Duration,
}

impl Default for HistoryLen {
    fn default() -> Self {
        let dur = Duration::from_secs(15 * 60);
        let text_input = fmt_seconds(dur.as_secs_f64());
        Self {
            text_input,
            editing: false,
            state: State::Empty,
            dur,
        }
    }
}

impl HistoryLen {
    pub(crate) fn process(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char(c) => {
                self.text_input.push(c);
            }
            KeyCode::Backspace => {
                self.text_input.pop();
            }
            _other => return Some(key),
        }

        if let Ok(dur) = parse_duration(&self.text_input) {
            self.state = State::Valid;
            self.dur = dur;
        } else if self.text_input.is_empty() {
            self.state = State::Empty;
        } else {
            self.state = State::Invalid;
        }

        None
    }

    pub(crate) fn style_left_x_label(&self, org_label: Span<'static>) -> Span<'static> {
        let mut text = self.text_input.clone();
        if self.editing {
            text.push_str("_");
        }
        match self.state {
            State::Empty => {
                let style = Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::SLOW_BLINK);
                Span::raw(text).style(style)
            }
            State::Invalid => {
                let style = Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::ITALIC);
                Span::raw(text).style(style)
            }
            State::Valid => {
                let style = Style::default().add_modifier(Modifier::ITALIC);
                Span::raw(text).style(style)
            }
            State::Fetching(started) if started.elapsed() > Duration::from_secs(2) => {
                let style = Style::default().add_modifier(Modifier::ITALIC);
                text.push_str(" (fetching)");
                Span::raw(text).style(style)
            }
            State::Fetched | State::Fetching(_) => {
                if self.editing {
                    Span::raw(text)
                } else {
                    org_label
                }
            }
        }
    }

    pub(crate) fn exit_editing(&mut self) {
        self.editing = false;
        self.state = match self.state {
            State::Empty | State::Invalid => State::Fetched,
            State::Valid | State::Fetching(_) | State::Fetched => self.state,
        }
    }

    pub(crate) fn start_editing(&mut self) {
        self.editing = true;
        self.text_input.clear();
    }
}
