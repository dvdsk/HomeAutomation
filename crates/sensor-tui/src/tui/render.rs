use std::iter;

use itertools::Itertools;
use protocol::reading_tree::{ReadingInfo, Tree as _};
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use tracing_subscriber::fmt::format;
use tui_tree_widget::{Tree, TreeItem};

use super::{
    reading::{ChartParts, TreeKey},
    ActiveList, App,
};

mod chart;

pub(crate) fn app(
    frame: &mut Frame,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
    chart: Option<ChartParts>,
    histogram: &[Bar],
) {
    let area = frame.size();
    let [top, bottom] = Layout::vertical([Constraint::Min(10), Constraint::Min(10)])
        .flex(Flex::Legacy)
        .areas(area);

    render_readings_and_actuators(frame, top, app, readings);
    render_graphs(frame, bottom, app, histogram, chart);
}

fn render_graphs(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    histogram: &[Bar],
    chart: Option<ChartParts>,
) {
    match (chart, app.show_histogram) {
        (None, true) => render_histogram(frame, layout, histogram),
        (None, false) => (),
        (Some(chart), true) => {
            let [top, lower] =
                Layout::vertical([Constraint::Percentage(65), Constraint::Percentage(35)])
                    .flex(Flex::Legacy)
                    .areas(layout);
            chart::render(frame, top, app, chart);
            render_histogram(frame, lower, histogram);
        }
        (Some(chart), false) => {
            chart::render(frame, layout, app, chart);
        }
    }
}

fn render_histogram(frame: &mut Frame, lower: Rect, histogram: &[Bar]) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Histogram"))
        .data(BarGroup::default().bars(histogram))
        .bar_width(12)
        .bar_style(Style::default())
        .value_style(Style::default());
    frame.render_widget(barchart, lower)
}


pub(crate) fn render_readings_and_actuators(
    frame: &mut Frame,
    layout: Rect,
    app: &mut App,
    readings: &[TreeItem<'static, TreeKey>],
) {
    let horizontal: [_; 2] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(Flex::Legacy)
            .areas(layout);

    frame.render_stateful_widget(
        Tree::new(readings)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Sensor readings")
                    .borders(Borders::ALL)
                    .border_style(app.active_list.style(ActiveList::Readings)),
            )
            .style(app.active_list.style(ActiveList::Readings))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        horizontal[0],
        &mut app.reading_list_state,
    );

    // frame.render_stateful_widget(
    //     List::new(app.actuators)
    //         .block(
    //             Block::default()
    //                 .title("Actuators")
    //                 .borders(Borders::ALL)
    //                 .border_style(app.active_list.style(ActiveList::Actuators)),
    //         )
    //         .style(app.active_list.style(ActiveList::Actuators))
    //         .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
    //         .highlight_symbol(">>")
    //         .repeat_highlight_symbol(false)
    //         .direction(ratatui::widgets::ListDirection::TopToBottom),
    //     horizontal[1],
    //     app.actuator_list_state,
    // );
}
