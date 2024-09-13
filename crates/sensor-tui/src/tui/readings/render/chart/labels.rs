use std::time::Duration;

use protocol::reading;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::tui::readings::sensor_info::ChartParts;

use super::Bounds;

type Labels = Vec<Span<'static>>;

pub fn labels<'a>(
    chart: &ChartParts,
    layout: Rect,
    x_bounds: Bounds,
    y_bounds: Bounds,
) -> (Labels, Labels) {
    let y_label_spacing = 10;
    let info = &chart.reading;

    // Characters are about twice as high as wide
    let x = evenly_spaced_labels(layout.width / y_label_spacing / 2, info, x_bounds)
        .rev()
        .map(fmt_seconds)
        .map(Into::into)
        .collect();
    let y = evenly_spaced_labels(layout.height / y_label_spacing, info, y_bounds)
        .map(|y| format!("{0:.1$}", y, info.precision()).into())
        .collect();

    (x, y)
}

fn fmt_seconds(secs: f64) -> String {
    if secs == 0.0 {
        "now".to_string()
    } else if secs > 2. * 60. * 60. && secs < 24. * 60. * 60. {
        fmt_hh_mm(secs)
    } else {
        "-".to_string() + &crate::time::format::fmt_seconds(secs)
    }
}

fn fmt_hh_mm(secs: f64) -> String {
    let now = jiff::Timestamp::now();
    let label = now - Duration::from_secs_f64(secs);
    label.strftime("%H:%M").to_string()
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
    (0..=n_labels)
        .into_iter()
        .map(move |i| bounds[0] + y_spacing * i as f64)
}
