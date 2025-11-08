use std::time::Duration;

use futures::FutureExt;
use futures_concurrency::future::Race;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};

use crate::controller::rooms::common::RecvFiltered;
use crate::controller::rooms::small_bedroom;
use crate::controller::{Event, RestrictedSystem};

const INTERVAL: Duration = Duration::from_secs(5);

#[derive(PartialEq, Eq, Serialize, Deserialize, Default)]
enum State {
    Sleep,
    #[default]
    Daylight,
}

#[derive(Debug)]
enum RelevantEvent {
    Sleep,
    Daylight,
}

fn filter(event: Event) -> Option<RelevantEvent> {
    match event {
        Event::StateChangeSB(small_bedroom::State::Sleep) => {
            Some(RelevantEvent::Sleep)
        }
        Event::StateChangeSB(small_bedroom::State::Daylight) => {
            Some(RelevantEvent::Daylight)
        }
        _ => None,
    }
}

#[dbstruct::dbstruct(db=sled)]
struct Store {
    #[dbstruct(Default)]
    state: State,
}

super::impl_open_or_wipe!(Store);

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
    db: sled::Tree,
) -> color_eyre::Result<()> {
    enum Res {
        Event(RelevantEvent),
        ShouldUpdate,
    }

    let db = open_or_wipe(db)?;

    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        match res {
            Res::Event(RelevantEvent::Sleep) => {
                db.state().set(&State::Sleep)?;
                system.one_lamp_off("hallway:ceiling").await;
            }
            Res::Event(RelevantEvent::Daylight) => {
                db.state().set(&State::Daylight)?;

                update(&mut system).await;
                system.all_lamps_on().await;
            }
            Res::ShouldUpdate => {
                if db.state().get()? == State::Daylight {
                    update(&mut system).await;
                    system.all_lamps_on().await;
                }

                next_update = Instant::now() + INTERVAL;
            }
        }
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = small_bedroom::daylight_now();
    let new_ct = new_ct.clamp(1000, 3000);
    let new_bri = new_bri.clamp(0.5, 1.0);
    system.all_lamps_ct(new_ct, new_bri).await;
    tracing::trace!("updated lamps");
}
