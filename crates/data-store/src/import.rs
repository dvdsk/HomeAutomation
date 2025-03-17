use byteseries::ByteSeries;
use color_eyre::eyre::{bail, Context, OptionExt};
use color_eyre::{Result, Section};
use indicatif::{MultiProgress, ProgressBar};
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::data::series::Meta;

pub fn perform(
    data_dir: &Path,
    only: Option<PathBuf>,
    allow_corrupt: bool,
) -> Result<()> {
    let list = files_to_import(data_dir)?;
    if list.is_empty() {
        bail!("No files left to import")
    }

    let to_handle: Vec<_> = list
        .iter()
        .filter(|p| {
            only.as_ref().is_none_or(|allowed| {
                p.ends_with(allowed) || p.with_extension("").ends_with(allowed)
            })
        })
        .collect();

    if to_handle.is_empty() {
        return crate::no_filter_match_err(list, only);
    }

    let bars = MultiProgress::new();
    let files_bar =
        ProgressBar::new(to_handle.len() as u64).with_style(crate::bar_style());
    let files_bar = bars.insert(0, files_bar);
    files_bar.inc(0); // make the bar appear

    for path in &to_handle {
        handle_file(&path, bars.clone(), allow_corrupt)
            .wrap_err("Failed to import data")
            .with_note(|| format!("Input file: {}", path.display()))?;
        files_bar.inc(1);
    }

    drop(bars);
    tracing::info!(
        "Done, imported {} files to {}",
        to_handle.len(),
        data_dir.display()
    );

    Ok(())
}

fn files_to_import(
    data_dir: &Path,
) -> Result<Vec<PathBuf>, color_eyre::eyre::Error> {
    let mut buf = [0u8; 3];
    let mut res = Vec::new();

    crate::visit_dirs(data_dir, &mut |entry: &DirEntry| {
        if entry.path().extension() == Some(OsStr::new("csv")) {
            let mut file =
                fs::File::open(entry.path()).wrap_err("Could not open file")?;

            match file.read_exact(&mut buf) {
                Ok(()) => (),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                other => other.wrap_err("Could not read first 3 bytes")?,
            }

            let header = String::from_utf8_lossy(&buf);
            if header.contains("ts,") {
                res.push(entry.path());
            }
        }
        Ok(())
    })
    .wrap_err("Could not search for csv files")?;
    Ok(res)
}

fn resolve_readings(
    csv: &mut BufReader<fs::File>,
) -> color_eyre::Result<(Vec<protocol::Reading>, usize)> {
    let mut csv_header = String::new();
    let header_len = csv
        .read_line(&mut csv_header)
        .wrap_err("Could not read csv header")?;
    let readings = csv_header
        .trim()
        .strip_prefix("ts,")
        .ok_or_eyre("Csv header is missing the timestamp column header: 'ts,'")
        .with_note(|| format!("header is: {csv_header}"))?
        .split(',')
        .map(|s| {
            ron::from_str::<protocol::Reading>(s)
                .wrap_err("Could not decode Reading from string")
                .with_note(|| format!("string was: {s}"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((readings, header_len))
}

/// must be called after `resolve_readings`
/// this is a guess since some lines are longer some shorter.
fn number_of_lines(
    csv: &mut BufReader<fs::File>,
    header_len: usize,
) -> color_eyre::Result<u64> {
    let file = csv.get_mut();
    let data_length = file
        .metadata()
        .wrap_err("Could not get file metadata")?
        .len()
        - header_len as u64;
    let _ = file;

    if data_length == 0 {
        return Ok(0);
    }

    let mut line = String::new();
    let line_size = csv
        .read_line(&mut line)
        .wrap_err("Could not read data line from csv")?;
    csv.seek_relative(-(line_size as i64))
        .wrap_err("Could not seek back to start of csv data")?;

    Ok(data_length / line_size as u64)
}

fn handle_file(
    path: &Path,
    bars: MultiProgress,
    allow_corrupt: bool,
) -> Result<()> {
    let input_csv = fs::File::open(path).wrap_err("Could not open file")?;
    let mut input_csv = BufReader::new(input_csv);
    let (readings, header_len) = resolve_readings(&mut input_csv)?;
    let lines = number_of_lines(&mut input_csv, header_len)?;

    let specs = crate::data::series::to_speclist(&readings);
    let fields = crate::data::series::bitspec::speclist_to_fields(specs);
    let (meta_list, payload_size) =
        crate::data::series::meta_list_and_payload_size(&readings, &fields);
    for (Meta { reading, .. }, in_csv) in meta_list.iter().zip(&readings) {
        assert_eq!(reading, in_csv);
    }

    let header = crate::data::series::Header {
        readings: readings.to_vec(),
        encoding: fields.clone(),
    }
    .serialized()?;

    let (bs, _) = byteseries::ByteSeries::builder()
        .payload_size(payload_size)
        .with_header(header)
        .create_new(true)
        .open(path.with_extension("byteseries"))
        .wrap_err("Could not open output series")?;

    let copy_bar = ProgressBar::new(lines)
        .with_style(crate::bar_style())
        .with_message(format!(
            "{:?}",
            path.file_name().expect("we only handle files with names")
        ));
    let copy_bar = bars.insert(1, copy_bar);

    let mut skipped_sections = 0;
    let mut last_correct_before_corrupt = 0;
    if allow_corrupt {
        copy_over_content(
            input_csv,
            bs,
            meta_list,
            copy_bar.clone(),
            |_, prev| {
                if prev != last_correct_before_corrupt {
                    skipped_sections += 1;
                } else {
                    last_correct_before_corrupt = prev;
                }
                true
            },
        )
        .wrap_err("Failed to copy over content")?;
    } else {
        copy_over_content(
            input_csv,
            bs,
            meta_list,
            copy_bar.clone(),
            |_, _| false,
        )
        .wrap_err("Failed to copy over content")?;
    }

    if skipped_sections > 0 {
        bars.suspend(|| {
            warn!(
                "skipped {skipped_sections} sections of data due to \
                corrupt full meta ts"
            );
        })
    }

    bars.remove(&copy_bar);

    Ok(())
}

fn copy_over_content(
    input_csv: BufReader<fs::File>,
    mut bs: ByteSeries,
    meta_list: Vec<Meta>,
    copy_bar: ProgressBar,
    mut on_invalid_ts: impl FnMut(u64, u64) -> bool,
) -> color_eyre::Result<()> {
    use byteseries::series::Error as BsError;

    let mut encoded_line = vec![0; bs.payload_size()];
    for (i, line) in input_csv.lines().enumerate() {
        let line = line.wrap_err("Could not read line from csv")?;
        let (ts, line) =
            line.split_once(',').ok_or_eyre("Empty line in csv")?;
        let ts: u64 = ts
            .parse()
            .wrap_err("Could not parse timestamp as integer")
            .with_note(|| format!("timestamp: {ts}"))?;

        for (value, Meta { field, .. }) in line.split(',').zip(&meta_list) {
            let value: f32 =
                value.parse().wrap_err("Could not parse field as f32")?;
            field.encode(value, &mut encoded_line);
        }

        match bs.push_line(ts, &encoded_line) {
            Ok(()) => (),
            Err(BsError::TimeNotAfterLast { new, prev })
                if on_invalid_ts(new, prev) =>
            {
                ()
            }
            Err(other) => Err(other)
                .wrap_err("Could not push line to output")
                .with_note(|| format!("line {} in file", i + 1))?,
        };

        copy_bar.inc(1);
    }

    Ok(())
}
