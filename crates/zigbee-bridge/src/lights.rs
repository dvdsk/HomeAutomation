#![allow(clippy::missing_panics_doc)]
use tokio::sync::mpsc;

use self::state::Change;

// TODO: make private once system is ported
pub use conversion::{denormalize, kelvin_to_mired, mired_to_kelvin, normalize};

mod cached_bridge;
mod conversion;
mod state;

#[derive(Debug, Clone)]
pub struct Controller {
    change_sender: mpsc::UnboundedSender<(String, state::Change)>,
}

impl Controller {
    #[must_use]
    pub fn start_bridge() -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = cached_bridge::run(change_receiver);
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }

    pub fn set_on(&self, friendly_name: &str) {
        self.change_sender
            .send((friendly_name.to_string(), Change::On(true)))
            .expect("Sender should never be dropped");
    }

    pub fn set_off(&self, friendly_name: &str) {
        self.change_sender
            .send((friendly_name.to_string(), Change::On(false)))
            .expect("Sender should never be dropped");
    }

    // TODO: document, is this lumen? Maybe use newtype pattern with a new lumen
    // struct <dvdsk noreply@davidsk.dev>
    pub fn set_brightness(&self, friendly_name: &str, brightness: f64) {
        self.change_sender
            .send((friendly_name.to_string(), Change::Brightness(brightness)))
            .expect("Sender should never be dropped");
    }

    pub fn set_color_temp(&self, friendly_name: &str, kelvin: usize) {
        self.change_sender
            .send((friendly_name.to_string(), Change::ColorTemp(kelvin)))
            .expect("Sender should never be dropped");
    }

    // pub fn set_color_xy(&self, friendly_name: &str, xy: (f64, f64)) {
    //     self.change_sender
    //         .send((friendly_name.to_string(), Change::ColorXy(xy)))
    //         .expect("Sender should never be dropped");
    // }
}
