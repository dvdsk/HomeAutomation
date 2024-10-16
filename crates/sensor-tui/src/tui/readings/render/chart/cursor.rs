use itertools::Itertools;
use ratatui::prelude::Rect;
use ratatui::text::Span;
use ratatui::widgets::Clear;

use crate::tui::readings::sensor_info::ChartParts;

#[derive(Debug, Clone, Copy)]
pub struct TooInaccurate;

pub struct CursorData {
    pub pos: f64,
    /// None means too far from cursor pos to be accurate
    pub values: Vec<Result<f64, TooInaccurate>>,
}
impl CursorData {
    pub(crate) fn new(
        steps: u16,
        x_bounds: [f64; 2],
        charts: &mut [ChartParts],
        chart_width: u16,
    ) -> Self {
        let pos = resolve_pos(steps, x_bounds, chart_width).clamp(0.0, f64::MAX);

        // chart data has x axis reversed correct
        // for that here
        let flipped_pos = x_bounds[1] - pos;
        // one character
        let max_deviation = (x_bounds[1] - x_bounds[0]) / chart_width as f64;
        let values = find_values(flipped_pos, max_deviation, charts);
        CursorData { pos, values }
    }
}

/// # panics
/// if x is not >= ax and <= db
fn interpolate(a: (f64, f64), b: (f64, f64), x: f64) -> f64 {
    assert!(x >= a.0);
    assert!(x <= b.0);

    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let dydx = dy / dx;
    let delta_x = x - a.0;

    a.1 + delta_x * dydx
}

/// do not interpolate large gaps
fn maybe_interpolate(
    a: (f64, f64),
    b: (f64, f64),
    x: f64,
    max_deviation: f64,
) -> Result<f64, TooInaccurate> {
    if b.0 - a.0 > max_deviation {
        if b.0 - x < max_deviation {
            Ok(b.1)
        } else if x - a.0 < max_deviation {
            Ok(a.1)
        } else {
            Err(TooInaccurate)
        }
    } else {
        Ok(interpolate(a, b, x))
    }
}

fn find_values(
    pos: f64,
    max_deviation: f64,
    charts: &mut [ChartParts],
) -> Vec<Result<f64, TooInaccurate>> {
    charts
        .iter()
        .map(|ChartParts { data, .. }| data)
        .map(|data| {
            let mut left = None;
            let mut right = None;
            for (x, y) in data.iter() {
                if *x < pos {
                    left = Some((*x, *y));
                } else {
                    right = Some((*x, *y));
                    break;
                }
            }
            (left, right)
        })
        .map(|(left, right)| match (left, right) {
            (None, None) => Err(TooInaccurate),
            (None, Some((x, y))) | (Some((x, y)), None) => {
                if (pos - x).abs() <= max_deviation {
                    Ok(y)
                } else {
                    Err(TooInaccurate)
                }
            }
            (Some(a), Some(b)) => maybe_interpolate(a, b, pos, max_deviation),
        })
        .collect()
}

pub(crate) fn resolve_pos(steps: u16, x_bounds: [f64; 2], chart_width: u16) -> f64 {
    let bounds_width = x_bounds[1] - x_bounds[0];
    let dist = bounds_width / chart_width as f64;
    x_bounds[1] - dist * steps as f64
}

pub(crate) fn render(chart: Rect, frame: &mut ratatui::Frame, steps: u16, axis_label_width: u16) {
    let left = chart
        .left()
        .saturating_add(steps)
        .saturating_add(axis_label_width)
        .saturating_add(2);
    let area = Rect {
        x: left.clamp(chart.x, chart.x + chart.width - 2),
        y: chart.bottom() - 3,
        width: 1,
        height: 1,
    };
    frame.render_widget(Clear, area);
    frame.render_widget(Span::raw("*"), area);
}
