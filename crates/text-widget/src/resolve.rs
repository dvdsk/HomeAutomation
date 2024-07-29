use color_eyre::eyre::eyre;
use data_server::subscriber::reconnecting::Subscriber;
use nucleo_matcher::pattern::Pattern;
use protocol::Reading;
use std::time::Duration;
use tokio::time::timeout;

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization};

mod fetch;

pub async fn query(
    cli: &crate::Cli,
    client: &mut Subscriber,
) -> color_eyre::Result<Reading> {
    let mut list = if let Some(store_addr) = cli.store {
        fetch::datalist_on_store(store_addr, crate::name())
            .await
            .unwrap_or(Vec::new())
    } else {
        Vec::new()
    };
    if list.is_empty() {
        let _ignore_res = timeout(
            Duration::from_secs(7),
            fetch::datalist_from_updates(client, &mut list),
        )
        .await;
    };

    tracing::trace!("{:?}", list);
    let Some(reading) = resolve_argument(&cli.reading, &list) else {
        return Err(eyre!("Could not resolve argument"));
    };

    Ok(reading)
}

fn to_path(reading: &Reading) -> String {
    let path = format!("{:?}", reading);
    let (path, _) = path
        .rsplit_once('(')
        .expect("Reading is tree with at least deph 2");
    path.replace('(', " ")
}

fn resolve_argument(description: &str, options: &[Reading]) -> Option<Reading> {
    if options.is_empty() {
        return None;
    }

    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        description,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    let best_scored = options
        .iter()
        .map(|r| {
            (
                pattern.score(
                    nucleo_matcher::Utf32Str::new(to_path(r).as_str(), &mut buf),
                    &mut matcher,
                ),
                r,
            )
        })
        .max_by_key(|(score, _)| score.unwrap_or(0))
        .unwrap()
        .1;
    Some(best_scored.clone())
}
