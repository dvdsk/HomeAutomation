use protocol::reading_tree::Tree;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Frame,
};

use crate::tui::reading::ChartParts;

use super::App;

mod labels;
use labels::labels;

pub fn render(frame: &mut Frame, layout: Rect, app: &mut App, chart: ChartParts) {
    let dataset = Dataset::default()
        .name(chart.reading.name())
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default())
        .data(chart.data);

    let (x_bounds, y_bounds) = bounds(&chart);
    let (mut x_labels, y_labels) = labels(&chart, layout, x_bounds, y_bounds);

    let borrowed = x_labels.first_mut().expect("min labels is 2");
    let owned = std::mem::take(borrowed);
    x_labels[0] = app.history_length.style_left_x_label(owned);

    let x_axis = Axis::default()
        .title("Time")
        .style(Style::default())
        .bounds(x_bounds)
        .labels(x_labels)
        .labels_alignment(Alignment::Center);
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
