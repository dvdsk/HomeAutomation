use protocol::reading_tree::Tree as _;
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem};

use super::{
    reading::{ChartParts, TreeKey},
    ActiveList,
};

pub(crate) fn app(
    frame: &mut Frame,
    app: &mut super::App,
    readings: &[TreeItem<'static, TreeKey>],
    chart: Option<ChartParts>,
    histogram: &[Bar],
) {
    let area = frame.size();
    let [top, bottom] = Layout::vertical([Constraint::Min(10), Constraint::Min(10)])
        .flex(Flex::Legacy)
        .areas(area);

    render_readings_and_actuators(frame, top, app, readings);
    render_graphs(frame, bottom, app, histogram, chart);
}

fn render_graphs(
    frame: &mut Frame,
    layout: Rect,
    app: &mut super::App,
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
            render_chart(frame, top, app, chart);
            render_histogram(frame, lower, histogram);
        }
        (Some(chart), false) => {
            render_chart(frame, layout, app, chart);
        }
    }
}

fn render_histogram(frame: &mut Frame, lower: Rect, histogram: &[Bar]) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12)
        .bar_style(Style::default())
        .value_style(Style::default());
    frame.render_widget(barchart, lower)
}

type Bounds = [f64; 2];
fn bounds(chart: &ChartParts) -> (Bounds, Bounds) {
    let x_bounds = [
        0f64,
        chart.data.last().map(|(x, _)| x).copied().unwrap_or(0f64),
    ];

    let y_bounds = chart
        .data
        .iter()
        .map(|(_, y)| y)
        .fold([f64::MAX, f64::MIN], |[start, end], y| {
            [start.min(*y), end.max(*y)]
        });
    let y_range = y_bounds[1] - y_bounds[0];
    let y_margin = f64::max(y_range * 0.5, 0.001 * y_bounds[0].abs());
    let y_bounds = [y_bounds[0] - y_margin, y_bounds[1] + y_margin];
    (x_bounds, y_bounds)
}

type Labels<'a> = Vec<Span<'a>>;
fn labels<'a>(
    app: &'a mut super::App,
    chart: &ChartParts,
    x: Bounds,
    y: Bounds,
) -> (Labels<'a>, Labels<'a>) {
    let left_x_label = app.history_length.render_x_label(x[1]);
    let x_labels = vec![left_x_label, "0".into()];

    let y_labels = vec![
        format!("{0:.1$}", y[0], chart.reading.leaf().precision()).into(),
        format!("{0:.1$}", y[1], chart.reading.leaf().precision()).into(),
    ];
    (x_labels, y_labels)
}

fn render_chart(frame: &mut Frame, layout: Rect, app: &mut super::App, chart: ChartParts) {
    let dataset = Dataset::default()
        .name(chart.reading.name())
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default())
        .data(chart.data);

    let (x_bounds, y_bounds) = bounds(&chart);
    let (x_labels, y_labels) = labels(app, &chart, x_bounds, y_bounds);

    let x_axis = Axis::default()
        .title("Time")
        .style(Style::default())
        .bounds(x_bounds)
        .labels(x_labels);
    let y_axis = Axis::default()
        .title(chart.reading.leaf().unit.to_string())
        .style(Style::default())
        .bounds(y_bounds)
        .labels(y_labels);
    let linechart = Chart::new(vec![dataset])
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, layout);
}

pub(crate) fn render_readings_and_actuators(
    frame: &mut Frame,
    layout: Rect,
    app: &mut super::App,
    readings: &[TreeItem<'static, TreeKey>],
) {
    let horizontal: [_; 2] =
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
        horizontal[0],
        &mut app.reading_list_state,
    );

    // frame.render_stateful_widget(
    //     List::new(app.actuators)
    //         .block(
    //             Block::default()
    //                 .title("Actuators")
    //                 .borders(Borders::ALL)
    //                 .border_style(app.active_list.style(ActiveList::Actuators)),
    //         )
    //         .style(app.active_list.style(ActiveList::Actuators))
    //         .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
    //         .highlight_symbol(">>")
    //         .repeat_highlight_symbol(false)
    //         .direction(ratatui::widgets::ListDirection::TopToBottom),
    //     horizontal[1],
    //     app.actuator_list_state,
    // );
}
