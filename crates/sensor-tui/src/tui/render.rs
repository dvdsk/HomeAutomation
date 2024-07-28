use jiff::Timestamp;
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::Text,
    widgets::{self, Bar, BarChart, BarGroup, Block, Borders},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem};

use super::{
    reading::{ChartParts, Details, NumErrorSince, TreeKey},
    ActiveList, App,
};

mod chart;

pub(crate) fn app(
    frame: &mut Frame,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
    affectors: &[TreeItem<'static, TreeKey>],
    details: Option<Details>,
    chart: Option<ChartParts>,
    histogram: &[Bar],
) {
    let [list_constraint, graph_constraint] = if chart.is_some() {
        let list_height = 2 + app.reading_tree_state.flatten(readings).len();
        if (frame.size().height as f32) / 3. > list_height as f32 {
            [
                Constraint::Min(list_height as u16),
                Constraint::Percentage(100),
            ]
        } else {
            [Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]
        }
    } else {
        [Constraint::Percentage(100), Constraint::Percentage(100)]
    };

    let area = frame.size();
    let [top, bottom, footer] =
        Layout::vertical([list_constraint, graph_constraint, Constraint::Min(1)])
            .flex(Flex::Legacy)
            .areas(area);

    render_readings_and_actuators(frame, top, app, readings, affectors, details);
    render_graph_and_hist(frame, bottom, app, histogram, chart);
    render_footer(frame, footer, app);
}

fn render_footer(frame: &mut Frame, layout: Rect, app: &mut App) {
    let text = if app.history_length.editing {
        "ESC: stop bound editing"
    } else {
        if app.show_histogram {
            "b: edit graph start, h: hide histogram"
        } else {
            "b: edit graph start, h: show histogram"
        }
    };

    let text = Text::raw(text);
    frame.render_widget(text, layout)
}

fn render_graph_and_hist(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    histogram: &[Bar],
    chart: Option<ChartParts>,
) {
    match (chart, app.show_histogram) {
        (None, true) => render_histogram(frame, layout, histogram),
        (None, false) => (),
        (Some(chart), true) => {
            let [top, lower] =
                Layout::vertical([Constraint::Percentage(65), Constraint::Percentage(35)])
                    .flex(Flex::Legacy)
                    .areas(layout);
            chart::render(frame, top, app, chart);
            render_histogram(frame, lower, histogram);
        }
        (Some(chart), false) => {
            chart::render(frame, layout, app, chart);
        }
    }
}

fn render_histogram(frame: &mut Frame, lower: Rect, histogram: &[Bar]) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12);
    // .bar_style(Style::default())
    // .value_style(Style::default());
    frame.render_widget(barchart, lower)
}

pub(crate) fn render_readings_and_actuators(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
    affectors: &[TreeItem<'static, TreeKey>],
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
                    .borders(Borders::ALL)
                    .border_style(app.active_list.style(ActiveList::Readings)),
            )
            .style(app.active_list.style(ActiveList::Readings))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        left,
        &mut app.reading_tree_state,
    );

    if let (Some(details), ActiveList::Readings) = (details, app.active_list) {
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)])
                .flex(Flex::Legacy)
                .areas(right);
        render_affectors(frame, top, app, affectors);
        render_details(frame, bottom, details);
    } else {
        render_affectors(frame, right, app, affectors)
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

    let NumErrorSince {
        t5_min,
        t15_min,
        t30_min,
        t45_min,
        t60_min,
    } = errors_since;
    let errors_since = format!("errors in the past:\n5min: {t5_min}, 15min: {t15_min}, 30min: {t30_min}, 45min {t45_min}, 60m: {t60_min}");

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
                    .title("Controllable actuators")
                    .borders(Borders::ALL)
                    .border_style(app.active_list.style(ActiveList::Affectors)),
            )
            .style(app.active_list.style(ActiveList::Affectors))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        layout,
        &mut app.affector_tree_state,
    );
}
