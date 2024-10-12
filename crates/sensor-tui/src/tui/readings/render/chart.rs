use protocol::reading;
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
use labels::{x_labels, y_labels};

/// mul then add to get original from points between 0 and 1
#[derive(Debug)]
struct Scaling {
    mul: f64,
    add: f64,
}

impl Scaling {
    fn to_normalize([min, max]: Bounds) -> Self {
        Self {
            mul: max - min,
            add: -min,
        }
    }
    fn normalized(&self, y: f64) -> f64 {
        (y + self.add) / self.mul
    }
}

pub fn render(frame: &mut Frame, layout: Rect, tab: &mut UiState, charts: &mut [ChartParts]) {
    let mut merged_y_bounds: Vec<(reading::Info, [f64; 2])> = Vec::new();
    for chart in charts.iter() {
        let y_bounds = ybounds(chart, layout);
        for (info, existing) in merged_y_bounds.iter_mut() {
            if info.unit == chart.reading.unit {
                existing[0] = existing[0].min(y_bounds[0]);
                existing[1] = existing[1].min(y_bounds[1]);
                continue;
            }
        }
        merged_y_bounds.push((chart.reading.clone(), y_bounds));
    }

    let mut labels_y = Vec::new();
    for new in merged_y_bounds
        .iter()
        .map(|(info, y_bounds)| y_labels(info, layout, *y_bounds))
    {
        if labels_y.is_empty() {
            labels_y = new;
        } else {
            for (existing, new) in labels_y.iter_mut().zip(new) {
                existing.push(' ');
                existing.push_str(new.as_str());
            }
        }
    }

    let datasets = charts
        .iter_mut()
        .flat_map(|chart| {
            let (_info, y_bounds) = merged_y_bounds
                .iter()
                .find(|(info, _)| info.unit == chart.reading.unit)
                .expect("made bound for every chart");
            let scaling = Scaling::to_normalize(*y_bounds);

            for (_, y) in chart.data.iter_mut() {
                *y = scaling.normalized(*y)
            }

            split::split(chart, layout.width).map(|line| {
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default())
                    .data(line)
            })
        })
        .collect();

    let x_bounds = [0f64, tab.history_length.dur.as_secs_f64()];
    let scale = labels::scale(x_bounds[1]);

    let mut labels_x = x_labels(layout, x_bounds, &scale);
    let borrowed = labels_x.first_mut().expect("min labels is 2");
    let owned = std::mem::take(borrowed);
    labels_x[0] = tab.history_length.style_left_x_label(owned);

    let x_axis = Axis::default()
        .title(format!("Time ({scale})",))
        .style(Style::default())
        .bounds(x_bounds)
        .labels(labels_x)
        .labels_alignment(Alignment::Center);

    let y_axis = Axis::default()
        .title("selected readings")
        .style(Style::default())
        .bounds([0.0, 1.0])
        .labels(labels_y);

    let linechart = Chart::new(datasets)
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, layout);
}

type Bounds = [f64; 2];
fn ybounds(chart: &ChartParts, layout: Rect) -> Bounds {
    let bounds = chart
        .data
        .iter()
        .map(|(_, y)| y)
        .fold([f64::MAX, f64::MIN], |[start, end], y| {
            [start.min(*y), end.max(*y)]
        });
    let y_range = bounds[1] - bounds[0];
    let margin = 5.0 / layout.height as f64;

    let range = &chart.reading.range;
    let resolution = chart.reading.resolution as f64;
    let y_margin = f64::max(y_range * margin, resolution / 2.0);
    let bounds = [bounds[0] - y_margin, bounds[1] + y_margin];
    [
        bounds[0].max(range.start as f64),
        bounds[1].min(range.end as f64),
    ]
}
