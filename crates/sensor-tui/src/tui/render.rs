use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use super::App;

pub(super) fn render(frame: &mut Frame, app: &App) -> Rect {
    let report_needs_lines = app.reports.needed_lines();
    let constraints = report_needs_lines
        .map(Constraint::Max)
        .into_iter()
        .chain([Constraint::Min(1), Constraint::Fill(1)]);
    let layout = Layout::default()
        .constraints(constraints)
        .flex(Flex::Legacy)
        .split(frame.area());
    let mut layout = layout.iter();

    if report_needs_lines.is_some() {
        app.reports
            .render(frame, *layout.next().expect("is long enough"))
    }

    render_tab_bar(app, frame, *layout.next().expect("is long enough"));

    *layout.next().expect("is long enough")
}

fn render_tab_bar(app: &App, frame: &mut Frame, topline: Rect) {
    let tabs = Tabs::new(vec!["Readings", "Affectors"])
        .style(app.theme.bars)
        .select(app.active_tab.number())
        .divider("|")
        .padding(" ", " ");

    frame.render_widget(tabs, topline);
}
