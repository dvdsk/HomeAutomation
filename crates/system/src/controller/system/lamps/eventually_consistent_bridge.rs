use super::{bridge_connect, lamp::Lamp};
use super::{ApplyChangeError, State};
use crate::errors::Error;
use futures_util::stream::Peekable;
use futures_util::{FutureExt, Stream, StreamExt};
use philipshue::LightCommand;
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot;
use tracing::error;

use philipshue::bridge::Bridge;

pub(crate) type LampId = usize;
pub(crate) enum Change {
    AllOff,
    AllOn,
    Off { lamp_id: LampId },
    On { lamp_id: LampId },
    CtBri { lamp_id: LampId, bri: u8, ct: u16 },
    AllCtBri { bri: u8, ct: u16 },
    AllXY { bri: u8, xy: (f32, f32) },
}

pub(crate) struct CachedBridge {
    pub(crate) bridge: Bridge,
    pub(crate) needed_state: State,
    pub(crate) known_state: State,
}

impl CachedBridge {
    pub(crate) fn try_init() -> Result<Self, Error> {
        let (bridge, lights_info) = bridge_connect::get_bridge_and_status()?;
        let state: HashMap<usize, Lamp> = lights_info
            .iter()
            .map(|(id, light)| (*id, Lamp::from(&light.state)))
            .collect();

        Ok(Self {
            bridge,
            needed_state: state.clone(),
            known_state: state,
        })
    }

    pub(crate) fn apply_changes(&mut self) -> Result<(), Error> {
        for (id, lamp) in self.known_state.iter_mut() {
            let Some(needed) = self.needed_state.get(id) else {
                continue;
            };

            if lamp != needed {
                self.bridge
                    .set_light_state(
                        *id,
                        &LightCommand::default()
                            .with_xy(needed.xy.unwrap())
                            .with_bri(needed.bri),
                    )
                    .unwrap();
            }
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
            Change::Off { lamp_id } => {
                if let Some(lamp) = self.needed_state.get_mut(&lamp_id) {
                    lamp.on = true;
                } else {
                    error!("no lamp with id: {lamp_id} exists");
                }
            }
            Change::On { lamp_id } => {
                if let Some(lamp) = self.needed_state.get_mut(&lamp_id) {
                    lamp.on = false;
                } else {
                    error!("no lamp with id: {lamp_id} exists");
                }
            }
            Change::CtBri { lamp_id, bri, ct } => {
                if let Some(lamp) = self.needed_state.get_mut(lamp_id) {
                    lamp.bri = *bri;
                    lamp.ct = Some(*ct);
                }
            }
            Change::AllCtBri { bri, ct } => {
                self.needed_state.values_mut().for_each(|lamp| {
                    lamp.bri = *bri;
                    lamp.ct = Some(*ct);
                });
            }
            Change::AllXY { bri, xy } => {
                self.needed_state.values_mut().for_each(|lamp| {
                    lamp.bri = *bri;
                    lamp.xy = Some(*xy);
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
    loop {
        let mut to_answer = Vec::new();
        // process backlog
        loop {
            let Some((_, change)) = stream.as_mut().peek_mut().now_or_never().flatten() else {
                let change_appears = stream.as_mut().peek_mut();
                let not_instant = tokio::time::sleep(Duration::from_millis(50));
                select! {
                    _ = change_appears => continue,
                    _ = not_instant => break,

                }
            };
            bridge.prep_change(change);
            // remove the now processed item from the stream and store its
            // answer tx
            let (tx, _) = stream.as_mut().next().await.expect("just peeked");
            to_answer.push(tx);
        }

        if let Err(e) = bridge.apply_changes() {
            for tx in to_answer {
                let _ignore_canceld_requester = tx.send(Err(ApplyChangeError));
            }
            return e;
        }
    }
}
