use std::net::SocketAddr;

use color_eyre::eyre::Context;
use futures::SinkExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_serde::formats::Bincode;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use color_eyre::{Result, Section};

use tracing::{info, warn};

use crate::api;
use crate::api::Response;

use super::{Conn, Event};

async fn handle_client_inner(
    conn: &mut Conn,
    new_subscribers: &mut mpsc::Sender<Event>,
) -> color_eyre::Result<()> {
    let request = conn
        .try_next()
        .await
        .wrap_err("Could not get next client request")?;
    let Some(request) = request else {
        return Ok(());
    };

    let response = match &request {
        api::Request::Handshake { name } => Response::Handshake,
        api::Request::Actuate(affector) => todo!(),
        api::Request::Subscribe => api::Response::Subscribe,
    };

    conn.send(response)
        .await
        .wrap_err("failed to send response to client")?;

    if let api::Request::Subscribe = request {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        new_subscribers
            .send(Event::NewSub { tx })
            .await
            .wrap_err("Could not register the client as subscriber")?;
        return Err(crate::server::subscribe::handle_subscriber(conn, rx)
            .await
            .wrap_err("Somewhing went wrong sending updates to the client"));
    }

    Ok(())
}

async fn handle_client(mut conn: Conn, id: String, mut new_subscribers: mpsc::Sender<Event>) {
    loop {
        if let Err(e) = handle_client_inner(&mut conn, &mut new_subscribers).await {
            warn!("Error while handling client {id}: {e:?}")
        }
    }
}

pub async fn handle(addr: SocketAddr, tx: mpsc::Sender<Event>) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start receiving updates")
        .with_note(|| format!("trying to listen on: {addr}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                if let Some((conn, name)) = handshake(stream, source).await {
                    let id = format!("{name}@{source}");
                    tokio::task::spawn(handle_client(conn, id, tx.clone()));
                }
            }
            Err(e) => {
                warn!("new connection failed: {e}");
                continue;
            }
        };
    }
}

#[tracing::instrument(skip(stream))]
async fn handshake(stream: TcpStream, source: SocketAddr) -> Option<(Conn, String)> {
    let length_delimited = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(1024)
            .new_codec(),
    );
    let mut stream: tokio_serde::Framed<_, api::Request, api::Response, _> =
        tokio_serde::Framed::new(length_delimited, Bincode::default());

    match stream.try_next().await {
        Ok(Some(api::Request::Handshake { name })) => {
            info!("Client {name} connected from {source}");
            if let Err(e) = stream.send(Response::Handshake).await {
                warn!("Failed to acknowledge handshake, error: {e}");
            };
            return Some((stream, name));
        }
        Ok(Some(other)) => warn!("client tried to connected without handshake, it send: {other:?}"),
        Ok(None) => warn!("client closed connection immediately"),
        Err(e) => warn!("connection or decoding issue while receiving handshake: {e:?}"),
    }

    None
}
