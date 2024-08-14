use log_store::api::ErrorEvent;
use ratatui::layout::{Constraint, Flex, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Cell, HighlightSpacing, Row, Table, TableState};
use ratatui::Frame;

use super::centered_text;

pub fn render(
    frame: &mut Frame,
    layout: Rect,
    table_state: &mut TableState,
    logs: Option<Vec<ErrorEvent>>,
    theme: &super::Theme,
) {
    if let Some(logs) = logs {
        if logs.is_empty() {
            centered_text("Logs are empty", frame, layout, theme)
        } else {
            render_table(frame, layout, table_state, logs)
        }
    } else {
        centered_text("No logs", frame, layout, theme)
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
    let rows = logs
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, ErrorEvent { start, end, error })| {
            let color = match i % 2 {
                0 => Color::Gray,
                _ => Color::White,
            };
            let end = if let Some(timestamp) = end {
                format!("{}", timestamp.strftime("%I:%M:%S"))
            } else {
                "ongoing".to_owned()
            };
            let item = [
                format!("{error}"),
                format!("{}", start.strftime("%I:%M:%S")),
                end,
            ];
            item.into_iter()
                .map(|content| Text::from(content))
                .map(|content| Cell::from(content))
                .collect::<Row>()
                .style(Style::new().fg(Color::Black).bg(color))
                .height(1)
        });

    let bar = " █ ";
    let table = Table::new(
        rows,
        [
            Constraint::Max(longest_error_msg),
            Constraint::Max(10),
            Constraint::Max(10),
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
