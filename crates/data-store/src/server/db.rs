use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use data_server::subscriber::reconnecting;
use time::OffsetDateTime;
use tokio::sync::Mutex;

use color_eyre::Result;
use data_server::SubMessage;

mod series;
use series::Series;

use crate::api;

pub(crate) async fn run(data_server_addr: SocketAddr, data: Data, data_dir: &Path) -> Result<()> {
    let mut sub = reconnecting::Subscriber::new(data_server_addr, "ha-data-store".to_string());

    loop {
        let msg = sub.next_msg().await;
        tracing::trace!("got msg: {msg:?}");

        let res = match msg {
            SubMessage::Reading(reading) => series::store(&data, &reading, data_dir).await,
            SubMessage::ErrorReport(_) => continue,
        };

        if let Err(e) = res {
            tracing::error!("Error processing new reading: {e:?}");
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
            .flat_map(protocol::Device::affected_readings)
            .cloned()
            .collect()
    }
    pub(crate) async fn get(
        &self,
        reading: protocol::Reading,
        start: OffsetDateTime,
        end: OffsetDateTime,
        n: usize,
    ) -> Result<(Vec<OffsetDateTime>, Vec<f32>), api::ServerError> {
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
