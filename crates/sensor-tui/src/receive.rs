use std::net::SocketAddr;
use std::sync::mpsc;

use color_eyre::eyre::{Report, WrapErr};
use data_server::api::{Client, SubMessage};

use crate::{client_name, Update};

pub async fn receive_data(data_server: SocketAddr, tx: mpsc::Sender<Update>) {
    let client = match Client::connect(data_server, client_name()).await {
        Ok(client) => client,
        Err(err) => {
            let _ignore_panicked_ui = tx.send(Update::SubscribeError(
                Report::new(err).wrap_err("Could not connect to data server"),
            ));
            return;
        }
    };

    let mut subbed = match client.subscribe().await {
        Ok(client) => client,
        Err(err) => {
            let _ignore_panicked_ui = tx.send(Update::SubscribeError(
                Report::new(err).wrap_err("Could not subscribe to data server"),
            ));
            return;
        }
    };

    loop {
        let res = subbed
            .next()
            .await
            .wrap_err("Error getting next reading from server")
            .map(|msg| match msg {
                SubMessage::Reading(reading) => Update::SensorReading(reading),
                SubMessage::ErrorReport(error) => Update::SensorError(error),
                SubMessage::AffectorControlled {
                    affector,
                    controlled_by,
                } => Update::AffectorControlled {
                    affector,
                    controlled_by,
                },
            });

        match res {
            Ok(msg) => {
                tx.send(msg).unwrap();
            }
            Err(err) => {
                tx.send(Update::SubscribeError(err)).unwrap();
                break;
            }
        }
    }
}
