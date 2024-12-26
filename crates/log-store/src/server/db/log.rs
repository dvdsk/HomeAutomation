use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::iter;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use byteseries::file::OpenError as FileOpenError;
use byteseries::{series, ByteSeries};
use color_eyre::eyre::Context;
use color_eyre::{Result, Section};
use protocol::Device;
use serde::{Deserialize, Serialize};
use series::data::OpenError as DataOpenError;
use series::Error::Open;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

use crate::api::{self, ErrorEvent, GetLogResponse};

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct Log {
    #[derivative(Debug = "ignore")]
    history: ByteSeries,
    current: CurrentError,
}

#[derive(Debug)]
struct CurrentError {
    file: std::fs::File,
    value: Option<(jiff::Timestamp, protocol::Error)>,
}

impl CurrentError {
    fn open_or_create(path: &Path) -> Result<Self> {
        let path = path.with_extension("current_error");
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .wrap_err("Could not open or create on disk backup")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .wrap_err("Failed to read from on disk backup")?;

        let value = if buf.is_empty() {
            None
        } else {
            match bincode::deserialize(&buf) {
                Ok(value) => Some(value),
                Err(err) => {
                    tracing::debug!("Data was: {buf:?}");
                    warn!(
                        "On disk backup failed to deserialize, setting to None, \
                        error was: {err}"
                    );
                    file.set_len(0).wrap_err(
                        "Could not set on disk backup to value None after \
                        failing deserialize",
                    )?;
                    None
                }
            }
        };

        Ok(Self { file, value })
    }

    fn take(&mut self) -> Result<Option<(jiff::Timestamp, protocol::Error)>> {
        self.file.set_len(0).wrap_err(
            "Could not set on disk backup of current error to None",
        )?;
        Ok(self.value.take())
    }

    fn set(&mut self, report: protocol::Error) -> Result<()> {
        let value = (jiff::Timestamp::now(), report);
        let bytes = bincode::serialize(&value)
            .wrap_err("could not serialize current error")?;
        self.file
            .set_len(0)
            .wrap_err("Could not clear file prior to backing up value")?;
        self.file
            .write_all(&bytes)
            .wrap_err("could not backup current error to disk")?;
        self.file
            .flush()
            .wrap_err("failed to flush current err backup to disk")?;

        self.value = Some(value);
        Ok(())
    }

    fn get(&self) -> &Option<(jiff::Timestamp, protocol::Error)> {
        &self.value
    }
}

impl Log {
    #[instrument]
    pub fn open_or_create(dir: &Path, device: &Device) -> Result<Self> {
        let path = base_path(device);
        let path = dir.join(path);

        let payload_size = protocol::Error::max_size();
        let header = format!(
            "Bincode encoded error logs for {device:?}. \
            Each line has a size: {payload_size} + 2"
        );

        let res = ByteSeries::builder()
            .payload_size(payload_size)
            .with_header(header.as_bytes().to_vec())
            .open(&path);

        let history = match res {
            Ok((byteseries, _)) => byteseries,
            Err(Open(DataOpenError::File {
                source: FileOpenError::Io(e),
                ..
            })) if e.kind() == io::ErrorKind::NotFound => {
                if let Some(dirs) = path.parent() {
                    std::fs::create_dir_all(dirs)
                        .wrap_err("Could not create dirs structure for reading")
                        .with_note(|| format!("dirs: {}", dirs.display()))?;
                }
                info!("creating new byteseries");
                ByteSeries::builder()
                    .payload_size(payload_size)
                    .with_header(header.into_bytes())
                    .create_new(true)
                    .open(&path)
                    .wrap_err("Could not create new byteseries")
                    .with_note(|| format!("path: {}", path.display()))?
                    .0
            }
            Err(e) => {
                return Err(e)
                    .wrap_err("Could not open existing byteseries")
                    .with_note(|| format!("path: {}", path.display()))
            }
        };

        Ok(Self {
            history,
            current: CurrentError::open_or_create(&path)
                .wrap_err("Could not setup current error store")
                .with_note(|| format!("device: {device:?}"))
                .with_note(|| format!("path: {}", path.display()))?,
        })
    }

    #[instrument]
    pub fn set_err(&mut self, new_report: protocol::Error) -> Result<()> {
        if let Some((started, report)) = self.current.get() {
            if report == &new_report {
                return Ok(());
            }
            let line = StoredErrorEvent {
                end: jiff::Timestamp::now(),
                error: report.clone(),
            };
            let line = bincode::serialize(&line)
                .wrap_err("Could not serialize ErrorEvent")?;
            let payload_size = protocol::Error::max_size();
            let line: Vec<_> = line
                .into_iter()
                .chain(iter::repeat(0))
                .take(payload_size)
                .collect();

            self.history
                .push_line(started.as_second() as u64, line)
                .wrap_err("Could not push new ErrorEvent into history")?;
        }

        debug!("Registered new error: {new_report}");
        self.current
            .set(new_report)
            .wrap_err("Failed to set new error in current error store")
    }

    #[instrument]
    fn clear(&mut self) -> Result<()> {
        self.current
            .take()
            .wrap_err("failed to set value of current error to None")?;
        debug!("Cleared any error");
        Ok(())
    }

    fn get(
        &mut self,
        range: RangeInclusive<jiff::Timestamp>,
    ) -> GetLogResponse {
        use byteseries::seek::Error::{EmptyFile, StopBeforeData};
        use byteseries::series::Error::InvalidRange;
        const MAX_IN_ONE_READ: usize = 200;

        let ts_range = RangeInclusive::new(
            range.start().as_second() as u64,
            range.end().as_second() as u64,
        );

        let current = self
            .current
            .value
            .as_ref()
            .map(|(ts, event)| api::ErrorEvent {
                start: *ts,
                end: None,
                error: event.clone(),
            })
            .into_iter();

        let mut timestamps = Vec::new();
        let mut data = Vec::new();
        match self.history.read_first_n(
            MAX_IN_ONE_READ,
            &mut Decoder,
            ts_range,
            &mut timestamps,
            &mut data,
        ) {
            Ok(()) => (),
            Err(InvalidRange(StopBeforeData | EmptyFile)) => {
                return GetLogResponse::All(current.collect())
            }
            Err(other) => {
                let report = color_eyre::eyre::Report::new(other)
                    .wrap_err("Could not read log events from disk");
                let report = format!("{report:?}");
                return GetLogResponse::Err(report);
            }
        }

        let mut res: Vec<_> = timestamps
            .into_iter()
            .zip(data)
            .map(|(start, StoredErrorEvent { end, error })| api::ErrorEvent {
                start: jiff::Timestamp::from_second(start as i64)
                    .expect("was a jiff::Timestamp before it became a u64"),
                end: Some(end),
                error,
            })
            .collect();

        if res.len() < MAX_IN_ONE_READ {
            res.extend(current.filter(|ErrorEvent { start, .. }| {
                start >= range.start() && start <= range.end()
            }));

            GetLogResponse::All(res)
        } else {
            GetLogResponse::Partial(res)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredErrorEvent {
    end: jiff::Timestamp,
    error: protocol::Error,
}

#[derive(Debug)]
struct Decoder;
impl byteseries::Decoder for Decoder {
    type Item = StoredErrorEvent;

    fn decode_payload(&mut self, payload: &[u8]) -> Self::Item {
        bincode::deserialize(payload)
            .expect("if its successfully serialized it should deserialize")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Logs(pub(crate) Arc<Mutex<HashMap<protocol::Device, Log>>>);

impl Logs {
    pub async fn set_err(
        &self,
        report: protocol::Error,
        log_dir: &Path,
    ) -> Result<()> {
        let device = report.device();
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(&device) {
            log.set_err(report)?;
        } else {
            let mut log = Log::open_or_create(log_dir, &device)
                .wrap_err("Failed to open or create error Log")?;
            log.set_err(report)?;
            map.insert(device, log);
        }
        Ok(())
    }

    pub async fn get(
        &self,
        device: &protocol::Device,
        range: RangeInclusive<jiff::Timestamp>,
    ) -> api::GetLogResponse {
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(device) {
            log.get(range)
        } else {
            api::GetLogResponse::All(Vec::new())
        }
    }

    pub(crate) async fn clear_err(
        &self,
        device: protocol::Device,
    ) -> Result<()> {
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(&device) {
            log.clear()?;
        }
        Ok(())
    }

    pub(crate) async fn list_devices(&self) -> Vec<Device> {
        let map = self.0.lock().await;
        map.iter().map(|(key, _)| key.clone()).collect()
    }
}

/// Relative path without extension
fn base_path(device: &protocol::Device) -> PathBuf {
    use protocol::reading::tree::{Item, Tree};
    use protocol::reading::Info;

    let mut parts = Vec::new();
    let mut current = device
        .info()
        .affects_readings
        .first()
        .expect("a device has at least one reading it affects")
        as &dyn Tree;
    loop {
        match current.inner() {
            Item::Leaf(Info { device, .. }) => {
                parts.push(device.info().name.to_lowercase());
                break;
            }
            Item::Node(inner) => {
                parts.push(current.name().to_lowercase());
                current = inner;
                continue;
            }
        }
    }
    parts.into_iter().collect()
}
