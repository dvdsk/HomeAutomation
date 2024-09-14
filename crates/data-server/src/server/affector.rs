use std::sync::{Arc, Mutex};

use color_eyre::Result;
use protocol::Affector;
use slotmap::{DefaultKey, SlotMap};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::{Receiver, Sender};

use tracing::{instrument, warn};

#[derive(Debug)]
pub(crate) struct Registration {
    tx: tokio::sync::mpsc::Sender<protocol::Affector>,
    controls: Vec<protocol::Affector>,
}

impl Registration {
    fn update(&mut self, new: Affector) {
        let curr = self
            .controls
            .iter_mut()
            .find(|a| a.is_same_as(&new))
            .unwrap();
        *curr = new;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Registar(Arc<Mutex<SlotMap<DefaultKey, Registration>>>);

impl Registar {
    pub(crate) fn register(&self, tx: Sender<Affector>, affectors: Vec<Affector>) -> DefaultKey {
        let mut this = self.0.lock().expect("nothing should panic");

        let to_remove: Vec<_> = this
            .iter_mut()
            .filter(|(_, reg)| {
                reg.controls.iter().any(|control| {
                    affectors
                        .iter()
                        .any(|affector| affector.is_same_as(control))
                })
            })
            .map(|(key, _)| key)
            .collect();

        for key in to_remove {
            this.remove(key)
                .expect("held lock so can not have been removed");
        }

        this.insert(Registration {
            tx,
            controls: affectors,
        })
    }

    pub(crate) fn remove(&self, key: DefaultKey) {
        let mut this = self.0.lock().expect("nothing should panic");
        let _ = this.remove(key); // Could have been removed by register
    }

    pub(crate) fn activate(&self, order: Affector) -> Result<(), Offline> {
        tracing::info!("client is trying to activate: {order:?}");
        let mut this = self.0.lock().expect("nothing should panic");
        for possible_controller in this.iter_mut().map(|(_, reg)| reg).filter(|reg| {
            reg.controls
                .iter()
                .any(|control| control.is_same_as(&order))
        }) {
            if possible_controller.tx.try_send(order).is_ok() {
                possible_controller.update(order);
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

#[instrument(skip_all)]
pub(super) async fn control_affectors(mut writer: OwnedWriteHalf, mut rx: Receiver<Affector>) {
    while let Some(new_order) = rx.recv().await {
        let buf = new_order.encode();
        if let Err(e) = writer.write_all(&buf).await {
            warn!("Could not send affector order: {e}");
            break;
        }
    }
}
