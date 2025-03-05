use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use byteseries::{downsample, series, ByteSeries};
use color_eyre::eyre::{eyre, OptionExt, WrapErr};
use color_eyre::{Result, Section};
use protocol::reading::tree::{Item, Tree};
use protocol::{reading, IsSameAs};
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use byteseries::file::OpenError as FileOpenError;
use series::data::OpenError as DataOpenError;
use series::Error::Open;

pub mod bitspec;
mod resampler;

use self::resampler::Resampler;

use super::Data;

#[derive(Debug)]
pub(crate) struct Meta {
    reading: protocol::Reading,
    pub(crate) field: bitspec::Field<f32>,
    set_at: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct Series {
    line: Vec<u8>,
    meta_list: Vec<Meta>,
    last_timestamp_pushed: Option<u64>,
    byteseries: ByteSeries,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Header {
    pub(crate) readings: Vec<protocol::Reading>,
    pub(crate) encoding: Vec<bitspec::Field<f32>>,
}

impl Header {
    fn serialized(&self) -> Result<Vec<u8>> {
        let config = ron::ser::PrettyConfig::new();
        ron::ser::to_string_pretty(self, config)
            .map(|s| s.into_bytes())
            .wrap_err("Could not serialize header")
    }
}

impl Series {
    #[instrument]
    fn open_or_create(reading: &protocol::Reading, dir: &Path) -> Result<Self> {
        let readings = reading.device().info().affects_readings;
        let specs = to_speclist(readings);
        let fields = bitspec::speclist_to_fields(specs);
        let (meta_list, payload_size) =
            meta_list_and_payload_size(readings, &fields);

        let expected_header = Header {
            readings: readings.to_vec(),
            encoding: fields.clone(),
        }
        .serialized()?;

        let (resampler, configs) = resample_setup(&fields, payload_size);

        let path = base_path(reading);
        let path = dir.join(path);
        let res = ByteSeries::builder()
            .payload_size(payload_size)
            .with_downsampled_cache(resampler.clone(), configs.clone())
            .with_header(expected_header.clone())
            .open(&path);

        let byteseries = match res {
            Ok((byteseries, _)) => byteseries,
            Err(Open(DataOpenError::File {
                source: FileOpenError::Io(e),
                ..
            })) if e.kind() == io::ErrorKind::NotFound => {
                if let Some(dirs) = path.parent() {
                    create_dir_all(dirs)
                        .wrap_err("Could not create dirs structure for reading")
                        .with_note(|| format!("dirs: {}", dirs.display()))?;
                }

                ByteSeries::builder()
                    .payload_size(payload_size)
                    .with_downsampled_cache(resampler, configs)
                    .with_header(expected_header)
                    .create_new(true)
                    .open(&path)
                    .wrap_err("Could not create new byteseries")
                    .with_note(|| format!("path: {}", path.display()))
                    .map(|(db, _)| db)?
            }
            Err(e) => {
                return Err(e)
                    .wrap_err("Could not open existing byteseries")
                    .with_note(|| format!("path: {}", path.display()))?
            }
        };

        Ok(Self {
            line: vec![0; payload_size],
            meta_list,
            last_timestamp_pushed: None,
            byteseries,
        })
    }

    #[instrument(skip(self))]
    fn append(&mut self, reading: &protocol::Reading) -> Result<()> {
        let res = reading
            .device()
            .info()
            .affects_readings
            .iter()
            .map(|r| r.leaf().branch_id)
            .position(|id_in_list| id_in_list == reading.leaf().branch_id);
        let index = wrap_inder_err(res, reading)?;

        let meta = &mut self.meta_list[index];
        if !reading.leaf().range.contains(&reading.leaf().val) {
            return Err(eyre!(
                "value {} is out of range: {:?}",
                reading.leaf().val,
                reading.leaf().range
            ));
        }

        meta.field.encode(reading.leaf().val, &mut self.line);
        meta.set_at = Some(Instant::now());

        let max_interval = reading.device().info().max_sample_interval;

        if self
            .meta_list
            .iter()
            .map(|Meta { set_at, .. }| set_at)
            .all(|set| set.is_some_and(|set| set.elapsed() < max_interval))
        {
            let time = jiff::Timestamp::now();

            let device_info = reading.leaf().device.info();
            let scale_factor = millis_to_minimal_representation(device_info);
            let scaled_time = time.as_millisecond() as u64 / scale_factor;
            let new_ts = scaled_time;

            if self.last_timestamp_pushed.is_some_and(|ts| ts == new_ts) {
                tracing::trace!("Skipping datapoint with same timestamp");
                return Ok(());
            } else {
                self.last_timestamp_pushed = Some(new_ts);
            }

            self.byteseries
                .push_line(new_ts, &self.line)
                .wrap_err("Could not write to timeseries on disk")?;
            self.line.fill(0);

            for meta in &mut self.meta_list {
                meta.set_at = None;
            }
        }

        Ok(())
    }

    /// # Panics
    /// If any of the requested readings are not part of this series.
    #[instrument(skip(self))]
    pub fn read(
        &mut self,
        readings: &[protocol::Reading],
        start: jiff::Timestamp,
        end: jiff::Timestamp,
        n: usize,
    ) -> Result<(Vec<jiff::Timestamp>, Vec<Vec<f32>>), byteseries::series::Error>
    {
        let device_info = readings
            .first()
            .expect("There is at least one reading to read")
            .leaf()
            .device
            .info();
        let scale_factor = millis_to_minimal_representation(device_info);
        let start = start.as_millisecond() as u64 / scale_factor;
        let end = end.as_millisecond() as u64 / scale_factor;
        let range = start..=end;
        let fields = readings
            .iter()
            .map(|requested| {
                self.meta_list
                    .iter()
                    .find(|meta| requested.is_same_as(&meta.reading))
                    .inspect(|meta| trace!("meta used for decoding: {meta:?}"))
                    .map(|meta| meta.field.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "caller of read makes sure all readings are part of this \
                        series.\n\tseries: {:?},\n\trequested: {:?}",
                            self.meta_list, readings
                        )
                    })
            })
            .collect();
        let mut resampler = Resampler::from_fields(fields, self.line.len());

        let mut timestamps = Vec::with_capacity(n * 2);
        let mut interleaved_data = Vec::with_capacity(n * 2);

        self.byteseries.read_n(
            n,
            range,
            &mut resampler,
            &mut timestamps,
            &mut interleaved_data,
            false,
        )?;

        let time = timestamps
            .into_iter()
            .map(|ts| {
                let millis = ts * scale_factor;
                jiff::Timestamp::from_millisecond(millis as i64)
                    .expect("timestamps are between MIN and MAX times of Timestamp type")
            })
            .collect();

        let len = interleaved_data
            .first()
            .expect("read returns an error if there is not data")
            .len();
        let mut data = vec![Vec::new(); len];
        for interleaved in interleaved_data {
            for (interleaved, data) in
                interleaved.into_iter().zip(data.iter_mut())
            {
                data.push(interleaved);
            }
        }
        Ok((time, data))
    }
}

fn wrap_inder_err(
    res: Option<usize>,
    reading: &protocol::Reading,
) -> Result<usize> {
    res.ok_or_eyre(
        "Could not find reading index, branch id does not match 
                any in readings affected by the device",
    )
    .suggestion("In protocol lib, is every reading variant in the affected readings list?")
    .with_note(|| format!("reading: {reading:?}"))
    .with_note(|| format!("reading branch id: {:?}", reading.leaf().branch_id))
    .with_note(|| {
        format!(
            "affected readings: {:?}",
            reading
                .device()
                .info()
                .affects_readings
                .into_iter()
                .map(|r| r.leaf())
                .collect::<Vec<_>>()
        )
    })
}

pub(crate) fn meta_list_and_payload_size(
    readings: &[protocol::Reading],
    fields: &[bitspec::Field<f32>],
) -> (Vec<Meta>, usize) {
    let meta = readings
        .iter()
        .zip(fields.iter())
        .map(|(reading, field)| Meta {
            reading: reading.clone(),
            field: field.clone(),
            set_at: None,
        })
        .collect();

    let payload_size = fields
        .iter()
        .map(|spec| spec.length as usize)
        .sum::<usize>()
        .div_ceil(8);
    (meta, payload_size)
}

/// Scaling factor, when applied to milliseconds it returns some other
/// unit that is the most minimal representation of time for this device.
///
/// Example given a device with
///  - minimal sample interval 5 seconds
///  - the temporal_resolution 10 seconds
/// We only need to store something every 5 seconds and we can be up to 5
/// seconds off in both directions. Thus one 'minimal representation unit' is
/// 5 seconds
pub fn millis_to_minimal_representation(
    device_info: protocol::DeviceInfo,
) -> u64 {
    assert!(
        device_info.min_sample_interval > Duration::ZERO,
        "min sample interval may not be zero, device info: {device_info:?}"
    );
    let needed_interval = device_info
        .temporal_resolution
        .min(device_info.min_sample_interval)
        .as_secs_f32();
    let mul_factor = 0.001 / needed_interval;
    let div_factor = 1. / mul_factor;
    let factor = div_factor.round() as u64;
    assert_ne!(factor, 0);
    factor
}

#[instrument(level = "debug", skip(data))]
pub(crate) async fn store(
    data: &Data,
    reading: &protocol::Reading,
    data_dir: &Path,
) -> Result<()> {
    let mut data = data.0.lock().await;

    let key = reading.device();
    if let Some(series) = data.get_mut(&key) {
        series
            .append(reading)
            .wrap_err("failed to append to existing timeseries")?;
    } else {
        let mut series = Series::open_or_create(reading, data_dir)
            .wrap_err("Could not open new series")
            .with_note(|| format!("reading was: {reading:?}"))?;
        series
            .append(reading)
            .wrap_err("failed to newly created timeseries")?;
        let existing = data.insert(key, series);
        assert!(existing.is_none(), "should not race we still hold the lock");
    }
    trace!("stored new reading");

    Ok(())
}

fn to_speclist(readings: &[protocol::Reading]) -> Vec<bitspec::LengthWithOps> {
    readings
        .iter()
        .map(|r| bitspec::RangeWithRes {
            range: r.range(),
            resolution: r.resolution(),
        })
        .map(bitspec::LengthWithOps::from)
        .collect()
}

/// relative path without extension
fn base_path(reading: &protocol::Reading) -> PathBuf {
    let mut parts = Vec::new();
    let mut current = reading as &dyn Tree;
    loop {
        match current.inner() {
            Item::Leaf(reading::Info { device, .. }) => {
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

fn resample_setup(
    fields: &[bitspec::Field<f32>],
    payload_size: usize,
) -> (Resampler, Vec<downsample::Config>) {
    let resampler = Resampler::from_fields(fields.to_vec(), payload_size);
    let resample_configs = vec![
        downsample::Config {
            max_gap: None,
            bucket_size: 10,
        },
        downsample::Config {
            max_gap: None,
            bucket_size: 100,
        },
        downsample::Config {
            max_gap: None,
            bucket_size: 1000,
        },
    ];
    (resampler, resample_configs)
}

#[cfg(test)]
mod test {
    use super::*;
    use protocol::large_bedroom::{bed, desk};
    use protocol::{large_bedroom, Reading};

    #[test]
    #[should_panic(expected = "min sample interval may not be zero")]
    fn millis_to_minimal_representation_factor_is_not_zero() {
        let info = protocol::DeviceInfo {
            name: "test",
            affects_readings: &[],
            min_sample_interval: std::time::Duration::ZERO,
            max_sample_interval: std::time::Duration::MAX,
            temporal_resolution: std::time::Duration::from_secs(5),
            affectors: &[],
        };
        let factor = millis_to_minimal_representation(info);
        assert_eq!(5000 / factor, 5);
    }

    #[test]
    fn millis_to_minimal_representation_factor_makes_sense() {
        let info = protocol::DeviceInfo {
            name: "test",
            affects_readings: &[],
            min_sample_interval: std::time::Duration::from_secs(5),
            max_sample_interval: std::time::Duration::from_secs(5),
            temporal_resolution: std::time::Duration::from_secs(1),
            affectors: &[],
        };
        let factor = millis_to_minimal_representation(info);
        assert_eq!(5000 / factor, 5);

        let info = protocol::DeviceInfo {
            name: "test",
            affects_readings: &[],
            min_sample_interval: std::time::Duration::from_secs(5),
            max_sample_interval: std::time::Duration::from_secs(5),
            temporal_resolution: std::time::Duration::from_millis(1),
            affectors: &[],
        };
        let factor = millis_to_minimal_representation(info);
        assert_eq!(5005 / factor, 5005)
    }

    #[test]
    fn readings_from_same_device_have_same_path() {
        let reading_a = Reading::LargeBedroom(large_bedroom::Reading::Bed(
            bed::Reading::Temperature(0.0),
        ));
        let reading_b = Reading::LargeBedroom(large_bedroom::Reading::Bed(
            bed::Reading::Humidity(0.0),
        ));

        assert_eq!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_different_between_locations() {
        let reading_a = Reading::LargeBedroom(large_bedroom::Reading::Bed(
            bed::Reading::Humidity(0.0),
        ));
        let reading_b = Reading::LargeBedroom(large_bedroom::Reading::Desk(
            desk::Reading::Humidity(0.0),
        ));

        assert_ne!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_is_expected() {
        let reading = Reading::LargeBedroom(large_bedroom::Reading::Bed(
            bed::Reading::Humidity(0.0),
        ));
        assert_eq!(
            base_path(&reading),
            PathBuf::from("largebedroom/bed/sht31")
        );
    }
}
