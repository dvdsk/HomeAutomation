use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::PathBuf;

use color_eyre::eyre::Context;
use color_eyre::{Result, Section};
use protocol::Reading;
use tokio::fs;

pub(crate) async fn store_to_file(
    fully_qualified: &[Reading],
    queries: &[String],
) -> Result<()> {
    let mut entries = load_entries()
        .await
        .wrap_err("Could not load existing cache")?
        .unwrap_or_default();

    entries
        .extend(queries.iter().cloned().zip(fully_qualified.iter().cloned()));
    let serialized = ron::to_string(&entries)
        .wrap_err("Could not serialize query resolve result")?;

    let path = path()?;
    fs::write(path, serialized.as_bytes())
        .await
        .wrap_err("Could not update file")
}

fn path() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .ok_or(color_eyre::eyre::eyre!(
            "Could not load where to load query from"
        ))?
        .join(concat!(env!("CARGO_PKG_NAME"), ".ron")))
}

type Query = String;
pub(crate) async fn load_entries() -> Result<Option<HashMap<Query, Reading>>> {
    let path = path()?;
    let entries = match fs::read_to_string(&path).await {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        other_err => other_err
            .wrap_err("Could not read cache to string")
            .with_note(|| format!("path: {}", path.display()))?,
    };
    Ok(Some(
        ron::from_str(&entries)
            .wrap_err("Could not deserialize resolve cache")
            .with_note(|| format!("path: {}", path.display()))?,
    ))
}

pub(crate) async fn load_from_file(
    queries: &Vec<Query>,
) -> Result<Vec<Option<Reading>>> {
    let Some(mut entries) = load_entries().await? else {
        return Ok(queries.iter().map(|_| None).collect());
    };

    Ok(queries.iter().map(|q| entries.remove(q)).collect())
}

pub(crate) async fn clear() -> Result<()> {
    let path = path()?;
    match fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).wrap_err("io error"),
    }
}
