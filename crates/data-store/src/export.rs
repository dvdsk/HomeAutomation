use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use byteseries::ByteSeries;
use color_eyre::eyre::{bail, Context, OptionExt, Result};
use color_eyre::{Report, Section};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;

use crate::data::series;

mod csv;
use csv::Csv;
mod decoder;
use decoder::ExportDecoder;

pub fn perform(data_dir: &Path, only: Option<PathBuf>) -> Result<()> {
    let list = files_to_export(data_dir)?;
    if list.is_empty() {
        bail!("No files left to export")
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
        return no_filter_match_err(list, only);
    }

    let bars = MultiProgress::new();
    let files_bar =
        ProgressBar::new(to_handle.len() as u64).with_style(bar_style());
    let files_bar = bars.insert(0, files_bar);
    files_bar.inc(0); // make the bar appear

    for path in to_handle {
        handle_file(&path, bars.clone())
            .wrap_err("Failed to export data")
            .with_note(|| format!("Input file: {}", path.display()))?;
        files_bar.inc(1);
    }

    Ok(())
}

fn no_filter_match_err(
    list: Vec<PathBuf>,
    only: Option<PathBuf>,
) -> Result<(), Report> {
    return Err(Report::msg(
        "None of the paths ended with required argument",
    ))
    .with_note(|| {
        format!("filter argument (--only) {}", only.unwrap().display())
    })
    .with_note(|| {
        format!(
            "examples of files: \n\t- {}",
            list.iter()
                .map(|p| p.display().to_string())
                .take(5)
                .join("\n\t- ")
        )
    });
}

pub fn handle_file(path: &Path, bars: MultiProgress) -> Result<()> {
    let metadata =
        read_metadata(path).wrap_err("Could not extract metadata")?;
    let series::Header { readings, encoding } =
        ron::from_str(&metadata).wrap_err("Could not deserialize metadata")?;
    let (meta, payload_size) =
        series::meta_list_and_payload_size(&readings, &encoding);

    let (input_series, _) = ByteSeries::builder()
        .payload_size(payload_size)
        .with_header(metadata.as_bytes().to_vec())
        .open(path)
        .wrap_err("Could not open byteseries")?;

    let decoder = ExportDecoder::from_fields(
        meta.iter().map(|m| m.field.clone()).collect(),
    );
    let output = Csv::open(&readings, path.with_extension("csv"))
        .wrap_err("Failed to open output csv")
        .suggestion("If the file already exists remove it")?;

    let copy_bar = ProgressBar::new(input_series.len())
        .with_style(bar_style())
        .with_message(format!(
            "{:?}",
            path.file_name().expect("we only handle files with names")
        ));
    let copy_bar = bars.insert(1, copy_bar);
    copy_over_content(input_series, decoder, output, copy_bar.clone())
        .wrap_err("Failed to copy over content")?;
    bars.remove(&copy_bar);
    Ok(())
}

fn copy_over_content(
    mut input_series: ByteSeries,
    mut decoder: ExportDecoder,
    mut output: Csv,
    copy_bar: ProgressBar,
) -> Result<()> {
    let mut read_start = 0;
    loop {
        let mut timestamps = Vec::new();
        let mut data = Vec::new();

        if let Err(byteseries::series::Error::InvalidRange(
            byteseries::seek::Error::StartAfterData { .. },
        )) = input_series.read_first_n(
            100_000,
            &mut decoder,
            read_start..,
            &mut timestamps,
            &mut data,
        ) {
            copy_bar.finish();
            break Ok(());
        }

        let Some(last_ts) = timestamps.last() else {
            break Ok(()); // all data consumed
        };

        read_start = *last_ts + 1;
        for (ts, line) in timestamps.into_iter().zip(data.into_iter()) {
            copy_bar.inc(1);
            output
                .write_line(ts, &line)
                .wrap_err("failed to write line to csv")?;
        }
    }
}

fn read_metadata(path: &Path) -> Result<String> {
    const HEADER_END: &'static str = "In the case the creator of this \
    file wanted to store metadata in it that\n    follows now:";

    let file = fs::File::open(path)
        .wrap_err("Could not open file for reading metadata")?;
    let mut buf = Vec::with_capacity(12_000);
    file.take(12_000)
        .read_to_end(&mut buf)
        .wrap_err("Could not read first 12_000 bytes or end")?;
    let header = String::from_utf8_lossy(&buf);
    let Some((_, metadata_and_rest)) = header.split_once(HEADER_END) else {
        bail!("Could not find end of byteseries header")
    };

    let metadata = take_around_parenthesis(metadata_and_rest)
        .wrap_err("Could not extract metadata around parenthesis")?;

    Ok(metadata.to_string())
}

/// reads till first parenthesis it sees then looks for
/// the corresponding closing one
fn take_around_parenthesis(input: &str) -> Result<&str> {
    let start = input.find('(').ok_or_eyre("Missing starting parenthesis")?;
    let input = &input[start..];

    let mut end = None;
    let mut depth = 0;
    for (idx, char) in input.char_indices() {
        match char {
            '(' => {
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(idx);
                    break;
                }
            }
            _ => (),
        }
    }

    let end = end
        .ok_or_eyre(
            "Could not find matching closing parenthesis before end of input",
        )
        .with_note(|| format!("meta was:```\n{}\n```", input))?;

    Ok(&input[..=end])
}

fn files_to_export(
    data_dir: &Path,
) -> Result<Vec<PathBuf>, color_eyre::eyre::Error> {
    const HEADER_START: &'static str = "This is a byteseries 1 file, an embedded \
        timeseries file. Time may here may\n    be whatever value as long as it is \
        monotonically increasing. The entries\n    have a fixed length that \
        never changes.";

    let mut buf = [0u8; 300];
    let mut res = Vec::new();
    visit_dirs(data_dir, &mut |entry: &DirEntry| {
        if entry.path().extension() == Some(OsStr::new("byteseries")) {
            let mut file =
                fs::File::open(entry.path()).wrap_err("Could not open file")?;

            match file.read_exact(&mut buf) {
                Ok(()) => (),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                other => other.wrap_err("Could not read first 300 bytes")?,
            }

            let header = String::from_utf8_lossy(&buf);
            if header.contains(HEADER_START) {
                res.push(entry.path());
            }
        }
        Ok(())
    })
    .wrap_err("Could not search for byteseries files")?;
    Ok(res)
}

fn visit_dirs(
    dir: &Path,
    mut cb: &mut dyn FnMut(&DirEntry) -> Result<()>,
) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).wrap_err("Could not read dir")? {
            let entry = entry.wrap_err("Error walking dir")?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, &mut cb)?;
            } else {
                cb(&entry)
                    .wrap_err("Could not check file header")
                    .with_note(|| {
                        format!("file: {}", entry.path().display())
                    })?;
            }
        }
    }
    Ok(())
}

fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} [{eta}]",
    )
    .unwrap()
    .progress_chars("##-")
}
