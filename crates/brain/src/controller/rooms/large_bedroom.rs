use std::time::{Duration, SystemTime};

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::{info, warn};

use crate::controller::rooms::common::RecvFiltered;
use crate::controller::{Event, RestrictedSystem};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
enum State {
    // Sleep,
    // Wakeup,
    FadeOut(SystemTime),
    #[default]
    Normal,
    Bright,
    Off,
    // Away,
}

const INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
enum RelevantEvent {
    WakeUp,
    // WeightLeft(u32),
    // WeightRight(u32),
    // Brightness(f32), // millilux
    DeskButton(protocol::large_bedroom::desk::Button),
    BedButton(protocol::large_bedroom::bed::Button),
    // BedButton(protocol::large_bedroom::bed::Button),
}

fn filter(event: Event) -> Option<RelevantEvent> {
    use protocol::large_bedroom::bed::Reading as B;
    use protocol::large_bedroom::desk::Reading as D;
    use protocol::large_bedroom::Reading as R;
    use protocol::Reading::LargeBedroom as LB;
    use Event::{Sensor, WakeupLB};

    Some(match event {
        WakeupLB => RelevantEvent::WakeUp,
        Sensor(LB(R::Desk(D::Button(b)))) => RelevantEvent::DeskButton(b),
        Sensor(LB(R::Bed(B::Button(b)))) => RelevantEvent::BedButton(b),
        // Sensor(LB(R::Bed(B::Brightness(l)))) => RelevantEvent::Brightness(l),
        // Sensor(LB(R::Bed(B::WeightLeft(w)))) => RelevantEvent::WeightLeft(w),
        // Sensor(LB(R::Bed(B::WeightRight(w)))) => RelevantEvent::WeightRight(w),
        _ => return None,
    })
}

#[dbstruct::dbstruct(db=sled)]
struct Store {
    #[dbstruct(Default)]
    state: State,
}

super::impl_open_or_wipe!(Store);

pub(crate) async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
    tree: sled::Tree,
) -> Result<(), color_eyre::Report> {
    enum Res {
        Event(RelevantEvent),
        ShouldUpdate,
    }

    let db = open_or_wipe(tree)?;
    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        let new_state = match res {
            Res::Event(e) => handle_event(e),
            Res::ShouldUpdate => {
                next_update = Instant::now() + INTERVAL;
                update(&mut system, &db.state().get()?).await
            }
        };

        if let Some(new) = new_state {
            info!("transitioning to new state: {new:?}");
            db.state().set(&new)?;
            next_update = Instant::now() + INTERVAL;
            if update(&mut system, &db.state().get()?).await.is_some() {
                warn!("Transiting to a new state while in the first update is not allowed")
            }
        }
    }
}

async fn update(system: &mut RestrictedSystem, state: &State) -> Option<State> {
    match state {
        State::Off => {
            system.all_lamps_off().await;
            /* TODO: make states have destructors, maybe even remove enum in
             * favor of polymorphism? <dvdsk noreply@davidsk.dev> */
        }
        State::Normal => {
            let (new_ct, new_bri) = optimal_ct_bri();
            system.all_lamps_on().await;
            system.all_lamps_ct(new_ct, new_bri).await;
        }
        State::Bright => {
            system.all_lamps_on().await;
            system.all_lamps_ct(3900, 1.0).await;
        }
        State::FadeOut(started) => {
            system.all_lamps_on().await;
            // TODO: this was set to 1 mired (= 1M(!!) Kelvin) and 100% brightness
            // not sure that is the intended behaviour
            system.all_lamps_ct(3900, 1.0).await;
            if !started.elapsed().is_ok_and(|t| t < Duration::from_secs(40)) {
                return Some(State::Off);
            }
        }
    }
    None
}

fn optimal_ct_bri() -> (usize, f64) {
    let now = crate::time::now();
    match now.hour() {
        0..=5 | 22.. => (2000, 0.67),
        6..=16 => (3900, 1.0),
        17..=19 => (3100, 1.0),
        20..=21 => (3100, 0.87),
        _ => (3100, 1.0),
    }
}

fn handle_event(e: RelevantEvent) -> Option<State> {
    // use protocol::large_bedroom::DeskButton;
    // use RelevantEvent as R;

    match e {
        RelevantEvent::WakeUp => None,
        // RelevantEvent::WeightLeft(_) => (),
        // RelevantEvent::WeightRight(_) => (),
        // RelevantEvent::Brightness(_) => (),
        RelevantEvent::DeskButton(b) => handle_desk_button(b),
        RelevantEvent::BedButton(b) => handle_bed_button(b),
        // RelevantEvent::BedButton(_) => (),
    }
}

fn handle_desk_button(
    b: protocol::large_bedroom::desk::Button,
) -> Option<State> {
    use protocol::large_bedroom::desk::Button;

    println!("button pressed: {b:?}");
    match b {
        Button::OneOfFour(press) if press.is_long() => Some(State::Off),
        Button::OneOfFour(_) => Some(State::FadeOut(SystemTime::now())),
        Button::TwoOfFour(press) if press.is_long() => Some(State::Bright),
        Button::FourOfFour(press) if !press.is_long() => Some(State::Normal),
        // Button::TwoOfFour(_) => todo!(),
        // Button::ThreeOfFour(_) => todo!(),
        // Button::OneOfThree(_) => todo!(),
        // Button::TwoOfThree(_) => todo!(),
        // Button::ThreeOfThree(_) => todo!(),
        _ => None,
    }
}

fn handle_bed_button(b: protocol::large_bedroom::bed::Button) -> Option<State> {
    use protocol::large_bedroom::bed::Button as B;

    println!("button pressed: {b:?}");
    match b {
        B::MiddleOuter(_) => Some(State::Bright),
        B::MiddleInner(_) => Some(State::Normal),
        B::MiddleCenter(_) => Some(State::FadeOut(SystemTime::now())),
        _ => None,
    }
}
