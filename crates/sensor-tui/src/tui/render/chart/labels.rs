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
        .map(fmt_x_label)
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

struct FmtOption {
    unit: &'static str,
    factor: usize,
    /// at a max display this numb of items
    /// before going to the bigger FmtOption
    next: usize,
}

impl FmtOption {
    fn scale(&self, seconds: f64) -> usize {
        (seconds / self.factor as f64).floor() as usize
    }
}

fn fmt_x_label(secs: f64) -> String {
    #[rustfmt::skip]
    let mut options = [
        // next is purposefully set wrong, we set it in a loop below
        FmtOption{unit: "s", factor: 1, next: 0},
        FmtOption{unit: "m", factor: 60, next: 0},
        FmtOption{unit: "h", factor: 60 * 60, next: 0},
        FmtOption{unit: "d", factor: 60 * 60 * 24, next: 0},
        FmtOption{unit: "w", factor: 60 * 60 * 24 * 7, next: 0},
        FmtOption{unit: "y", factor: 60 * 60 * 24 * 365, next: 0},
    ];

    let mut next = usize::MAX;
    for fmt in options.iter_mut().rev() {
        fmt.next = next;
        next = fmt.factor;
    }

    let mut small = &options[0];
    for big in &options[1..] {
        if big.scale(secs) == 0 {
            return format!("{}{}", small.scale(secs), small.unit);
        }

        if secs >= big.next as f64 {
            small = big;
            continue;
        }

        if small.scale(secs % small.next as f64) == 0 {
            return format!("{}{}", big.scale(secs), big.unit);
        }

        return format!(
            "{}{}{}{}",
            big.scale(secs),
            big.unit,
            small.scale(secs % small.next as f64),
            small.unit
        );
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_x() {
        let cases = [
            (5, "5s"),
            (92, "1m32s"),
            (60 * 60 * 12, "12h"),
            (60 * 60 * 24 * 2, "2d"),
            (60 * 60 * 24 * 7, "1w"),
            (60 * 60 * 24 * 366, "1y"),
            (60 * 60 * 24 * 365 * 14 + 60 * 60 * 24 * 7 * 6, "14y6w"),
        ];

        for (input, correct_output) in cases {
            assert_eq!(fmt_x_label(input as f64), correct_output);
        }
    }
}
