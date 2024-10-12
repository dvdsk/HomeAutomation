use std::time::Duration;

use protocol::reading;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::time::format::progressively_more_specified;

use super::Bounds;

const Y_LABEL_SPACING: u16 = 10;

pub fn y_labels(info: &reading::Info, layout: Rect, y_bounds: Bounds) -> Vec<String> {
    // Characters are about twice as high as wide
    evenly_spaced_ylabels(
        layout.height / Y_LABEL_SPACING,
        info.resolution as f64,
        y_bounds,
    )
    .map(|y| format!("{0:.1$}", y, info.precision()))
    .collect()
}

type Labels = Vec<Span<'static>>;
pub fn x_labels(
    layout: Rect,
    x_bounds: Bounds,
    scale: &progressively_more_specified::FmtScale,
) -> Labels {
    // Characters are about twice as high as wide
    let max_labels = layout.width / Y_LABEL_SPACING / 2;
    let n_labels = max_labels.max(2);
    let x_spacing = (x_bounds[1] - x_bounds[0]) / n_labels as f64;

    (0..=n_labels)
        .map(move |i| x_bounds[0] + x_spacing * i as f64)
        .rev()
        .map(|x| fmt(x, scale))
        .map(Into::into)
        .collect()
}

fn fmt(x: f64, scale: &progressively_more_specified::FmtScale) -> String {
    let now = jiff::Timestamp::now();
    let elapsed = Duration::from_secs_f64(x);
    let time = now - elapsed;
    scale.render(time, elapsed, "")
}

pub fn scale(secs: f64) -> progressively_more_specified::FmtScale {
    assert!(secs >= 0.0, "Argument must be larger then or equal to zero");
    let now = jiff::Timestamp::now();
    let elapsed = Duration::from_secs_f64(secs);
    let time = now - elapsed;

    progressively_more_specified::FmtScale::optimal_for(time, elapsed)
}

fn evenly_spaced_ylabels(
    max_labels: u16,
    resolution: f64,
    bounds: Bounds,
) -> impl DoubleEndedIterator<Item = f64> {
    let steps_in_data = (bounds[1] - bounds[0]) / resolution;

    let n_labels = max_labels.min(steps_in_data as u16).max(2);

    let y_spacing = (bounds[1] - bounds[0]) / n_labels as f64;
    (0..=n_labels).map(move |i| bounds[0] + y_spacing * i as f64)
}
