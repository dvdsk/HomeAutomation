use std::time::Duration;

use protocol::reading;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::time::format::progressively_more_specified::{self, FmtScale};
use crate::tui::readings::history_len::PlotRange;

use super::Bounds;

const Y_LABEL_SPACING: u16 = 10;

pub fn y_and_title(
    merged_y_bounds: &Vec<(reading::Info, [f64; 2])>,
    layout: Rect,
) -> (Vec<String>, String) {
    let mut title = String::new();
    let mut labels = Vec::new();
    for (info, y_bounds) in merged_y_bounds.iter() {
        title.push_str(&info.unit.to_string());
        title.push(' ');

        let new = y_labels(info, layout, *y_bounds);
        if labels.is_empty() {
            labels = new;
        } else {
            for (existing, new) in labels.iter_mut().zip(new) {
                existing.push(' ');
                existing.push_str(new.as_str());
            }
        }
    }
    (labels, title)
}

fn y_labels(info: &reading::Info, layout: Rect, y_bounds: Bounds) -> Vec<String> {
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
pub fn x_and_title(
    layout: Rect,
    x_bounds: Bounds,
    history_len: &PlotRange,
) -> (Labels, String, FmtScale) {
    // Characters are about twice as high as wide
    let max_labels = layout.width / Y_LABEL_SPACING / 2;
    let n_labels = max_labels.max(2);
    let x_spacing = (x_bounds[1] - x_bounds[0]) / n_labels as f64;

    let scale = scale(x_bounds[1]);
    let mut labels: Labels = (0..=n_labels)
        .map(move |i| x_bounds[0] + x_spacing * i as f64)
        .rev()
        .map(|x| fmt(x, &scale))
        .map(Into::into)
        .collect();

    let borrowed = labels.first_mut().expect("min labels is 2");
    let owned = std::mem::take(borrowed);
    labels[0] = history_len.style_left_x_label(owned);

    let title = format!("Time ({scale})");
    (labels, title, scale)
}

pub fn fmt(x: f64, scale: &progressively_more_specified::FmtScale) -> String {
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
