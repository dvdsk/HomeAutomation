use jiff::Timestamp;
use log_store::api::Percentile;
use ratatui::{
    self,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{self, Bar, BarChart, BarGroup, Block, Borders},
    Frame,
};
use tui_tree_widget::Tree;

use crate::tui::Theme;

use super::{
    sensor_info::{ChartParts, Details, ErrorDensity, LogList, Readings},
    UiState,
};

pub mod chart;
mod logs;
mod tree;

pub(crate) fn layout(
    frame: &mut Frame,
    layout: Rect,
    readings: &Readings,
    chart: bool,
    logs: bool,
    ui_state: &UiState,
) -> [Rect; 3] {
    let [list_constraint, graph_constraint] = if chart {
        let tree_height = 2 + readings
            .arena
            .iter()
            .filter(|node| node.is_removed())
            .count();
        let details_height = 9;
        if (frame.area().height as f32) / 3. > tree_height as f32 {
            [
                Constraint::Min(tree_height.max(details_height) as u16),
                Constraint::Percentage(100),
            ]
        } else {
            [Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]
        }
    } else if logs {
        [Constraint::Percentage(60), Constraint::Percentage(40)]
    } else {
        [Constraint::Percentage(100), Constraint::Percentage(0)]
    };

    let help_size = 1 + ui_state.show_complete_help as u16;
    Layout::vertical([
        list_constraint,
        graph_constraint,
        Constraint::Min(help_size),
    ])
    .flex(Flex::Legacy)
    .areas(layout)
}

pub(crate) fn footer(frame: &mut Frame, layout: Rect, app: &mut UiState, theme: &Theme) {
    let mut main = Vec::new();

    if app.history_length.editing {
        main.push("ESC or q: stop bound editing");
    } else {
        main.push("ESC or q: quit");
    }

    if app.reading_selected {
        if !app.history_length.editing {
            main.push("b: edit graph start");
        }

        if app.show_histogram {
            main.push("h: hide histogram");
        } else {
            main.push("h: show histogram");
        }

        if app.show_logs {
            main.push("l: hide logs");
        } else {
            main.push("l: show logs");
        }

        if app.show_complete_help {
            main.push("?: hide more help");
        } else {
            main.push("?: show more help");
        }
    }

    let layout = if app.show_complete_help {
        let [top, bottom] = Layout::vertical([Constraint::Min(1), Constraint::Min(1)])
            .flex(Flex::Legacy)
            .areas(layout);

        let mut extra = Vec::new();
        if app.reading_selected {
            extra.push("c: toggle compare");
            if app.show_cursor {
                extra.push("x: hide cursor");
            } else {
                extra.push("x: show cursor");
            }
        }
        let footer = extra.join("  ");
        let footer = Text::raw(footer)
            .alignment(Alignment::Center)
            .style(theme.bars);
        frame.render_widget(footer, bottom);
        top
    } else {
        layout
    };

    let footer = main.join("  ");
    let footer = Text::raw(footer)
        .alignment(Alignment::Center)
        .style(theme.bars);
    frame.render_widget(footer, layout)
}

pub(crate) fn graph_hist_logs(
    frame: &mut Frame,
    layout: Rect,
    app: &mut UiState,
    percentiles: &[Percentile],
    logs: Option<LogList>,
    mut chart: Vec<ChartParts>,
    theme: &Theme,
) {
    let num_elems = 1usize + app.show_histogram as usize + app.show_logs as usize;

    let mut constraints = [Constraint::Max(2); 3];
    if !chart.is_empty() {
        constraints[0] = Constraint::Fill(10);
    }
    if logs
        .as_ref()
        .is_some_and(|LogList { items, .. }| !items.is_empty())
        && app.show_logs
    {
        constraints[1] = Constraint::Fill(10);
    }
    if !percentiles.is_empty() && app.show_histogram {
        let idx = 1 + app.show_logs as usize;
        constraints[idx] = Constraint::Fill(10);
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&constraints[..num_elems])
        .flex(Flex::Legacy)
        .split(layout);

    let mut layout = layout.iter().cloned();

    if chart.is_empty() {
        centered_text(
            "No chart as there is no data",
            frame,
            layout.next().unwrap(),
            theme,
        )
    } else {
        chart::render(frame, layout.next().unwrap(), app, &mut chart);
    }

    if app.show_logs {
        logs::render(
            frame,
            layout.next().unwrap(),
            &mut app.logs_table_state,
            logs,
            theme,
        )
    }

    if app.show_histogram {
        if percentiles.is_empty() {
            centered_text(
                "No histogram as there is no timing information",
                frame,
                layout.next().unwrap(),
                theme,
            )
        } else {
            render_histogram(frame, layout.next().unwrap(), percentiles);
        }
    }
}

fn render_histogram(frame: &mut Frame, lower: Rect, percentiles: &[Percentile]) {
    let histogram = histogram_bars(percentiles);
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(&histogram))
        .bar_width(12);
    frame.render_widget(barchart, lower)
}

fn histogram_bars(percentiles: &[log_store::api::Percentile]) -> Vec<Bar<'static>> {
    percentiles
        .iter()
        .map(
            |log_store::api::Percentile {
                 bucket_ends,
                 percentile,
                 count_in_bucket,
             }| {
                Bar::default()
                    .value(*count_in_bucket)
                    .text_value(format!("p{percentile}: {}", count_in_bucket))
                    .label(Line::from(format!("..{}", bucket_ends)))
            },
        )
        .collect()
}

pub(crate) fn readings_and_details(
    frame: &mut Frame,
    layout: Rect,
    app: &mut UiState,
    readings: &Readings,
    details: Option<Details>,
) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(Flex::Legacy)
            .areas(layout);

    let tree = tree::build_ui(readings, &app.comparing);
    frame.render_stateful_widget(
        Tree::new(&tree)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Sensor readings")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        left,
        &mut app.tree_state,
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
            let time_ago = crate::time::format::duration(seconds_ago as f64);
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

fn centered_text(text: &str, frame: &mut Frame, area: Rect, theme: &Theme) {
    let footer = Text::raw(text)
        .alignment(Alignment::Center)
        .style(theme.centered_text);
    frame.render_widget(footer, area)
}
