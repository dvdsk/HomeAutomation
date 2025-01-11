use std::collections::{HashSet, VecDeque};

use color_eyre::eyre;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Report {
    short: String,
    detailed: String,
}

impl From<eyre::Report> for Report {
    fn from(value: eyre::Report) -> Self {
        let detailed = format!("{value:?}");
        let mut detailed = strip_ansi_escapes::strip_str(&detailed);
        let last_paragraph = detailed
            .find("Location:")
            .expect("Detailed report always lists a location");
        let backtrace_start = detailed[last_paragraph..]
            .find("\n\n")
            .map(|pos| pos + last_paragraph);

        if let Some(pos) = backtrace_start {
            detailed.truncate(pos);
        }

        Self {
            short: value.to_string(),
            detailed,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Reports {
    list: VecDeque<Report>,
    current: Option<Report>,
    to_ignore: HashSet<Report>,
    show_details: bool,
}

impl Reports {
    pub(super) fn add(&mut self, raw_report: eyre::Report) {
        let report = Report::from(raw_report);
        tracing::debug!("report: {report:?}");
        if self.to_ignore.contains(&report) {
            return;
        }

        if let Some(prev) = self.current.take() {
            self.list.push_front(prev);
        }
        self.current = Some(report);
    }

    pub(super) fn render(&self, frame: &mut Frame, layout: Rect) {
        let current = self.current.as_ref().expect(
            "should only be called when current is Some/needed lines > 0",
        );

        let [hint, error] =
            Layout::vertical([Constraint::Max(1), Constraint::Min(1)])
                .areas(layout);

        let error_style = Style::default().on_light_red().fg(Color::White);

        if self.show_details {
            frame.render_widget(
                Text::raw("e: hide details, x: close this error, i: ignore this error")
                    .alignment(Alignment::Center)
                    .style(error_style),
                hint,
            );
            frame.render_widget(
                Paragraph::new(current.detailed.as_str())
                    .wrap(Wrap { trim: false })
                    .style(error_style),
                error,
            );
        } else {
            frame.render_widget(
                Text::raw("e: show details, x: close this error, i: ignore this error")
                    .alignment(Alignment::Center)
                    .style(error_style),
                hint,
            );
            frame.render_widget(
                Text::raw(&current.short).style(error_style),
                error,
            );
        }
    }

    pub(super) fn needed_lines(&self) -> Option<u16> {
        let current = self.current.as_ref()?;

        Some(if self.show_details {
            1 + current.detailed.lines().count() as u16
        } else {
            1 + current.short.lines().count() as u16
        })
    }

    fn close_current(&mut self) {
        let Some(current) = self.current.take() else {
            return;
        };

        self.list.retain(|report| *report != current);
        self.current = self.list.pop_front();
    }

    fn ignore_current(&mut self) {
        let Some(current) = self.current.take() else {
            return; // may have closed
        };

        self.list.retain(|report| *report != current);

        self.to_ignore.insert(current);
        self.close_current()
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('e') => {
                self.show_details = !self.show_details;
            }
            KeyCode::Char('x') => {
                self.close_current();
            }
            KeyCode::Char('i') => {
                self.ignore_current();
            }
            _ => return Some(key),
        }
        None
    }
}
