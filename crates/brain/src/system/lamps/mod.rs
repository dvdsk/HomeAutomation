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
    ip: String,
    rx: mpsc::UnboundedReceiver<(oneshot::Sender<Result<(), ApplyChangeError>>, Change)>,
) {
    let stream = stream::unfold(rx, |mut rx| async move {
        let yielded = rx.recv().await;
        yielded.map(|item| (item, rx))
    });

    let mut prev_error = String::new();
    let stream = stream.peekable();
    pin_mut!(stream);
    loop {
        let error = match eventually_consistent_bridge::CachedBridge::try_init(&ip).await {
            Ok(bridge) => {
                eventually_consistent_bridge::process_lamp_changes(&mut stream, bridge).await
            }
            Err(e) => e,
        };

        if prev_error != error.to_string() {
            error!("could not connect to bridge: {error:?}");
            prev_error = error.to_string();
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[derive(Debug, Clone)]
pub struct Lighting {
    tx: mpsc::UnboundedSender<(oneshot::Sender<Result<(), ApplyChangeError>>, Change)>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not get lights from bridge")]
    GettingLights(hueclient::HueError),
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
        pub async fn $name(&mut self, $($($arg: $type),+)?) -> Result<(), ApplyChangeError> {
            // todo use oneshot to get error back from other side
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.tx.send((tx, Change::$change$({$($arg),+})?)).expect("managed_bridge should channel should never drop as managed bridge task should never end");
            let res = rx.await.expect("managed_bridge should not crash and thus not drop the receive end of the channel");
            res
        }
    };
}

#[allow(dead_code)]
impl Lighting {
    pub fn start_init(ip: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let manage_bridge = manage_bridge(ip, rx);
        tokio::task::spawn(manage_bridge);
        Self { tx }
    }

    light_fn! {all_off, AllOff}
    light_fn! {all_on, AllOn}
    light_fn! {single_off, Off; name: &'static str}
    light_fn! {single_on, On; name: &'static str}

    light_fn! {set_ct, CtBri; name: &'static str, bri: u8, ct: u16 }
    light_fn! {set_xy, XyBri; name: &'static str, bri: u8, xy: (f32, f32)}
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
