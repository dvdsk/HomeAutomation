use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use data_server::subscriber::reconnecting;

use color_eyre::Result;
use data_server::SubMessage;

mod log;
pub(crate) use log::Logs;

mod stats;
pub(crate) use stats::Stats;

pub(crate) async fn run(
    data_server_addr: SocketAddr,
    stats: Stats,
    logs: Logs,
    log_dir: &Path,
) -> Result<()> {
    let mut sub = reconnecting::Subscriber::new(data_server_addr, "ha-data-store".to_string());

    let mut recently_logged = (Instant::now(), String::new());
    loop {
        let msg = sub.next_msg().await;
        let res = match msg {
            SubMessage::Reading(reading) => stats.increment(reading.device()).await,
            SubMessage::ErrorReport(report) => logs.store(*report).await,
        };

        const FIVE_MIN: Duration = Duration::from_secs(60 * 5);
        if let Err(report) = res {
            let e = format!("got error with report: {report:?}");
            tracing::warn!("test: {e}");
            if recently_logged.1 == e && recently_logged.0.elapsed() <= FIVE_MIN {
                continue;
            } else {
                tracing::error!("Error processing new reading: {e}");
                recently_logged = (Instant::now(), e);
            }
        }
    }
}
