use ratatui::{
    self,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType, List, ListState,
    },
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{
    reading::{ChartParts, TreeKey},
    ActiveList,
};

pub(crate) fn all(
    frame: &mut Frame,
    readings: &[TreeItem<'static, TreeKey>],
    reading_state: &mut TreeState<TreeKey>,
    actuators: Vec<String>,
    actuator_list_state: &mut ListState,
    active_list: ActiveList,
    histogram: &[Bar],
    chart: Option<ChartParts>,
) {
    let area = frame.size();
    let [top, bottom] = Layout::vertical([Constraint::Min(10), Constraint::Min(10)])
        .flex(ratatui::layout::Flex::Legacy)
        .areas(area);

    render_top(
        frame,
        top,
        readings,
        reading_state,
        actuators,
        actuator_list_state,
        active_list,
    );
    render_lower(frame, bottom, histogram, chart);
}

fn render_lower(frame: &mut Frame, layout: Rect, histogram: &[Bar], chart: Option<ChartParts>) {
    match chart {
        Some(chart) => detail_view(frame, layout, chart, histogram),
        None => global_view(frame, layout, histogram),
    }
}

fn detail_view(frame: &mut Frame, layout: Rect, chart: ChartParts, histogram: &[Bar]) {
    let [top, lower] = Layout::vertical([Constraint::Percentage(65), Constraint::Percentage(35)])
        .flex(ratatui::layout::Flex::Legacy)
        .areas(layout);

    let dataset = Dataset::default()
        .name(chart.name)
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default())
        .data(chart.data);

    let x_bounds = [
        0f64,
        chart.data.last().map(|(x, _)| x).copied().unwrap_or(0f64),
    ];

    let y_bounds = chart
        .data
        .iter()
        .map(|(_, y)| y)
        .fold([f64::MAX, f64::MIN], |[start, end], y| {
            [start.min(*y), end.max(*y)]
        });
    let y_range = y_bounds[1] - y_bounds[0];
    let y_margin = f64::max(y_range * 0.5, 0.001 * y_bounds[0].abs());
    let y_bounds = [y_bounds[0] - y_margin, y_bounds[1] + y_margin];

    let x_labels = vec![format_time(x_bounds[1]).into(), "0".into()];
    let y_labels = vec![
        format!("{:.3}", y_bounds[0]).into(),
        format!("{:.3}", y_bounds[1]).into(),
    ];

    let x_axis = Axis::default()
        .title("Time")
        .style(Style::default())
        .bounds(x_bounds)
        .labels(x_labels);
    let y_axis = Axis::default()
        .title("Value")
        .style(Style::default())
        .bounds(y_bounds)
        .labels(y_labels);
    let linechart = Chart::new(vec![dataset])
        .block(Block::bordered().title("History"))
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(linechart, top);

    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12)
        .bar_style(Style::default())
        .value_style(Style::default());
    frame.render_widget(barchart, lower)
}

fn global_view(frame: &mut Frame, layout: Rect, histogram: &[Bar]) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12)
        .bar_style(Style::default())
        .value_style(Style::default());
    frame.render_widget(barchart, layout)
}

pub(crate) fn render_top(
    frame: &mut Frame,
    layout: Rect,
    readings: &[TreeItem<'static, TreeKey>],
    reading_list_state: &mut TreeState<TreeKey>,
    actuators: Vec<String>,
    actuator_list_state: &mut ListState,
    active_list: ActiveList,
) {
    let horizontal: [_; 2] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(ratatui::layout::Flex::Legacy)
            .areas(layout);

    frame.render_stateful_widget(
        Tree::new(readings)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Sensor readings")
                    .borders(Borders::ALL)
                    .border_style(active_list.style(ActiveList::Readings)),
            )
            .style(active_list.style(ActiveList::Readings))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        horizontal[0],
        reading_list_state,
    );

    frame.render_stateful_widget(
        List::new(actuators)
            .block(
                Block::default()
                    .title("Actuators")
                    .borders(Borders::ALL)
                    .border_style(active_list.style(ActiveList::Actuators)),
            )
            .style(active_list.style(ActiveList::Actuators))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(false)
            .direction(ratatui::widgets::ListDirection::TopToBottom),
        horizontal[1],
        actuator_list_state,
    );
}

fn format_time(seconds: f64) -> String {
    let seconds = seconds as u32;
    if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 600 {
        let m = seconds / 60;
        let s = seconds % 60;
        format!("{m}:{s:0>1} ago")
    } else {
        format!("{}m ago", seconds / 60)
    }
}
