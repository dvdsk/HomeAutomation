extern crate philipshue;
extern crate serde_yaml;

use futures_util::{pin_mut, stream, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use std::collections::HashMap;
use std::time::Duration;

use self::bridge_connect::SaveBridgeError;
use self::eventually_consistent_bridge::Change;

mod bridge_connect;
mod eventually_consistent_bridge;
mod lamp;

type State = HashMap<usize, lamp::Lamp>;

async fn manage_bridge(
    rx: mpsc::UnboundedReceiver<(oneshot::Sender<Result<(), ApplyChangeError>>, Change)>,
) {
    let stream = stream::unfold(rx, |mut rx| async move {
        let yielded = rx.recv().await;
        yielded.map(|item| (item, rx))
    });

    let stream = stream.peekable();
    pin_mut!(stream);
    loop {
        let error = match eventually_consistent_bridge::CachedBridge::try_init() {
            Ok(bridge) => {
                eventually_consistent_bridge::process_lamp_changes(&mut stream, bridge).await
            }
            Err(e) => e,
        };

        error!("could not connect to bridge: {error:?}");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub struct Lighting {
    _manage_bridge: tokio::task::JoinHandle<()>,
    tx: mpsc::UnboundedSender<(oneshot::Sender<Result<(), ApplyChangeError>>, Change)>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not get lights from bridge")]
    GettingLights(philipshue::errors::HueError),
    #[error("Could not find the bridge via upnp")]
    NoBridgeFound,
    #[error("Error while trying to discover the bridge")]
    Discovery(philipshue::errors::HueError),
    #[error("Failed to register on bridge")]
    Register(#[from] bridge_connect::RegisterError),
    #[error("Something went wrong saving bridge account to disk: {0}")]
    SavingBridgeAccount(#[from] SaveBridgeError),
}

#[derive(Debug, thiserror::Error)]
#[error("Could not apply changes to lighting system. Will simulate this and further changes then automatically apply the simulation outcome once the issue is resolved")]
pub struct ApplyChangeError;

macro_rules! light_fn {
    ($name:ident, $change:ident$(; $($arg:ident: $type:ty),+)?) => {
        pub fn $name(&mut self, $($($arg: $type),+)?) -> Result<(), ApplyChangeError> {
            // todo use oneshot to get error back from other side
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.tx.send((tx, Change::$change$({$($arg),+})?)).expect("managed_bridge should channel should never drop as managed bridge task should never end");
            let res = rx.blocking_recv().expect("managed_bridge should not crash and thus not drop the receive end of the channel");
            res
        }
    };
}

impl Lighting {
    pub fn start_init() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let manage_bridge = manage_bridge(rx);
        let manage_bridge = tokio::task::spawn(manage_bridge);
        Self {
            _manage_bridge: manage_bridge,
            tx,
        }
    }

    //how to deal with errors?
    pub fn toggle(&mut self) -> Result<(), Error> {
        error!("not implemented");
        Ok(())
    }

    light_fn! {all_off, AllOff}
    light_fn! {all_on, AllOn}
    light_fn! {single_off, Off; lamp_id: usize}
    light_fn! {single_on, On; lamp_id: usize}

    light_fn! {set_ct, CtBri; lamp_id: usize, bri: u8, ct: u16 }
    light_fn! {set_all_ct, AllCtBri; bri: u8, ct: u16 }
    light_fn! {set_all_xy, AllXY; bri: u8, xy: (f32, f32) }

    pub fn set_all_rgb(&mut self, bri: u8, rgb: (f32, f32, f32)) -> Result<(), ApplyChangeError> {
        let xy = lamp::xy_from_rgb(rgb);
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send((tx, Change::AllXY { bri, xy }))
            .expect("manage_bridge should never return");

        let res = rx.blocking_recv().expect(
            "managed_bridge should not crash and thus not drop the receive end of the channel",
        );
        res
    }
}
