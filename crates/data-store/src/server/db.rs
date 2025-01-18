use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use data_server::api::subscriber::ReconnectingClient;
use data_server::api::subscriber::SubMessage;

use color_eyre::{Result, Section};

use crate::data::Data;

pub(crate) async fn run(
    data_server_addr: SocketAddr,
    data: Data,
    data_dir: &Path,
) -> Result<()> {
    let mut sub =
        ReconnectingClient::new(data_server_addr, "ha-data-store".to_string())
            .subscribe();

    let mut recently_logged = (Instant::now(), String::new());
    loop {
        let msg = sub.next().await;
        let SubMessage::Reading(reading) = msg else {
            continue;
        };

        let res = crate::data::series::store(&data, &reading, data_dir)
            .await
            .with_note(|| format!("reading: {reading:?}"));

        const FIVE_MIN: Duration = Duration::from_secs(60 * 5);
        if let Err(report) = res {
            let e = format!("{report:?}");
            if recently_logged.1 == e && recently_logged.0.elapsed() <= FIVE_MIN
            {
                continue;
            } else {
                tracing::error!("Error processing new reading {reading:?},\nerror: {e}");
                recently_logged = (Instant::now(), e);
            }
        }
    }
}
