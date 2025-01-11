use std::ops::Bound;

use jiff::tz::TimeZone;
use jiff::Zoned;
use log_store::api::ErrorEvent;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Cell, HighlightSpacing, Row, Table, TableState};
use ratatui::Frame;

use crate::time::format::progressively_more_specified::FmtScale;
use crate::tui::readings::sensor_info::{LogList, LogSource};

use super::centered_text;

pub fn render_log_range((start, stop): (Bound<jiff::Timestamp>, Bound<jiff::Timestamp>)) -> String {
    let start = match start {
        Bound::Included(start) | Bound::Excluded(start) => {
            let elapsed = start.duration_until(jiff::Timestamp::now()).unsigned_abs();
            FmtScale::optimal_for(start, elapsed).render(start, elapsed, " ago")
        }
        Bound::Unbounded => unreachable!("unbounded not allowed for log range start"),
    };
    let stop = match stop {
        Bound::Included(stop) | Bound::Excluded(stop) => {
            let elapsed = stop.duration_until(jiff::Timestamp::now()).unsigned_abs();
            FmtScale::optimal_for(stop, elapsed).render(stop, elapsed, " ago")
        }
        Bound::Unbounded => "now".to_owned(),
    };

    format!("{start} and {stop}")
}

pub fn render(
    frame: &mut Frame,
    layout: Rect,
    table_state: &mut TableState,
    logs: Option<LogList>,
    theme: &super::Theme,
) {
    match logs {
        Some(LogList {
            items,
            source: LogSource::Store,
            covers,
        }) => {
            if items.is_empty() {
                let text = format!("No logs between {}", render_log_range(covers));
                centered_text(&text, frame, layout, theme)
            } else {
                render_table(frame, layout, table_state, items)
            }
        }
        Some(LogList {
            items,
            source: LogSource::Local,
            covers,
        }) => {
            let [status, layout] = Layout::new(
                Direction::Vertical,
                [Constraint::Max(1), Constraint::Fill(1)],
            )
            .areas(layout);

            centered_text(
                "Loading additional logs from store ..",
                frame,
                status,
                theme,
            );
            if items.is_empty() {
                let text = format!("No local logs between {}", render_log_range(covers));
                centered_text(&text, frame, layout, theme)
            } else {
                render_table(frame, layout, table_state, items);
            }
        }
        None => centered_text("This item can not have logs", frame, layout, theme),
    }
}

pub fn render_table(
    frame: &mut Frame,
    layout: Rect,
    table_state: &mut TableState,
    logs: Vec<ErrorEvent>,
) {
    let header_style = Style::default().fg(Color::Black).bg(Color::Gray);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::White);

    let header = ["Error", "Started at", "Cleared at"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let longest_error_msg = logs
        .iter()
        .map(|ErrorEvent { error, .. }| error.to_string().chars().count() as u16)
        .max()
        .unwrap_or_default();

    let now = Zoned::now();
    let rows = logs
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, ErrorEvent { start, end, error })| {
            let color = match i % 2 {
                0 => Color::Gray,
                _ => Color::White,
            };

            let start = start.to_zoned(TimeZone::system());
            let start = if start.day() == now.day() && start.year() == now.year() {
                format!("{}", start.strftime("%H:%M:%S"))
            } else {
                format!("{}", start.strftime("%D %H:%M:%S"))
            };

            let end = if let Some(end) = end {
                let end = end.to_zoned(TimeZone::system());
                if end.day() == now.day() && end.year() == now.year() {
                    format!("{}", end.strftime("%H:%M:%S"))
                } else {
                    format!("{}", end.strftime("%D %H:%M:%S"))
                }
            } else {
                "ongoing".to_owned()
            };
            let item = [format!("{error}"), start, end];
            item.into_iter()
                .map(Text::from)
                .map(Cell::from)
                .collect::<Row>()
                .style(Style::new().fg(Color::Black).bg(color))
                .height(1)
        });

    let bar = " █ ";
    let table = Table::new(
        rows,
        [
            Constraint::Max(longest_error_msg),
            Constraint::Max(18),
            Constraint::Max(18),
        ],
    )
    .header(header)
    .flex(Flex::SpaceAround)
    .column_spacing(1)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(Color::White)
    .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(table, layout, table_state);
}
