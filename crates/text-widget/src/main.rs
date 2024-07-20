use data_server::subscriber::reconnecting;
use data_server::SubMessage;
use gethostname::gethostname;
use nucleo_matcher::pattern::Pattern;
use protocol::reading_tree::Tree;
use protocol::Reading;
use std::net::SocketAddr;

use data_store::api::Client;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the address of the store
    #[arg(long, default_value_t = SocketAddr::from(([127,0,0,1], 1236)))]
    store: SocketAddr,

    /// the address of the store
    #[arg(long, default_value_t = SocketAddr::from(([127,0,0,1], 1235)))]
    server: SocketAddr,

    /// print json format: {"msg": "reading value"}
    #[arg(short, long)]
    json: bool,

    /// String describing the reading. Something like temp can resolve to
    /// `large_bedroom desk temperature`
    reading: String,

    /// prints the resolved string
    #[arg(short, long)]
    debug: bool,
}

fn to_path(reading: &Reading) -> String {
    let path = format!("{:?}", reading);
    let (path, _) = path
        .rsplit_once('(')
        .expect("Reading is tree with at least deph 2");
    let path = path.replace('(', " ");
    path
}

fn resolve_argument(description: &str, options: &[Reading]) -> Reading {
    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        description,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    let best_scored = options
        .into_iter()
        .map(|r| {
            (
                pattern.score(
                    nucleo_matcher::Utf32Str::new(to_path(&r).as_str(), &mut buf),
                    &mut matcher,
                ),
                r,
            )
        })
        .max_by_key(|(score, _)| score.unwrap_or(0))
        .unwrap()
        .1;
    best_scored.clone()
}

async fn wait_for_update(client: &mut reconnecting::Subscriber, needed: &Reading) -> Reading {
    loop {
        let SubMessage::Reading(r) = client.next_msg().await else {
            continue;
        };
        if r.is_same_as(needed) {
            return r;
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut client = Client::connect(cli.store).await.unwrap();
    let readings = client.list_data().await.unwrap();

    let reading = resolve_argument(&cli.reading, &readings);
    if cli.debug {
        eprintln!("Will be showing: {reading:?}");
    }

    let host = gethostname();
    let host = host.to_string_lossy();
    let name = format!("text-widget@{host}");
    let mut client = reconnecting::Subscriber::new(cli.server, name);

    loop {
        let new = wait_for_update(&mut client, &reading).await;
        let info = new.leaf();
        let to_print = format!("{0:.1$} {2}", info.val, info.precision(), info.unit);
        let use_json = cli.json;
        if use_json {
            println!("{{\"msg\": \"{to_print}\"}}");
        } else {
            println!("{to_print}");
        }
    }
}
