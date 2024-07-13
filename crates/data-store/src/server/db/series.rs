use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use byteseries::{downsample, series, ByteSeries};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::{Result, Section};
use protocol::reading_tree::{Item, ReadingInfo, Tree};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{info, instrument};

use byteseries::file::OpenError as FileOpenError;
use series::data::OpenError as DataOpenError;
use series::Error::Open;

mod bitspec;
mod resampler;

use self::resampler::Resampler;

use super::Data;

#[derive(Debug)]
struct Meta {
    reading: protocol::Reading,
    field: bitspec::MetaField<f32>,
    set_at: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct Series {
    line: Vec<u8>,
    meta: Vec<Meta>,
    byteseries: ByteSeries,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Header {
    readings: Vec<protocol::Reading>,
    encoding: Vec<bitspec::MetaField<f32>>,
}

impl Series {
    fn open_or_create(reading: &protocol::Reading, dir: &Path) -> Result<Self> {
        let readings = reading.device().affected_readings();
        let specs = to_speclist(readings);
        let fields = bitspec::speclist_to_fields(specs);
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
        let resampler = Resampler::from_fields(fields.clone(), payload_size);
        let resample_configs = vec![downsample::Config {
            max_gap: None,
            bucket_size: 10,
        }];

        let path = base_path(reading);
        let path = dir.join(path);
        let res = ByteSeries::open_existing_with_resampler::<Header, _>(
            &path,
            payload_size,
            resampler.clone(),
            resample_configs.clone(),
        );

        let expected_header = Header {
            readings: readings.to_vec(),
            encoding: fields.clone(),
        };

        let byteseries = try_create_new_if_open_failed(
            res,
            expected_header,
            path,
            payload_size,
            resampler,
            resample_configs,
        )?;

        Ok(Self {
            line: vec![0; payload_size],
            meta,
            byteseries,
        })
    }

    fn append(&mut self, reading: &protocol::Reading) -> Result<()> {
        let index = reading
            .device()
            .affected_readings()
            .iter()
            .map(|r| r.leaf().branch_id)
            .position(|local_id| local_id == reading.leaf().branch_id)
            .expect("series are grouped by devices, elements come from affected_readings");

        let meta = &mut self.meta[index];
        meta.field.encode::<f32>(reading.leaf().val, &mut self.line);
        meta.set_at = Some(Instant::now());

        if self
            .meta
            .iter()
            .map(|Meta { set_at, .. }| set_at)
            .all(|set| set.map(|s| s.elapsed().as_millis() < 500).unwrap_or(false))
        {
            let time = OffsetDateTime::now_utc()
                - self.meta[index]
                    .set_at
                    .take()
                    .expect("if any is None the if is false")
                    .elapsed();

            self.byteseries
                .push_line(time.unix_timestamp() as u64, &self.line)
                .wrap_err("Could not write to timeseries on disk")?;
            tracing::debug!("encoded data stored in byteseries");
        }

        Ok(())
    }

    /// # Panics
    /// If any of the requested readings are not part of this series.
    #[instrument(skip(self))]
    pub fn read(
        &mut self,
        readings: &[protocol::Reading],
        start: OffsetDateTime,
        end: OffsetDateTime,
        n: usize,
    ) -> Result<(Vec<OffsetDateTime>, Vec<Vec<f32>>), byteseries::series::Error> {
        let range = start.unix_timestamp() as u64..=end.unix_timestamp() as u64;
        let fields = readings
            .iter()
            .map(|requested| {
                self.meta
                    .iter()
                    .find(|meta| *requested == meta.reading)
                    .inspect(|meta| info!("meta used for decoding: {meta:?}"))
                    .map(|meta| meta.field.clone())
                    .expect(
                        "caller of read makes sure all readings are\
                        part of this series",
                    )
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
        )?;

        let time = timestamps
            .into_iter()
            .map(|ts| {
                OffsetDateTime::from_unix_timestamp(ts as i64)
                    .expect("only valid timestamps are stored in byteseries")
            })
            .collect();

        let len = interleaved_data
            .first()
            .expect("read returns an error if there is not data")
            .len();
        let mut data = vec![Vec::new(); len];
        for interleaved in interleaved_data {
            for (interleaved, data) in interleaved.into_iter().zip(data.iter_mut()) {
                data.push(interleaved)
            }
        }
        Ok((time, data))
    }
}

fn try_create_new_if_open_failed(
    res: Result<(ByteSeries, Header), series::Error>,
    expected_header: Header,
    path: PathBuf,
    payload_size: usize,
    resampler: Resampler,
    resample_configs: Vec<downsample::Config>,
) -> Result<ByteSeries, color_eyre::eyre::Error> {
    match res {
        Ok((byteseries, opened_file_header)) => {
            if opened_file_header == expected_header {
                Ok(byteseries)
            } else {
                return Err(eyre!("header in file does not match readings"))
                    .with_note(|| {
                        format!(
                            "header in the just existing (opened) byteseries: {:?}",
                            opened_file_header
                        )
                    })
                    .with_note(|| {
                        format!(
                            "header for the data we want to write: {:?}",
                            expected_header
                        )
                    });
            }
        }
        Err(Open(DataOpenError::File(FileOpenError::Io(e))))
            if e.kind() == io::ErrorKind::NotFound =>
        {
            if let Some(dirs) = path.parent() {
                create_dir_all(dirs)
                    .wrap_err("Could not create dirs structure for reading")
                    .with_note(|| format!("dirs: {}", dirs.display()))?;
                std::fs::read_dir(dirs)
                    .unwrap()
                    .map(Result::unwrap)
                    .for_each(|p| println!("{p:?}"));
            }
            // compile_error!("create directory structure");
            info!("creating new byteseries");
            ByteSeries::new_with_resamplers(
                &path,
                payload_size,
                expected_header,
                resampler,
                resample_configs,
            )
            .wrap_err("Could not create new byteseries")
            .with_note(|| format!("path: {}", path.display()))
        }
        Err(e) => return Err(e).wrap_err("Could not open existing byteseries")?,
    }
}

#[instrument(level = "debug", skip(data))]
pub(crate) async fn store(data: &Data, reading: &protocol::Reading, data_dir: &Path) -> Result<()> {
    tracing::debug!("storing received reading: {reading:?}");
    let mut data = data.0.lock().await;

    let key = reading.device();
    if let Some(series) = data.get_mut(&key) {
        series
            .append(reading)
            .wrap_err("failed to append to existing timeseries")?;
    } else {
        let mut series = Series::open_or_create(&reading, data_dir)
            .wrap_err("Could not open new series")
            .with_note(|| format!("reading was: {reading:?}"))?;
        series
            .append(reading)
            .wrap_err("failed to newly created timeseries")?;
        let existing = data.insert(key, series);
        assert!(existing.is_none(), "should not race we still hold the lock");
    }

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
            Item::Leaf(ReadingInfo { device, .. }) => {
                parts.push(device.as_str().to_lowercase());
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

#[cfg(test)]
mod test {
    use super::*;
    use protocol::large_bedroom::{bed, desk};
    use protocol::{large_bedroom, Reading};

    #[test]
    fn readings_from_same_device_have_same_path() {
        let reading_a =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Temperature(0.0)));
        let reading_b =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));

        assert_eq!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_different_between_locations() {
        let reading_a =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));
        let reading_b =
            Reading::LargeBedroom(large_bedroom::Reading::Desk(desk::Reading::Humidity(0.0)));

        assert_ne!(base_path(&reading_a), base_path(&reading_b));
    }

    #[test]
    fn reading_path_is_expected() {
        let reading =
            Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(0.0)));
        assert_eq!(base_path(&reading), PathBuf::from("largebedroom/bed/sht31"));
    }
}
