use futures::{stream, Stream};
use std::net::SocketAddr;
use tokio::sync::mpsc;

use color_eyre::Result;

use crate::api::subscriber;

use super::{affector::Registar, Event};

#[derive(Debug, Clone)]
struct SubHandler {
    new_events: mpsc::Sender<Event>,
}

async fn do_setup(
    new_events: mpsc::Sender<Event>,
) -> impl Stream<Item = subscriber::Response> + Send + 'static {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    new_events
        .send(Event::NewSub { tx })
        .await
        .expect("Events processor (rx) should never stop");
    stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(subscriber::Response::SubUpdate)
            .map(|yielded| (yielded, rx))
    })
}

impl rpc::SubscriberHandler for SubHandler {
    type Update = subscriber::Response;

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
    request: subscriber::Request,
    client_name: String,
    new_event: mpsc::Sender<Event>,
    affectors: Registar,
) -> subscriber::Response {
    match perform_request_inner(request, client_name, new_event, affectors).await {
        Ok(response) => response,
        Err(error) => subscriber::Response::Error(error),
    }
}

async fn perform_request_inner(
    request: subscriber::Request,
    client_name: String,
    new_event: mpsc::Sender<Event>,
    affectors: Registar,
) -> Result<subscriber::Response, subscriber::ServerError> {
    Ok(match request {
        subscriber::Request::Handshake { .. } => {
            unreachable!("handshake only takes place during connection")
        }
        subscriber::Request::Actuate(affector) => match affectors.activate(affector) {
            Ok(()) => {
                new_event
                    .send(Event::AffectorControlled {
                        affector,
                        controlled_by: client_name.to_owned(),
                    })
                    .await
                    .map_err(|_| subscriber::ServerError::FailedToSpread)?;
                subscriber::Response::Actuate(Ok(()))
            }
            Err(err) => subscriber::Response::Actuate(Err(err)),
        },
        subscriber::Request::ListAffectors => subscriber::Response::ListAffectors(affectors.list()),
    })
}
