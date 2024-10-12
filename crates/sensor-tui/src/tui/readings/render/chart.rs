use protocol::reading;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Text},
    widgets::{Axis, Block, Chart, Clear, Dataset, GraphType},
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
    };
    preferred.iter().chain(ALL_COLORS.iter()).copied()
}

pub fn render(frame: &mut Frame, layout: Rect, tab: &mut UiState, charts: &mut [ChartParts]) {
    let mut colors: Vec<Color> = Vec::new(); // same order as charts
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

    let mut y_axis_title = String::new();
    let mut labels_y = Vec::new();
    for (info, y_bounds) in merged_y_bounds.iter() {
        y_axis_title.push_str(&info.unit.to_string());
        y_axis_title.push(' ');

        let new = y_labels(info, layout, *y_bounds);
        if labels_y.is_empty() {
            labels_y = new;
        } else {
            for (existing, new) in labels_y.iter_mut().zip(new) {
                existing.push(' ');
                existing.push_str(new.as_str());
            }
        }
    }

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
        .title(y_axis_title)
        .style(Style::default())
        .bounds([0.0, 1.0])
        .labels(labels_y);

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

    let linechart = Chart::new(datasets)
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, layout);

    if charts.len() > 1 {
        render_legend(layout, frame, charts, colors);
    }
}

fn render_legend(layout: Rect, frame: &mut Frame, charts: &mut [ChartParts], colors: Vec<Color>) {
    let text: Text = charts
        .iter()
        .zip(colors)
        .map(|(ChartParts { reading, .. }, color)| (reading, color))
        .map(fmt_reading)
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

fn fmt_reading((reading, color): (&protocol::Reading, Color)) -> Line {
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
        bounds[0].max(range.start as f64),
        bounds[1].min(range.end as f64),
    ]
}
