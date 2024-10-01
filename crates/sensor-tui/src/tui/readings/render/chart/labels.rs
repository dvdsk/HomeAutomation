use std::time::Duration;

use protocol::reading;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::time::format::progressively_more_specified::{self, FmtScale};
use crate::tui::readings::sensor_info::ChartParts;

use super::Bounds;

type Labels = Vec<Span<'static>>;

pub fn labels(
    chart: &ChartParts,
    layout: Rect,
    x_bounds: Bounds,
    y_bounds: Bounds,
    scale: &progressively_more_specified::FmtScale
) -> (Labels, Labels) {
    let y_label_spacing = 10;
    let info = &chart.reading;


    // Characters are about twice as high as wide
    let x = evenly_spaced_labels(layout.width / y_label_spacing / 2, info, x_bounds)
        .rev()
        .map(|x| fmt(x, &scale))
        .map(Into::into)
        .collect();
    let y = evenly_spaced_labels(layout.height / y_label_spacing, info, y_bounds)
        .map(|y| format!("{0:.1$}", y, info.precision()).into())
        .collect();

    (x, y)
}

fn fmt(x: f64, scale: &progressively_more_specified::FmtScale) -> String {
    let now = jiff::Timestamp::now();
    let elapsed = Duration::from_secs_f64(x);
    let time = now - elapsed;
    scale.render(time, elapsed, "")
}

pub fn scale(secs: f64) -> progressively_more_specified::FmtScale {
    let now = jiff::Timestamp::now();
    let elapsed = Duration::from_secs_f64(secs);
    let time = now - elapsed;

    progressively_more_specified::FmtScale::optimal_for(time, elapsed)
}

fn evenly_spaced_labels(
    max_labels: u16,
    reading: &reading::Info,
    bounds: Bounds,
) -> impl DoubleEndedIterator<Item = f64> {
    let resolution = reading.resolution as f64;
    let steps_in_data = (bounds[1] - bounds[0]) / resolution;

    let n_labels = max_labels.min(steps_in_data as u16).max(2);

    let y_spacing = (bounds[1] - bounds[0]) / n_labels as f64;
    (0..=n_labels).map(move |i| bounds[0] + y_spacing * i as f64)
}
