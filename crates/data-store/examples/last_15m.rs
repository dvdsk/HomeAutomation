use jiff::ToSpan;
use nucleo_matcher::pattern::Pattern;
use protocol::Reading;
use std::net::SocketAddr;

use data_store::api::Client;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the address of the store
    #[arg(short, long, default_value_t = SocketAddr::from(([127,0,0,1], 1236)))]
    addr: SocketAddr,
    reading: String,
}

fn to_path(reading: &Reading) -> String {
    let path = format!("{:?}", reading);
    let (path, _) = path.rsplit_once('(').unwrap();
    let path = path.replace('(', " ");
    path
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut client = Client::connect(cli.addr, "data_store_example".to_owned())
        .await
        .unwrap();
    let readings = client.list_data().await.unwrap();

    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        &cli.reading,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    let best_scored = readings
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

    println!("Showing results for: {best_scored:?}");
    let now = jiff::Timestamp::now();
    let data = client
        .get_data(now - 5.minutes(), now, best_scored, 10)
        .await
        .unwrap();

    dbg!(data);
}
