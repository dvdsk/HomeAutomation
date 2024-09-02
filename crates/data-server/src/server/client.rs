use futures::{stream, Stream};
use std::net::SocketAddr;
use tokio::sync::mpsc;

use color_eyre::Result;

use crate::api::AffectorError;
use crate::api::{self, ServerError};

use super::{affector::Offline, affector::Registar, Event};

#[derive(Debug, Clone)]
struct SubHandler {
    new_events: mpsc::Sender<Event>,
}

async fn do_setup(
    new_events: mpsc::Sender<Event>,
) -> impl Stream<Item = api::Response> + Send + 'static {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    new_events
        .send(Event::NewSub { tx })
        .await
        .expect("Events processor (rx) should never stop");
    stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(api::Response::SubUpdate)
            .map(|yielded| (yielded, rx))
    })
}

impl rpc::SubscriberHandler for SubHandler {
    type Update = crate::api::Response;

    fn setup(
        &mut self,
    ) -> impl std::future::Future<
        Output = impl futures::prelude::Stream<Item = Self::Update> + Send + 'static,
    > + Send
           + 'static {
        do_setup(self.new_events.clone())
    }
}

pub async fn handle(
    addr: SocketAddr,
    tx: mpsc::Sender<Event>,
    affectors: Registar,
) -> color_eyre::Result<()> {
    let port = addr.port();
    let handler = SubHandler {
        new_events: tx.clone(),
    };
    rpc::server::run(
        port,
        move |req, name| {
            let tx = tx.clone();
            let affectors = affectors.clone();
            perform_request(req, name.to_owned(), tx, affectors)
        },
        Some(handler),
    )
    .await
}

async fn perform_request(
    request: api::Request,
    client_name: String,
    new_event: mpsc::Sender<Event>,
    affectors: Registar,
) -> api::Response {
    match perform_request_inner(request, client_name, new_event, affectors).await {
        Ok(response) => response,
        Err(error) => api::Response::Error(error),
    }
}

async fn perform_request_inner(
    request: api::Request,
    client_name: String,
    new_event: mpsc::Sender<Event>,
    affectors: Registar,
) -> Result<api::Response, ServerError> {
    Ok(match request {
        api::Request::Handshake { .. } => {
            unreachable!("handshake only takes place during connection")
        }
        api::Request::Actuate(affector) => match affectors.activate(affector) {
            Ok(()) => {
                new_event
                    .send(Event::AffectorControlled {
                        affector,
                        controlled_by: client_name.to_owned(),
                    })
                    .await
                    .map_err(|_| ServerError::FailedToSpread)?;
                api::Response::Actuate(Ok(()))
            }
            Err(Offline) => api::Response::Actuate(Err(AffectorError::Offline)),
        },
        api::Request::ListAffectors => api::Response::ListAffectors(affectors.list()),
    })
}
