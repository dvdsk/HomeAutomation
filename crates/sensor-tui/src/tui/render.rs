use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use super::App;

pub(super) fn render_tab(frame: &mut Frame, app: &App) -> Rect {
    let [topline, rest] = Layout::vertical([Constraint::Min(1), Constraint::Fill(1)])
        .flex(Flex::Legacy)
        .areas(frame.area());

    let tabs = Tabs::new(vec!["Readings", "Affectors"])
        .style(app.theme.bars)
        .select(app.active_tab.number())
        .divider("|")
        .padding(" ", " ");

    frame.render_widget(tabs, topline);
    rest
}
