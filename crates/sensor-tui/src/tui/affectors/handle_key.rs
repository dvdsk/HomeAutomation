use crossterm::event::{KeyCode, KeyEvent};
use protocol::affector::ControlValue as C;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tracing::{error, warn};

use super::AffectorState;

pub(crate) fn handle(
    key: KeyEvent,
    state: &mut AffectorState,
    control_tx: &mut mpsc::Sender<protocol::Affector>,
) -> Option<KeyEvent> {
    let mut controls = state.affector.controls();
    let control = &mut controls[state.selected_control].value;

    match key.code {
        KeyCode::Char('t') => handle_trigger(control),
        KeyCode::Char('f') => handle_increase(control),
        KeyCode::Char('b') => handle_decrease(control),

        // Otherwise return since we have not changed the affector
        KeyCode::Char('d') => {
            state.selected_control = (state.selected_control + 1).min(controls.len() - 1);
            return None;
        }
        KeyCode::Char('u') => {
            state.selected_control = state.selected_control.saturating_sub(1);
            return None;
        }
        _ => return Some(key),
    };

    core::mem::drop(controls);
    match control_tx.try_send(state.affector.clone()) {
        Ok(_) => (),
        Err(TrySendError::Full(_)) => warn!("control lagging behind ui inputs"),
        Err(TrySendError::Closed(_)) => error!("control lost connection to data-server"),
    }

    None
}

fn handle_trigger(control: &mut C) {
    match control {
        C::Trigger => (),
        C::SetNum { .. } => (),
    }
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
            let new_value = value.saturating_sub(1);
            let new_value = new_value as u64;
            let new_value = new_value.clamp(valid_range.start, valid_range.end);
            let setter = setter.take().expect("just created controls");
            setter(new_value as usize);
        }
    }
}
