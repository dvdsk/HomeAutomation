use std::iter;

use cursor::{CursorData, TooInaccurate};
use protocol::reading;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Text},
    widgets::{Axis, Block, Chart, Clear, Dataset, GraphType},
    Frame,
};

use crate::{
    time::format::progressively_more_specified::FmtScale,
    tui::readings::{sensor_info::ChartParts, UiState},
};

mod cursor;
mod labels;
mod split;

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

// excludes:
// - light colors: same as the normal colors on some terminals
// - gray: to light to see
// - magenta: looks the same as red
const ALL_COLORS: &[Color] = &[
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Cyan,
    Color::DarkGray,
];

fn plot_colors(unit: protocol::Unit) -> impl Iterator<Item = Color> {
    use protocol::Unit as U;
    use Color::*;

    let preferred = match unit {
        U::Pa => &[Black, Cyan, Blue] as &'static [Color],
        U::C => &[Red] as &'static [Color],
        U::RH => &[Blue, Cyan] as &'static [Color],
        U::Lux => &[Yellow] as &'static [Color],
        U::Ohm => &[DarkGray] as &'static [Color],
        U::Ppm | U::MicrogramPerM3 | U::NumberPerCm3 | U::NanoMeter => {
            &[Green, Cyan] as &'static [Color]
        }
        U::None => &[] as &'static [Color],
        U::RelativePower => &[Red] as &'static [Color],
    };
    preferred.iter().chain(ALL_COLORS.iter()).copied()
}

pub fn render(frame: &mut Frame, layout: Rect, ui: &mut UiState, charts: &mut [ChartParts]) {
    let (colors, merged_y_bounds) = bounds_and_colors(charts, layout);
    let (y_labels, y_title) = labels::y_and_title(&merged_y_bounds, layout);

    let (x_labels, x_title, scale) = labels::x_and_title(layout, &ui.plot_range);
    let data_bounds = ui.plot_range.data_bounds();

    let x_axis = Axis::default()
        .title(x_title)
        .style(Style::default())
        .bounds(data_bounds)
        .labels(x_labels)
        .labels_alignment(Alignment::Center);

    let y_axis_width = y_labels.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
    let y_axis = Axis::default()
        .title(y_title)
        .style(Style::default())
        .bounds([0.0, 1.0])
        .labels(y_labels);

    let chart_width = layout.width - y_axis_width - 3;
    let cursor = if let Some(steps) = ui.chart_cursor.get(chart_width) {
        Some(CursorData::new(steps, data_bounds, charts, chart_width))
    } else {
        None
    };

    let datasets = datasets(charts, &colors, merged_y_bounds, layout);
    let linechart = Chart::new(datasets)
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, layout);

    if charts.len() > 1 || cursor.is_some() {
        render_legend(layout, frame, charts, colors, cursor, scale);
    }
    if let Some(steps) = ui.chart_cursor.get(chart_width) {
        cursor::render(layout, frame, steps, y_axis_width);
    }
}

fn datasets<'a>(
    charts: &'a mut [ChartParts],
    colors: &Vec<Color>,
    mut merged_y_bounds: Vec<(reading::Info, [f64; 2])>,
    layout: Rect,
) -> Vec<Dataset<'a>> {
    let datasets = charts
        .iter_mut()
        .zip(colors.iter())
        .flat_map(|(chart, color)| {
            let (_, y_bounds) = merged_y_bounds
                .iter_mut()
                .find(|(info, _)| info.unit == chart.info.unit)
                .expect("made bound for every chart");
            let scaling = Scaling::to_normalize(*y_bounds);

            for (_, y) in chart.data.iter_mut() {
                *y = scaling.normalized(*y)
            }

            let color = color.clone();
            split::split(chart, layout.width).map(move |line| {
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color))
                    .data(line)
            })
        })
        .collect();
    datasets
}

fn bounds_and_colors(
    charts: &mut [ChartParts],
    layout: Rect,
) -> (Vec<Color>, Vec<(reading::Info, [f64; 2])>) {
    let mut colors: Vec<Color> = Vec::new();
    // same order as charts
    let mut merged_y_bounds: Vec<(reading::Info, [f64; 2])> = Vec::new();
    'outer: for chart in charts.iter() {
        let color = plot_colors(chart.info.unit)
            .find(|color| !colors.contains(&color))
            .clone()
            .unwrap_or(Color::Black);
        colors.push(color);
        let y_bounds = ybounds(chart, layout);

        for (info, existing) in merged_y_bounds.iter_mut() {
            if info.unit == chart.info.unit {
                existing[0] = existing[0].min(y_bounds[0]);
                existing[1] = existing[1].max(y_bounds[1]);
                continue 'outer;
            }
        }
        merged_y_bounds.push((chart.info.clone(), y_bounds));
    }
    (colors, merged_y_bounds)
}

fn render_legend(
    layout: Rect,
    frame: &mut Frame,
    charts: &mut [ChartParts],
    colors: Vec<Color>,
    cursor: Option<CursorData>,
    scale: FmtScale,
) {
    let (values, cursor_label) = if let Some(CursorData { pos, values }) = cursor {
        let renderd = labels::fmt(pos, &scale);
        let label = format!("cursor: {} {}", renderd, scale);
        let label = Line::styled(label, Style::default().bg(Color::Gray));
        (values, Some(label))
    } else {
        (Vec::new(), None)
    };
    let values = values.into_iter().map(Some).chain(iter::repeat(None));

    let text: Text = charts
        .iter()
        .map(|ChartParts { reading, .. }| reading)
        .zip(colors)
        .zip(values)
        .map(fmt_reading)
        .chain(cursor_label)
        .collect();

    let area = legend_area(&text, layout);
    frame.render_widget(Clear, area);
    frame.render_widget(text, area);
}

fn legend_area(text: &Text, chart: Rect) -> Rect {
    let lines = text.iter().count() as u16;
    let columns = text
        .iter()
        .map(|l| l.iter().map(|s| s.content.len() as u16).sum::<u16>())
        .max()
        .expect("legend is never empty");

    Rect {
        x: chart.right() - columns - 2,
        y: chart.top() + 1,
        width: columns,
        height: lines,
    }
}

fn fmt_reading(
    ((reading, color), cursor_value): (
        (&protocol::Reading, Color),
        Option<Result<f64, cursor::TooInaccurate>>,
    ),
) -> Line<'_> {
    use protocol::reading::tree::{Item, Tree};
    let mut node = reading as &dyn Tree;
    let mut text = node.name();

    loop {
        match node.inner() {
            Item::Node(inner) => node = inner,
            Item::Leaf(_) => break,
        }
        text.push('/');
        text.push_str(&node.name());
    }
    match cursor_value {
        Some(Ok(v)) => text.push_str(&format!(": {0:.1$}", v, reading.leaf().precision())),
        Some(Err(TooInaccurate)) => text.push_str(" x"),
        None => (),
    }
    Line::styled(text, Style::default().fg(color).bg(Color::Gray))
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

    let range = &chart.info.range;
    let resolution = chart.info.resolution as f64;
    let y_margin = f64::max(y_range * margin, resolution / 2.0);
    let bounds = [bounds[0] - y_margin, bounds[1] + y_margin];
    [
        bounds[0].max(*range.start() as f64),
        bounds[1].min(*range.end() as f64),
    ]
}
