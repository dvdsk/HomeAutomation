use crate::api::subscriber::SubMessage;
use std::time::Duration;

use super::{AffectorRegistar, Event};
use protocol::reading::tree::Tree;
use protocol::{large_bedroom, small_bedroom, Affector, Reading};
use tokio::sync::mpsc;
use tokio::time::{timeout_at, Instant};

const CHECK_INTERVAL: Duration = Duration::from_secs(5);
const MIN_RESET_INTERVAL: Duration = Duration::from_secs(600);

#[derive(Default)]
struct LastSeen {
    map: Vec<(Reading, Instant)>,
    last_reset: Vec<(Affector, Instant)>,
}

impl LastSeen {
    fn update(&mut self, reading: Reading) {
        if let Some((_, time)) = self.map.iter_mut().find(|(r, _)| r.is_same_as(&reading)) {
            *time = Instant::now();
        } else {
            self.map.push((reading, Instant::now()));
        }
    }

    fn mark_reset(&mut self, affector: Affector) {
        if let Some((_, last)) = self
            .last_reset
            .iter_mut()
            .find(|(a, _)| a.is_same_as(&affector))
        {
            *last = Instant::now();
        } else {
            self.last_reset.push((affector, Instant::now()));
        }
    }

    fn check_and_bite(&mut self, registar: &AffectorRegistar) {
        let to_reset = self.map.iter().filter(|(reading, last_seen)| {
            let max_interval = reading.leaf().device.info().max_sample_interval;
            last_seen.elapsed() > max_interval * 10
        });

        let mut reset_commands: Vec<_> = to_reset
            .filter_map(|(reading, _)| match reading {
                Reading::LargeBedroom(large_bedroom::Reading::Bed(_)) => {
                    Some(Affector::LargeBedroom(large_bedroom::Affector::Bed(
                        large_bedroom::bed::Affector::ResetNode,
                    )))
                }
                Reading::LargeBedroom(large_bedroom::Reading::Desk(_)) => None,
                Reading::SmallBedroom(small_bedroom::Reading::Bed(_)) => {
                    Some(Affector::SmallBedroom(small_bedroom::Affector::Bed(
                        small_bedroom::bed::Affector::ResetNode,
                    )))
                }
                Reading::SmallBedroom(small_bedroom::Reading::Desk(_)) => None,
                Reading::SmallBedroom(small_bedroom::Reading::ButtonPanel(_)) => None,
            })
            .filter(|affector| {
                let last_reset = self
                    .last_reset
                    .iter()
                    .find(|(a, _)| a == affector)
                    .map(|(_, at)| at)
                    .copied();
                !last_reset.is_some_and(|last_reset| last_reset.elapsed() < MIN_RESET_INTERVAL)
            })
            .collect();
        reset_commands.dedup_by(|a, b| a.is_same_as(b));
        for cmd in reset_commands {
            if registar.activate(cmd).is_ok() {
                self.mark_reset(cmd);
            }
        }
    }
}

pub async fn node_watchdog(registar: AffectorRegistar, sub_tx: &mpsc::Sender<Event>) -> ! {
    let (tx, mut rx) = mpsc::channel(128);
    sub_tx
        .send(Event::NewSub { tx })
        .await
        .expect("handle_sub_should_still_run");

    let mut next_check = tokio::time::Instant::now() + CHECK_INTERVAL;
    let mut last_seen = LastSeen::default();
    loop {
        match timeout_at(next_check, rx.recv()).await {
            Ok(None) => unreachable!("subscribers should never be dropped"),
            Ok(Some(SubMessage::Reading(r))) => last_seen.update(r),
            Ok(Some(_)) => (),
            Err(_timeout) => {
                last_seen.check_and_bite(&registar);
                next_check = Instant::now() + CHECK_INTERVAL;
            }
        }
    }
}
