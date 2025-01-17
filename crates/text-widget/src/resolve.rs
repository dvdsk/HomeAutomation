use color_eyre::eyre::{eyre, Context};
use data_server::api::subscriber::ReconnectingSubscribedClient;
use itertools::Itertools;
use nucleo_matcher::pattern::Pattern;
use protocol::Reading;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization};

mod fetch;

pub async fn available_readings(
    store_addr: Option<SocketAddr>,
    client: &mut ReconnectingSubscribedClient,
) -> Vec<Reading> {
    if let Some(store_addr) = store_addr {
        fetch::datalist_on_store(store_addr, crate::name())
            .await
            .unwrap_or(Vec::new())
    } else {
        let mut list = Vec::new();
        let _ignore_res = timeout(
            Duration::from_secs(7),
            fetch::datalist_from_updates(client, &mut list),
        )
        .await;
        list
    }
}

pub fn query(
    list: &[Reading],
    reading_query: &str,
) -> color_eyre::Result<Reading> {
    tracing::trace!("{:?}", list);
    let options = rank_options(&reading_query, &list);
    if options.is_empty() {
        return Err(eyre!("No idea whats meant by: {reading_query}"));
    };

    for reading in options.into_iter().take(3) {
        let reading_path = to_path(&reading);
        if promptly::prompt_default(
            format!(
                "Is {reading_path:?} what you meant with: {reading_query}?"
            ),
            false,
        )
        .wrap_err("Failed to read user confirmation")?
        {
            return Ok(reading);
        }
    }

    Err(eyre!("Could not identify reading meant by user"))
}

fn to_path(reading: &Reading) -> String {
    let path = format!("{:?}", reading);
    let (path, _) = path
        .rsplit_once('(')
        .expect("Reading is tree with at least deph 2");
    path.replace('(', " ")
}

fn rank_options(query: &str, options: &[Reading]) -> Vec<Reading> {
    let query = query.replace("_", " ").replace("-", " ");

    if options.is_empty() {
        return Vec::new();
    }

    let mut matcher =
        nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        &query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    options
        .iter()
        .map(|r| {
            (
                pattern.score(
                    nucleo_matcher::Utf32Str::new(
                        to_path(r).as_str(),
                        &mut buf,
                    ),
                    &mut matcher,
                ),
                r,
            )
        })
        .filter_map(|(score, reading)| score.zip(Some(reading)))
        .sorted_unstable_by_key(|(score, _)| *score)
        .rev()
        .map(|(_, reading)| reading.clone())
        .collect()
}
