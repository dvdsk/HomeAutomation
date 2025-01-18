use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use color_eyre::{Result, Section};
use tokio::sync::Mutex;

pub mod series;
use series::Series;

use crate::api;

#[derive(Debug, Clone)]
pub(crate) struct Data(
    pub(crate) Arc<Mutex<HashMap<protocol::Device, Series>>>,
);

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
    ) -> Result<api::Data, api::GetDataError> {
        use byteseries::seek::Error as Se;
        use byteseries::series::Error as Be;

        let key = reading.device();
        let mut all_series = self.0.lock().await;
        let series = all_series.get_mut(&key).ok_or_else(|| {
            api::GetDataError::NotInStore {
                reading: reading.clone(),
            }
        })?;

        let (time, mut data) =
            series
                .read(&[reading], start, end, n)
                .map_err(|e| match e {
                    Be::InvalidRange(Se::NotFound) => {
                        api::GetDataError::NotFound
                    }
                    Be::InvalidRange(Se::EmptyFile) => {
                        api::GetDataError::EmptyFile
                    }
                    Be::InvalidRange(Se::StartAfterData { .. }) => {
                        api::GetDataError::StartAfterData
                    }
                    Be::InvalidRange(Se::StopBeforeData) => {
                        api::GetDataError::StopBeforeData
                    }
                    _ => api::GetDataError::ReadingFromStore(e.to_string()),
                })?;
        Ok(api::Data {
            time,
            values: data.pop().expect("one reading is put in so one comes out"),
        })
    }
}
