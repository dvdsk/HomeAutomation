use std::time::Duration;

use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};

use crate::controller::rooms::small_bedroom;
use crate::controller::{Event, RestrictedSystem};

const INTERVAL: Duration = Duration::from_secs(5);

pub async fn run(
    mut _event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
) {

    let mut next_update = Instant::now() + INTERVAL;
    loop {
        sleep_until(next_update).await;

        update(&mut system).await;
        system.all_lamps_on().await;

        next_update = Instant::now() + INTERVAL;
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = small_bedroom::daylight_now();
    let new_bri = new_bri.clamp(0.8, 1.0);
    system.all_lamps_ct(new_ct, new_bri).await;
    tracing::trace!("updated lamps");
}
