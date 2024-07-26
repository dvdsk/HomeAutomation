use protocol::reading_tree::{ReadingInfo, Tree};
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::tui::reading::ChartParts;

use super::Bounds;

type Labels = Vec<Span<'static>>;

pub fn labels<'a>(
    chart: &ChartParts,
    layout: Rect,
    x_bounds: Bounds,
    y_bounds: Bounds,
) -> (Labels, Labels) {
    let y_label_spacing = 10;
    let info = chart.reading.leaf();

    // characters are about twice as high as wide
    let x = evenly_spaced_labels(layout.width / y_label_spacing / 2, &info, x_bounds)
        .rev()
        .map(crate::time::format::fmt_seconds)
        .map(Into::into)
        .collect();
    let y = evenly_spaced_labels(layout.height / y_label_spacing, &info, y_bounds)
        .map(|y| format!("{0:.1$}", y, info.precision()).into())
        .collect();

    (x, y)
}

fn evenly_spaced_labels(
    max_labels: u16,
    reading: &ReadingInfo,
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
