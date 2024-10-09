use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Frame,
};

use crate::tui::readings::{sensor_info::ChartParts, UiState};

mod labels;
mod split;
use labels::labels;

pub fn render(frame: &mut Frame, layout: Rect, tab: &mut UiState, chart: ChartParts) {
    let (x_bounds, y_bounds) = bounds(&chart, layout);
    let scale = labels::scale(x_bounds[1]);
    let (mut x_labels, y_labels) = labels(&chart, layout, x_bounds, y_bounds, &scale);

    let borrowed = x_labels.first_mut().expect("min labels is 2");
    let owned = std::mem::take(borrowed);
    x_labels[0] = tab.history_length.style_left_x_label(owned);

    let x_axis = Axis::default()
        .title(format!("Time ({scale})",))
        .style(Style::default())
        .bounds(x_bounds)
        .labels(x_labels)
        .labels_alignment(Alignment::Center);
    let y_axis = Axis::default()
        .title(chart.reading.unit.to_string())
        .style(Style::default())
        .bounds(y_bounds)
        .labels(y_labels);

    let datasets = split::split(&chart, layout.width)
        .map(|line| {
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default())
                .data(line)
        })
        .collect();

    let linechart = Chart::new(datasets)
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, layout);
}

type Bounds = [f64; 2];
fn bounds(chart: &ChartParts, layout: Rect) -> (Bounds, Bounds) {
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
    let margin = 5.0 / layout.height as f64;

    let range = &chart.reading.range;
    let resolution = chart.reading.resolution as f64;
    let y_margin = f64::max(y_range * margin, resolution / 2.0);
    let y_bounds = [y_bounds[0] - y_margin, y_bounds[1] + y_margin];
    let y_bounds = [
        y_bounds[0].max(range.start as f64),
        y_bounds[1].min(range.end as f64),
    ];
    (x_bounds, y_bounds)
}
