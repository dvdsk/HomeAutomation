use jiff::Timestamp;
use log_store::api::ErrorEvent;
use ratatui::{
    self,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{self, Bar, BarChart, BarGroup, Block, Borders, Tabs},
    Frame,
};
use tracing::debug;
use tui_tree_widget::{Tree, TreeItem};

use super::{
    reading::{ChartParts, Details, ErrorDensity, TreeKey},
    ActiveTab, App,
};

mod chart;
mod logs;

pub(crate) fn app(
    frame: &mut Frame,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
    affectors: &[TreeItem<'static, TreeKey>],
    details: Option<Details>,
    chart: Option<ChartParts>,
    logs: Option<Vec<ErrorEvent>>,
    histogram: &[Bar],
) {
    let [list_constraint, graph_constraint] = if chart.is_some() {
        let tree_height = 2 + app.reading_tree_state.flatten(readings).len();
        let details_height = 9;
        if (frame.area().height as f32) / 3. > tree_height as f32 {
            [
                Constraint::Min(tree_height.max(details_height) as u16),
                Constraint::Percentage(100),
            ]
        } else {
            [Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]
        }
    } else {
        [Constraint::Percentage(100), Constraint::Percentage(100)]
    };

    let area = frame.area();
    let [top_line, top, bottom, footer] = Layout::vertical([
        Constraint::Min(1),
        list_constraint,
        graph_constraint,
        Constraint::Min(1),
    ])
    .flex(Flex::Legacy)
    .areas(area);

    render_tab(frame, top_line, app.active_tab);

    match app.active_tab {
        ActiveTab::Readings => {
            let have_details = details.is_some();
            render_readings_and_details(frame, top, app, readings, details);
            if have_details {
                render_graph_hist_logs(frame, bottom, app, histogram, logs, chart);
            }
        }
        ActiveTab::Affectors => render_affectors(frame, top, app, affectors),
    }
    render_footer(frame, footer, app);
}

fn render_tab(frame: &mut Frame, layout: Rect, active_tab: ActiveTab) {
    let tabs = Tabs::new(vec!["Readings", "Affectors"])
        .style(Style::new().bg(Color::Gray))
        .select(active_tab.number())
        .divider("|")
        .padding(" ", " ");

    frame.render_widget(tabs, layout);
}

fn render_footer(frame: &mut Frame, layout: Rect, app: &mut App) {
    let mut footer = Vec::new();

    if app.history_length.editing {
        footer.push("ESC or q: stop bound editing");
    } else {
        footer.push("ESC or q: quit");
        footer.push("b: edit graph start");
    }

    if app.show_histogram {
        footer.push("h: hide histogram");
    } else {
        footer.push("h: show histogram");
    }

    if app.show_logs {
        footer.push("l: hide logs");
    } else {
        footer.push("l: show logs");
    }

    let footer = footer.join("  ");
    let footer = Text::raw(footer)
        .alignment(Alignment::Center)
        .style(Style::new().bg(Color::Gray));
    frame.render_widget(footer, layout)
}

fn render_graph_hist_logs(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    histogram: &[Bar],
    logs: Option<Vec<ErrorEvent>>,
    chart: Option<ChartParts>,
) {
    let num_elems = 1u8 + app.show_histogram as u8 + app.show_logs as u8;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..num_elems).into_iter().map(|_| Constraint::Fill(10)))
        .flex(Flex::Legacy)
        .split(layout);

    let mut layout = layout.into_iter().cloned();

    if let Some(chart) = chart {
        chart::render(frame, layout.next().unwrap(), app, chart);
    } else {
        centered_text("No data", frame, layout.next().unwrap())
    }

    if app.show_logs {
        logs::render(
            frame,
            layout.next().unwrap(),
            &mut app.logs_table_state,
            logs,
        )
    }

    if app.show_histogram {
        if histogram.is_empty() {
            centered_text("No timing information", frame, layout.next().unwrap())
        } else {
            render_histogram(frame, layout.next().unwrap(), histogram);
        }
    }
}

fn render_histogram(frame: &mut Frame, lower: Rect, histogram: &[Bar]) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12);
    frame.render_widget(barchart, lower)
}
pub(crate) fn render_readings_and_details(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
    details: Option<Details>,
) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(Flex::Legacy)
            .areas(layout);

    frame.render_stateful_widget(
        Tree::new(readings)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Sensor readings")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        left,
        &mut app.reading_tree_state,
    );
    if let Some(details) = details {
        render_details(frame, right, details);
    }
}

fn render_details(frame: &mut Frame, layout: Rect, details: Details) {
    let Details {
        last_reading,
        condition,
        description,
        errors_since,
    } = details;
    let last_reading = match last_reading {
        None => "last read: Never".to_owned(),
        Some((ts, val)) => {
            let seconds_ago = Timestamp::now()
                .since(ts)
                .expect("should make sense")
                .get_seconds();
            let time_ago = crate::time::format::fmt_seconds(seconds_ago as f64);
            format!("last read: {time_ago} ago, value: {val}")
        }
    };

    let condition = match condition {
        Ok(()) => String::new(),
        Err(err) => format!("error: {err}\n"),
    };

    let ErrorDensity {
        t5_min,
        t15_min,
        t30_min,
        t45_min,
        t60_min,
    } = errors_since;
    let errors_since = format!("errors in the past:\n5min: {t5_min:.2}, 15min: {t15_min:.2}, 30min: {t30_min:.2}, 45min {t45_min:.2}, 60m: {t60_min:.2}");

    let text = format!("{description}\n{last_reading}\n{condition}{errors_since}");
    frame.render_widget(
        widgets::Paragraph::new(text)
            .block(Block::bordered().title("Details"))
            .wrap(widgets::Wrap { trim: true }),
        layout,
    )
}

fn render_affectors(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    affectors: &[TreeItem<'static, TreeKey>],
) {
    frame.render_stateful_widget(
        Tree::new(affectors)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Controllable affectors")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        layout,
        &mut app.affector_tree_state,
    );
}

fn centered_text(text: &str, frame: &mut Frame, area: Rect) {
    let footer = Text::raw(text)
        .alignment(Alignment::Center)
        .style(Style::new().bg(Color::Gray));
    frame.render_widget(footer, area)
}
