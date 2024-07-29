use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use color_eyre::Result;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Log(VecDeque<(jiff::Timestamp, protocol::Error)>);

impl Log {
    pub fn new(first_report: protocol::Error) -> Self {
        let mut this = Self(VecDeque::new());
        this.push(first_report);
        this
    }

    pub fn push(&mut self, report: protocol::Error) {
        const MAX_LENGTH: usize = 100;
        self.0.truncate(MAX_LENGTH - 1);
        self.0.push_front((jiff::Timestamp::now(), report))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Logs(pub(crate) Arc<Mutex<HashMap<protocol::Device, Log>>>);

impl Logs {
    pub async fn store(&self, report: protocol::Error) -> Result<()> {
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(&report.device()) {
            log.push(report);
        } else {
            map.insert(report.device(), Log::new(report));
        }
        Ok(())
    }

    pub async fn get(&self, device: &protocol::Device) -> Vec<(jiff::Timestamp, protocol::Error)> {
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(device) {
            let (logs, more_logs) = log.0.as_slices();
            let mut logs = logs.to_vec();
            logs.extend_from_slice(more_logs);
            logs
        } else {
            Vec::new()
        }
    }
}
