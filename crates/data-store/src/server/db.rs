use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use time::OffsetDateTime;
use tokio::sync::Mutex;

use color_eyre::eyre::WrapErr;
use color_eyre::{Result, Section};
use data_server::{AsyncSubscriber, SubMessage};

mod series;
use series::Series;

use crate::api;

// TODO make resistant to data_server going down
pub(crate) async fn run(data_server_addr: SocketAddr, data: Data) -> Result<()> {
    let mut sub = AsyncSubscriber::connect(data_server_addr)
        .await
        .wrap_err("failed to connect")
        .with_suggestion(|| format!("verify the server is listening on: {data_server_addr}"))?;

    loop {
        let res = match sub
            .next()
            .await
            .wrap_err("Error getting next reading from server")?
        {
            SubMessage::Reading(reading) => series::store(&data, &reading).await,
            SubMessage::ErrorReport(_) => continue,
        };

        if let Err(e) = res {
            tracing::error!("Error processing new reading: {e}");
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
            .flat_map(|device| device.affected_readings())
            .cloned()
            .collect()
    }
    pub(crate) async fn get(
        &self,
        reading: protocol::Reading,
        start: OffsetDateTime,
        end: OffsetDateTime,
        n: usize,
    ) -> Result<Vec<(OffsetDateTime, f32)>, api::ServerError> {
        let key = reading.device();
        let series = self.0
            .lock()
            .await
            .get_mut(&key)
            .ok_or(api::ServerError::NotInStore { reading })?;
    
        // series.
        
    }
}
