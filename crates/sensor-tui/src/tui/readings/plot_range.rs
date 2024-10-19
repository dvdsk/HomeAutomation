use crossterm::event::{KeyCode, KeyEvent};
use jiff::{SignedDuration, Timestamp};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::{
    ops::RangeInclusive,
    time::{Duration, Instant},
};

use crate::time::format::duration;
use crate::time::parse::parse_duration;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Empty,
    Invalid,
    Valid,
    Fetching(Instant),
    Fetched,
}

#[derive(Debug, Clone, Copy)]
pub enum Range {
    Relative(Duration),
    Window([jiff::Timestamp; 2]),
}

impl Range {
    pub(crate) fn is_relative(&self) -> bool {
        matches!(self, Range::Relative(_))
    }

    pub(crate) fn range_inclusive(&self) -> RangeInclusive<jiff::Timestamp> {
        match self {
            Range::Relative(dur) => {
                let dur = jiff::Span::try_from(dur.clone()).unwrap();
                let now = jiff::Timestamp::now();
                (now - dur)..=now
            }
            Range::Window([start, end]) => *start..=*end,
        }
    }
    pub(crate) fn duration(&self) -> Duration {
        match self {
            Range::Relative(dur) => dur.clone(),
            Range::Window([start, end]) => start.duration_until(*end).unsigned_abs(),
        }
    }
}

pub struct PlotRange {
    pub text_input: String,
    pub editing: bool,
    pub state: State,
    pub range: Range,
}

impl Default for PlotRange {
    fn default() -> Self {
        let dur = Duration::from_secs(15 * 60);
        let text_input = duration(dur.as_secs_f64());
        Self {
            text_input,
            editing: false,
            state: State::Empty,
            range: Range::Relative(dur),
        }
    }
}

impl PlotRange {
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
            self.range = Range::Relative(dur);
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
            text.push('_');
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

    pub(crate) fn label_bounds(&self) -> [f64; 2] {
        let bounds = match self.range {
            Range::Relative(dur) => [0f64, dur.as_secs_f64()],
            Range::Window([start, end]) => {
                let now = Timestamp::now();
                [
                    end.until(now)
                        .expect("now should be in the future")
                        .round(jiff::Unit::Second)
                        .expect("Timestamp.until(Timestamp) < Timestamp::Max")
                        .get_seconds() as f64,
                    start
                        .until(now)
                        .expect("now should be in the future")
                        .round(jiff::Unit::Second)
                        .expect("Timestamp.until(Timestamp) < Timestamp::Max")
                        .get_seconds() as f64,
                ]
            }
        };
        assert!(bounds[0] >= 0.0, "bounds must be >= zero: {bounds:?}");
        assert!(bounds[1] >= 0.0, "bounds must be >= zero: {bounds:?}");
        bounds
    }

    pub(crate) fn data_bounds(&self) -> [f64; 2] {
        let dur = match self.range {
            Range::Relative(dur) => dur.as_secs_f64(),
            Range::Window([start, end]) => start
                .until(end)
                .expect("end should be in the future")
                .total(jiff::Unit::Second)
                .expect("Timestamp.until(Timestamp) < Timestamp::Max"),
        };
        [0f64, dur]
    }

    pub(crate) fn change(&mut self, [mul_start, mul_end]: [f64; 2]) {
        let (dur, start) = match self.range {
            Range::Relative(dur) => {
                let start = Timestamp::now() - dur;
                (dur.as_secs_f64(), start)
            }
            Range::Window([start, end]) => {
                let dur = start
                    .until(end)
                    .expect("timestamps can be subtraced")
                    .round(jiff::Unit::Second)
                    .expect("Timestamp.until(Timestamp) < Timestamp::Max")
                    .get_seconds() as f64;
                (dur, start)
            }
        };
        let start = start + SignedDuration::from_secs_f64(dur * mul_start);
        let end = start + SignedDuration::from_secs_f64(dur * mul_end);
        let end = end.min(Timestamp::now());
        self.range = Range::Window([start, end]);
        self.state = State::Valid;
    }
}
