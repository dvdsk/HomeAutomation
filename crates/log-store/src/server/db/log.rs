use std::collections::HashMap;
use std::io::{self, Read, Write};
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
use tracing::{info, warn};

use crate::api::{self, ErrorEvent};

#[derive(Debug)]
pub(crate) struct Log {
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
        self.file
            .set_len(0)
            .wrap_err("Could not set on disk backup of current error to None")?;
        Ok(self.value.take())
    }

    fn set(&mut self, report: protocol::Error) -> Result<()> {
        let value = (jiff::Timestamp::now(), report);
        let bytes = bincode::serialize(&value).wrap_err("could not serialize current error")?;
        self.file
            .set_len(0)
            .wrap_err("Could not clear file prior to backing up value")?;
        self.file
            .write_all(&bytes)
            .wrap_err("could not backup current error to disk")?;
        self.file
            .flush()
            .wrap_err("failed to flush current err backup to disk")?;
        Ok(())
    }

    fn get(&self) -> &Option<(jiff::Timestamp, protocol::Error)> {
        &self.value
    }
}

impl Log {
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
            .with_header(header.clone())
            .open(&path);

        let history = match res {
            Ok((byteseries, _)) => byteseries,
            Err(Open(DataOpenError::File(FileOpenError::Io(e))))
                if e.kind() == io::ErrorKind::NotFound =>
            {
                if let Some(dirs) = path.parent() {
                    std::fs::create_dir_all(dirs)
                        .wrap_err("Could not create dirs structure for reading")
                        .with_note(|| format!("dirs: {}", dirs.display()))?;
                }
                info!("creating new byteseries");
                ByteSeries::builder()
                    .payload_size(payload_size)
                    .with_header(header)
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

    pub fn set_err(&mut self, new_report: protocol::Error) -> Result<()> {
        if let Some((started, report)) = self.current.get() {
            if report == &new_report {
                return Ok(());
            }
            let line = StoredErrorEvent {
                end: jiff::Timestamp::now(),
                error: report.clone(),
            };
            let line = bincode::serialize(&line).wrap_err("Could not serialize ErrorEvent")?;
            self.history
                .push_line(started.as_second() as u64, line)
                .wrap_err("Could not push new ErrorEvent into history")?;
        }

        self.current
            .set(new_report)
            .wrap_err("Failed to set new error in current error store")
    }

    fn clear(&mut self) -> Result<()> {
        self.current
            .take()
            .wrap_err("failed to set value of current error to None")?;
        Ok(())
    }

    fn get_all(&mut self) -> Result<ApiResult<Vec<ErrorEvent>>> {
        let mut timestamps = Vec::new();
        let mut data = Vec::new();
        let n_lines = self.history.n_lines_between(..).wrap_err(
            "Could not check if there is not too much data \
            between the requested points",
        )?;
        if n_lines > 200 {
            return Ok(Err(api::GetLogError::TooMuchData {
                max: 200,
                found: n_lines,
            }));
        }
        self.history
            .read_all(.., &mut Decoder, &mut timestamps, &mut data)
            .wrap_err("Could not read log events form disk")?;

        let res = timestamps
            .into_iter()
            .zip(data)
            .map(|(start, StoredErrorEvent { end, error })| api::ErrorEvent {
                start: jiff::Timestamp::from_second(start as i64)
                    .expect("was a jiff::Timestamp before it became a u64"),
                end: Some(end),
                error,
            })
            .collect();
        Ok(Ok(res))
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
        bincode::deserialize(payload).expect("if its successfully serialized it should deserialize")
    }
}

type ApiResult<T> = std::result::Result<T, api::GetLogError>;

#[derive(Debug, Clone)]
pub(crate) struct Logs(pub(crate) Arc<Mutex<HashMap<protocol::Device, Log>>>);

impl Logs {
    pub async fn set_err(&self, report: protocol::Error, log_dir: &Path) -> Result<()> {
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

    pub async fn get(&self, device: &protocol::Device) -> ApiResult<Vec<api::ErrorEvent>> {
        let mut map = self.0.lock().await;
        if let Some(log) = map.get_mut(device) {
            match log.get_all() {
                Err(report) => Err(api::GetLogError::InternalError(report.to_string())),
                Ok(res) => res,
            }
        } else {
            Ok(Vec::new())
        }
    }

    pub(crate) async fn clear_err(&self, device: protocol::Device) -> Result<()> {
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

/// relative path without extension
fn base_path(device: &protocol::Device) -> PathBuf {
    use protocol::reading::tree::{Item, Tree};
    use protocol::reading::Info;

    let mut parts = Vec::new();
    let mut current = device
        .info()
        .affects_readings
        .first()
        .expect("a device has at least one reading it affects") as &dyn Tree;
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
