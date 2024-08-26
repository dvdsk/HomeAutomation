use crossterm::event::{KeyCode, KeyEvent};
use protocol::affector::ControlValue as C;

use super::AffectorState;

pub(crate) fn handle(key: KeyEvent, state: &mut AffectorState) -> Option<KeyEvent> {
    let mut controls = state.affector.controls();
    let control = &mut controls[state.selected_control].value;

    match key.code {
        KeyCode::Char('f') => handle_increase(control),
        KeyCode::Char('b') => handle_decrease(control),

        KeyCode::Char('d') => {
            state.selected_control = (state.selected_control + 1).min(controls.len() - 1)
        }
        KeyCode::Char('u') => state.selected_control = state.selected_control.saturating_sub(1),
        _ => return Some(key),
    };
    None
}

fn handle_increase(control: &mut C) {
    match control {
        C::Trigger => (),
        C::SetNum {
            valid_range,
            setter,
            value,
        } => {
            let new_value = *value + 1;
            let new_value = new_value as u64;
            let new_value = new_value.clamp(valid_range.start, valid_range.end);
            let setter = setter.take().expect("just created controls");
            setter(new_value as usize);
        }
    }
}

fn handle_decrease(control: &mut C) {
    match control {
        C::Trigger => (),
        C::SetNum {
            valid_range,
            setter,
            value,
        } => {
            let new_value = *value - 1;
            let new_value = new_value as u64;
            let new_value = new_value.clamp(valid_range.start, valid_range.end);
            let setter = setter.take().expect("just created controls");
            setter(new_value as usize);
        }
    }
}
