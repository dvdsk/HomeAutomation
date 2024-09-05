use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use data_server::api::subscriber::ReconnectingClient;
use data_server::api::subscriber::SubMessage;

use tokio::sync::Mutex;
use color_eyre::{Result, Section};

mod series;
use series::Series;

use crate::api;

pub(crate) async fn run(data_server_addr: SocketAddr, data: Data, data_dir: &Path) -> Result<()> {
    let mut sub =
        ReconnectingClient::new(data_server_addr, "ha-data-store".to_string()).subscribe();

    let mut recently_logged = (Instant::now(), String::new());
    loop {
        let msg = sub.next().await;
        let SubMessage::Reading(reading) = msg else {
            continue;
        };

        let res = series::store(&data, &reading, data_dir)
            .await
            .with_note(|| format!("reading: {reading:?}"));

        const FIVE_MIN: Duration = Duration::from_secs(60 * 5);
        if let Err(report) = res {
            let e = format!("got error with report: {report:?}");
            tracing::warn!("test: {e}");
            if recently_logged.1 == e && recently_logged.0.elapsed() <= FIVE_MIN {
                continue;
            } else {
                tracing::error!("Error processing new reading: {e}");
                recently_logged = (Instant::now(), e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Data(pub(crate) Arc<Mutex<HashMap<protocol::Device, Series>>>);

impl Data {
    pub(crate) async fn list(&self) -> Vec<protocol::Reading> {
        self.0
            .lock()
            .await
            .keys()
            .flat_map(|dev| dev.info().affects_readings)
            .cloned()
            .collect()
    }
    pub(crate) async fn get(
        &self,
        reading: protocol::Reading,
        start: jiff::Timestamp,
        end: jiff::Timestamp,
        n: usize,
    ) -> Result<(Vec<jiff::Timestamp>, Vec<f32>), api::ServerError> {
        let key = reading.device();
        let mut all_series = self.0.lock().await;
        let series = all_series
            .get_mut(&key)
            .ok_or_else(|| api::ServerError::NotInStore {
                reading: reading.clone(),
            })?;

        let (time, mut data) = series
            .read(&[reading], start, end, n)
            .map_err(|e| api::ServerError::ReadingFromStore(e.to_string()))?;
        Ok((
            time,
            data.pop().expect("one reading is put in so one comes out"),
        ))
    }
}
