use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};

use crate::controller::{local_now, Event, RestrictedSystem};

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    // Sleep,
    // Wakeup,
    Normal,
    Bright,
    // Away,
}

const INTERVAL: Duration = Duration::from_secs(5);

trait RecvFiltered {
    async fn recv_filter_mapped<T>(&mut self, filter_map: impl Fn(Event) -> Option<T>) -> T;
}

impl RecvFiltered for broadcast::Receiver<Event> {
    async fn recv_filter_mapped<T>(&mut self, filter_map: impl Fn(Event) -> Option<T>) -> T {
        loop {
            let event = self.recv().await.unwrap();
            if let Some(relevant) = filter_map(event) {
                return relevant;
            }
        }
    }
}

#[derive(Debug)]
enum RelevantEvent {
    WakeUp,
    // WeightLeft(u32),
    // WeightRight(u32),
    // Brightness(f32), // millilux
    DeskButton(protocol::large_bedroom::desk::Button),
    // BedButton(protocol::large_bedroom::bed::Button),
}

fn filter(event: Event) -> Option<RelevantEvent> {
    // use protocol::large_bedroom::bed::Reading as B;
    use protocol::large_bedroom::desk::Reading as D;
    use protocol::large_bedroom::Reading as R;
    use protocol::Reading::LargeBedroom as LB;
    use Event::{Sensor, WakeUp};

    Some(match event {
        WakeUp => RelevantEvent::WakeUp,
        Sensor(LB(R::Desk(D::Button(b)))) => RelevantEvent::DeskButton(b),
        // Sensor(LB(R::Bed(B::Button(b)))) => RelevantEvent::BedButton(b),
        // Sensor(LB(R::Bed(B::Brightness(l)))) => RelevantEvent::Brightness(l),
        // Sensor(LB(R::Bed(B::WeightLeft(w)))) => RelevantEvent::WeightLeft(w),
        // Sensor(LB(R::Bed(B::WeightRight(w)))) => RelevantEvent::WeightRight(w),
        _ => return None,
    })
}

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
) {
    enum Res {
        Event(RelevantEvent),
        ShouldUpdate,
    }

    let mut state = State::Normal;
    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        let new_state = match res {
            Res::Event(e) => handle_event(e),
            Res::ShouldUpdate => {
                update(&mut system, &state).await;
                next_update = Instant::now() + INTERVAL;
                None
            }
        };

        if let Some(new) = new_state {
            state = new;
            update(&mut system, &state).await;
            next_update = Instant::now() + INTERVAL;
        }
    }
}

async fn update(system: &mut RestrictedSystem, state: &State) {
    match state {
        State::Normal => {
            let (new_ct, new_bri) = optimal_ct_bri();
            system.all_lamps_ct(new_ct, new_bri).await;
        }
        State::Bright => {
            system.all_lamps_ct(254, u8::MAX).await;
        }
    }
}

fn optimal_ct_bri() -> (u16, u8) {
    let now = local_now();
    match now.hour() {
        0..=5 | 22.. => (500, 170),
        6..=16 => (254, u8::MAX),
        17..=19 => (320, u8::MAX),
        20..=21 => (320, 220),
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
        RelevantEvent::DeskButton(b) => handle_button(b),
        // RelevantEvent::BedButton(_) => (),
    }
}

fn handle_button(b: protocol::large_bedroom::desk::Button) -> Option<State> {
    use protocol::large_bedroom::desk::Button;

    println!("button pressed: {b:?}");
    match b {
        Button::OneOfFour(press) if press.is_long() => Some(State::Bright),
        Button::FourOfFour(press) if !press.is_long() => Some(State::Normal),
        // Button::TwoOfFour(_) => todo!(),
        // Button::ThreeOfFour(_) => todo!(),
        // Button::OneOfThree(_) => todo!(),
        // Button::TwoOfThree(_) => todo!(),
        // Button::ThreeOfThree(_) => todo!(),
        _ => None,
    }
}
