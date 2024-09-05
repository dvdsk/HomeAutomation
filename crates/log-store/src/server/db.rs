use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use data_server::api::subscriber::ReconnectingClient;
use data_server::api::subscriber::SubMessage;

use color_eyre::Result;

mod log;
pub(crate) use log::Logs;

mod stats;
pub(crate) use stats::Stats;
use tracing::debug;

pub(crate) async fn run(
    data_server_addr: SocketAddr,
    stats: Stats,
    logs: Logs,
    log_dir: &Path,
) -> Result<()> {
    let mut sub =
        ReconnectingClient::new(data_server_addr, "ha-log-store".to_string()).subscribe();

    let mut recently_logged = (Instant::now(), String::new());
    loop {
        let msg = sub.next().await;
        debug!("Got msg from data-server: {msg:?}");
        let res = match msg {
            SubMessage::Reading(reading) => {
                if let Err(e) = stats.increment(reading.device()).await {
                    Err(e)
                } else {
                    logs.clear_err(reading.device()).await
                }
            }
            SubMessage::ErrorReport(report) => logs.set_err(*report, log_dir).await,
            SubMessage::AffectorControlled { .. } => continue,
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
