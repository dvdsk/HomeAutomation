use super::{bridge_connect, lamp::Lamp};
use super::{ApplyChangeError, State};
use crate::errors::Error;
use futures_util::stream::Peekable;
use futures_util::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tokio::time::{timeout, timeout_at};
use tracing::{error, warn};

use hueclient::Bridge;

#[derive(Debug)]
pub(crate) enum Change {
    AllOff,
    AllOn,
    Off {
        name: &'static str,
    },
    On {
        name: &'static str,
    },
    CtBri {
        name: &'static str,
        bri: u8,
        ct: u16,
    },
    XyBri {
        name: &'static str,
        bri: u8,
        xy: (f32, f32),
    },
    AllCtBri {
        bri: u8,
        ct: u16,
    },
    AllXY {
        bri: u8,
        xy: (f32, f32),
    },
}

type LampId = usize;
pub(crate) struct CachedBridge {
    last_cmd_send: Instant,
    pub(crate) bridge: Bridge,
    pub(crate) needed_state: State,
    pub(crate) known_state: State,
    // get names using: curl 192.168.1.11/api/<HUE API KEY>/lights | jq | grep '"name": "'
    lookup: HashMap<String, LampId>,
    reported_missing: HashSet<String>,
}

impl CachedBridge {
    pub(crate) async fn try_init(ip: &str) -> Result<Self, Error> {
        let (bridge, lights_info) = bridge_connect::get_bridge_and_status(ip).await?;
        let state: HashMap<usize, Lamp> = lights_info
            .iter()
            .map(|light| (light.id, Lamp::from(&light.light.state)))
            .collect();
        let lookup = lights_info
            .into_iter()
            .map(|light| (light.light.name, light.id))
            .collect();

        Ok(Self {
            bridge,
            needed_state: state.clone(),
            known_state: state,
            lookup,
            reported_missing: HashSet::new(),
            last_cmd_send: Instant::now(),
        })
    }

    fn report_missing_if_not_reported_yet(&mut self, missing_lamp: &str) {
        let new = self.reported_missing.insert(missing_lamp.to_string());
        if new {
            error!("no lamp with name: {missing_lamp} in lookup table, was recently (re)named?");
        }
    }

    pub(crate) async fn apply_changes(&mut self) -> Result<(), Error> {
        for (id, lamp) in self.known_state.iter_mut() {
            let Some(needed) = self.needed_state.get(id) else {
                continue;
            };

            if lamp != needed {
                let next_send_possible = self.last_cmd_send + Duration::from_millis(100);
                tokio::time::sleep_until(next_send_possible.into()).await;
                if let Err(e) = self.bridge.set_light_state(*id, &needed.light_cmd()).await {
                    warn!("could not apply changes to lamp: {e}")
                }
                self.last_cmd_send = Instant::now();
                *lamp = needed.clone()
            }
        }

        Ok(())
    }

    async fn push_state(&mut self) -> Result<(), Error> {
        for (id, lamp) in self.known_state.iter_mut() {
            let next_send_possible = self.last_cmd_send + Duration::from_millis(100);
            tokio::time::sleep_until(next_send_possible.into()).await;
            if let Err(e) = self.bridge.set_light_state(*id, &lamp.light_cmd()).await {
                warn!("could not apply changes to lamp: {e}")
            }
            self.last_cmd_send = Instant::now();
        }
        Ok(())
    }

    pub(crate) fn prep_change(&mut self, change: &Change) {
        match change {
            Change::AllOff => {
                self.needed_state
                    .values_mut()
                    .for_each(|lamp| lamp.on = false);
            }
            Change::AllOn => {
                self.needed_state
                    .values_mut()
                    .for_each(|lamp| lamp.on = true);
            }
            Change::Off { name } => {
                let Some(lamp_id) = self.lookup.get(*name) else {
                    self.report_missing_if_not_reported_yet(*name);
                    return;
                };
                if let Some(lamp) = self.needed_state.get_mut(lamp_id) {
                    lamp.on = false;
                } else {
                    error!("no lamp with id: {lamp_id} exists");
                }
            }
            Change::On { name } => {
                let Some(lamp_id) = self.lookup.get(*name) else {
                    self.report_missing_if_not_reported_yet(*name);
                    return;
                };
                if let Some(lamp) = self.needed_state.get_mut(&lamp_id) {
                    lamp.on = true;
                } else {
                    error!("no lamp with id: {lamp_id} exists");
                }
            }
            Change::CtBri { name, bri, ct } => {
                let Some(lamp_id) = self.lookup.get(*name) else {
                    self.report_missing_if_not_reported_yet(*name);
                    return;
                };
                if let Some(lamp) = self.needed_state.get_mut(lamp_id) {
                    lamp.bri = *bri;
                    lamp.ct = Some(*ct);
                    lamp.xy = None;
                }
            }
            Change::XyBri { name, bri, xy } => {
                let Some(lamp_id) = self.lookup.get(*name) else {
                    self.report_missing_if_not_reported_yet(*name);
                    return;
                };
                if let Some(lamp) = self.needed_state.get_mut(lamp_id) {
                    lamp.bri = *bri;
                    lamp.xy = Some(*xy);
                    lamp.ct = None;
                }
            }
            Change::AllCtBri { bri, ct } => {
                self.needed_state.values_mut().for_each(|lamp| {
                    lamp.bri = *bri;
                    lamp.ct = Some(*ct);
                    lamp.xy = None;
                });
            }
            Change::AllXY { bri, xy } => {
                self.needed_state.values_mut().for_each(|lamp| {
                    lamp.bri = *bri;
                    lamp.xy = Some(*xy);
                    lamp.ct = None;
                });
            }
        }
    }
}

pub(crate) async fn process_lamp_changes<S>(
    stream: &mut Pin<&mut Peekable<S>>,
    mut bridge: CachedBridge,
) -> Error
where
    S: Stream<Item = (oneshot::Sender<Result<(), ApplyChangeError>>, Change)>,
{
    const MAX_DUR: Duration = Duration::from_millis(5);
    let mut last_state_push = Instant::now();
    loop {
        if last_state_push.elapsed() > Duration::from_secs(5) {
            if let Err(err) = bridge.push_state().await {
                return err;
            }
            last_state_push = Instant::now();
        }

        let (tx, change) = match timeout(Duration::from_secs(5), stream.next()).await {
            Ok(Some(next)) => next,
            Ok(None) => unreachable!("light system should not drop"),
            Err(_timeout) => {
                if let Err(err) = bridge.push_state().await {
                    return err;
                } else {
                    continue;
                }
            }
        };

        bridge.prep_change(&change);
        let mut to_answer = vec![tx];

        let now = Instant::now();
        let deadline = tokio::time::Instant::from(now) + MAX_DUR;

        // process backlog
        while now.elapsed() < MAX_DUR {
            match timeout_at(deadline, stream.next()).await {
                Err(_timeout) => break,
                Ok(None) => unreachable!("light system should not drop"),
                Ok(Some((tx, change))) => {
                    bridge.prep_change(&change);
                    to_answer.push(tx);
                }
            };
        }

        if let Err(e) = bridge.apply_changes().await {
            error!("Could not apply changes to bridge immediately, err: {e}");
            for tx in to_answer {
                let _ignore_canceled_requester = tx.send(Err(ApplyChangeError));
            }
            return e;
        } else {
            for tx in to_answer {
                let _ignore_canceled_requester = tx.send(Ok(()));
            }
        }
    }
}
