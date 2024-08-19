use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use color_eyre::Result;
use futures::FutureExt;
use futures_concurrency::future::Race;
use protocol::Affector;
use slotmap::{DefaultKey, SlotMap};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::{Receiver, Sender};

use tracing::warn;

#[derive(Debug)]
pub(crate) struct Registration {
    tx: tokio::sync::mpsc::Sender<protocol::Affector>,
    controls: HashSet<protocol::Affector>,
}

#[derive(Debug, Clone, Default)]
pub struct Registar(Arc<Mutex<SlotMap<DefaultKey, Registration>>>);

impl Registar {
    fn register(&self, tx: Sender<Affector>) -> DefaultKey {
        let mut this = self.0.lock().expect("nothing should panic");
        this.insert(Registration {
            tx,
            controls: HashSet::new(),
        })
    }

    fn update_affectors(&self, key: DefaultKey, list: &[Affector]) {
        let mut this = self.0.lock().expect("nothing should panic");
        let registration = this
            .get_mut(key)
            .expect("items are removed when track_and_control_affectors only");
        registration.controls.extend(list.iter());
    }

    fn remove(&self, key: DefaultKey) {
        let mut this = self.0.lock().expect("nothing should panic");
        this.remove(key).expect("things are only removed once");
    }

    pub(crate) fn activate(&self, affector: Affector) -> Result<(), Offline> {
        let this = self.0.lock().expect("nothing should panic");
        for possible_controller in this
            .iter()
            .map(|(_, reg)| reg)
            .filter(|reg| reg.controls.contains(&affector))
        {
            if possible_controller.tx.try_send(affector).is_ok() {
                return Ok(());
            }
        }

        Err(Offline)
    }

    pub(crate) fn list(&self) -> Vec<Affector> {
        let this = self.0.lock().expect("nothing should panic");
        this.iter()
            .flat_map(|(_, reg)| reg.controls.iter())
            .cloned()
            .collect()
    }
}

pub struct Offline;

pub(super) async fn track_and_control_affectors(
    mut writer: OwnedWriteHalf,
    mut update_from_same_node: Receiver<protocol::Device>,
    registar: Registar,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let key = registar.register(tx);

    loop {
        let new_update = update_from_same_node.recv().map(Res::from);
        let new_order = rx
            .recv()
            .map(|opt| opt.expect("Register never drops"))
            .map(Res::Order);

        let res = (new_update, new_order).race().await;
        match res {
            Res::Disconnected => break,
            Res::Update(device) => {
                registar.update_affectors(key, device.info().affectors);
            }
            Res::Order(affector) => {
                let buf = affector.encode();
                if let Err(e) = writer.write_all(&buf).await {
                    warn!("Could not send affector order: {e}");
                    break;
                }
            }
        }
    }

    registar.remove(key);
}

enum Res {
    Disconnected,
    Update(protocol::Device),
    Order(protocol::Affector),
}

impl Res {
    fn from(val: Option<protocol::Device>) -> Self {
        match val {
            Some(device) => Self::Update(device),
            None => Self::Disconnected,
        }
    }
}
