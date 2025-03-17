#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use color_eyre::eyre::Context;
use color_eyre::Section;
use indicatif::ProgressStyle;
use itertools::Itertools;

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "server")]
pub mod data;
#[cfg(feature = "server")]
pub mod export;
#[cfg(feature = "server")]
pub mod import;
#[cfg(feature = "server")]
pub mod server;

pub(crate) fn visit_dirs(
    dir: &Path,
    mut cb: &mut dyn FnMut(&DirEntry) -> color_eyre::Result<()>,
) -> color_eyre::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).wrap_err("Could not read dir")? {
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

pub(crate) fn no_filter_match_err(
    list: Vec<PathBuf>,
    only: Option<PathBuf>,
) -> color_eyre::Result<()> {
    return Err(color_eyre::Report::msg(
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

fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} [{eta}]",
    )
    .unwrap()
    .progress_chars("##-")
}
