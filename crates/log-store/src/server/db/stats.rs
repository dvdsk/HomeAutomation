use color_eyre::eyre::WrapErr;
use color_eyre::{Result, Section};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::api::{self, Percentile};

#[derive(Debug)]
pub(crate) struct Histogram {
    last_reading: Instant,
    histogram: hdrhistogram::Histogram<u64>,
}

impl Histogram {
    fn new() -> Result<Self> {
        Ok(Self {
            last_reading: Instant::now(),
            histogram: hdrhistogram::Histogram::new_with_bounds(1, 24 * 60 * 60 * 1000, 2)
                .wrap_err("Could not create empty histogram")?,
        })
    }
    fn increment(&mut self) -> Result<()> {
        let val = self.last_reading.elapsed().as_millis();
        self.last_reading = Instant::now();
        self.histogram
            .record(val as u64)
            .wrap_err("Could not record event")
            .with_note(|| format!("duration was: {val}ms"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Stats(pub(crate) Arc<Mutex<HashMap<protocol::Device, Histogram>>>);

impl Stats {
    pub async fn increment(&self, device: protocol::Device) -> Result<()> {
        let mut map = self.0.lock().await;
        if let Some(hist) = map.get_mut(&device) {
            hist.increment()?;
        } else {
            map.insert(device, Histogram::new()?);
        }
        Ok(())
    }

    pub(crate) async fn get(
        &self,
        device: &protocol::Device,
    ) -> Result<Vec<crate::api::Percentile>, api::GetStatsError> {
        let mut map = self.0.lock().await;
        Ok(if let Some(timings) = map.get_mut(device) {
            timings
                .histogram
                .iter_quantiles(1)
                .map(|it| Percentile {
                    bucket_ends: it.value_iterated_to(),
                    percentile: it.percentile(),
                    count_in_bucket: it.count_at_value(),
                })
                .collect()
        } else {
            Vec::new()
        })
    }
}
