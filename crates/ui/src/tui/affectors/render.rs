use core::iter;
use protocol::affector;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{self, Block, Borders, Gauge};
use ratatui::Frame;
use tui_tree_widget::Tree;
use tui_tree_widget::{TreeItem, TreeState};

use crate::tui::Theme;

use super::{AffectorState, DeviceBroken, TreeKey};

pub(super) fn tree(
    frame: &mut Frame,
    left: Rect,
    ground: &[TreeItem<TreeKey>],
    state: &mut TreeState<TreeKey>,
) {
    frame.render_stateful_widget(
        Tree::new(ground)
            .expect("all item identifiers should be unique")
            .block(
                Block::default()
                    .title("Controllable affectors")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>"),
        left,
        state,
    );
}

pub(super) fn details(frame: &mut Frame, data: &mut AffectorState, top: Rect) {
    let mut text = vec![data.info.description.to_owned()];
    if let Some(ref name) = data.last_controlled_by {
        text.push(format!("\nlast controlled by: {name}"));
    }
    if let Some(ref at) = data.last_input {
        let input_action = match data
            .affector
            .controls()
            .get(data.selected_control)
            .expect("controls are never removed")
            .value
        {
            affector::ControlValue::Trigger => "trigged",
            affector::ControlValue::SetNum { .. } => "set",
        };
        let elapsed = crate::time::format::duration(at.elapsed().as_secs_f64());
        text.push(format!("\nwe last {input_action}: {elapsed} ago"));
    }
    if let Some(ref status) = data.last_order_status {
        text.push(format!("\nstatus: {status}"));
    }
    if let DeviceBroken::Yes = data.device_broken {
        text.push(
            "\nWarning: Device reports error, affector might not work"
                .to_owned(),
        );
    }
    let text = text.join("\n");

    frame.render_widget(
        widgets::Paragraph::new(text)
            .block(Block::bordered().title("Details"))
            .wrap(widgets::Wrap { trim: true }),
        top,
    )
}

pub(crate) fn controls(
    frame: &mut Frame,
    data: &mut &mut AffectorState,
    bottom: Rect,
) {
    use affector::ControlValue as V;

    let controls = data.affector.controls();
    let constraints = iter::once(Constraint::Max(1))
        .chain(controls.iter().map(|_| Constraint::Max(3)));
    let layout = Layout::default().constraints(constraints).split(bottom);
    let mut layout = layout.iter();

    frame.render_widget(
        Block::new().title("Controls").borders(Borders::TOP),
        *layout.next().expect("has at least one element"),
    );

    for (i, (control, layout)) in controls.iter().zip(layout).enumerate() {
        let is_selected = i == data.selected_control;
        match &control.value {
            V::Trigger => render_trigger(frame, *layout, i),
            V::SetNum {
                valid_range, value, ..
            } => render_slider(
                frame,
                *layout,
                control.name,
                &valid_range,
                *value,
                is_selected,
            ),
        }
    }
}

#[tracing::instrument(skip(frame, layout))]
fn render_slider(
    frame: &mut Frame,
    layout: Rect,
    name: &str,
    valid_range: &std::ops::Range<u64>,
    current_value: usize,
    is_active: bool,
) {
    let style = if is_active {
        Style::default().black()
    } else {
        Style::default()
    };

    let percentage =
        (current_value as u64 * 100) / (valid_range.end - valid_range.start);
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(name).style(style))
            .gauge_style(Style::default().add_modifier(Modifier::ITALIC))
            .label(format!(
                "{current_value} of {}-{}",
                valid_range.start, valid_range.end
            ))
            .percent(percentage as u16),
        layout,
    )
}

fn render_trigger(frame: &mut Frame, layout: Rect, _control_idx: usize) {
    frame.render_widget(
        widgets::Paragraph::new("activate")
            .block(Block::bordered().title("Controls"))
            .wrap(widgets::Wrap { trim: true }),
        layout,
    )
}

pub(crate) fn footer(
    frame: &mut Frame,
    layout: Rect,
    data: Option<&mut AffectorState>,
    theme: &Theme,
) {
    let mut footer = Vec::new();

    use affector::ControlValue as C;
    if let Some(AffectorState {
        affector,
        selected_control,
        ..
    }) = data
    {
        footer.push("u/d: select prev/next");
        match affector.controls()[*selected_control].value {
            C::Trigger => footer.push("t: trigger affector"),
            C::SetNum { .. } => footer.push("f/b increase/decrease"),
        }
    }

    let footer = footer.join("  ");
    let footer = Text::raw(footer)
        .alignment(Alignment::Center)
        .style(theme.bars);
    frame.render_widget(footer, layout)
}
