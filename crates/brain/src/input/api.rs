use axum::body::Bytes;
use axum::extract::State as aState;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use super::jobs::WakeUp;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not bind to port {port} on 127.0.0.1, error: {err}")]
    Binding { err: std::io::Error, port: u16 },
    #[error("Can not run server, io error: {0:?}")]
    Serving(std::io::Error),
}

/// sends back bincode serialized
async fn usually(aState(state): aState<State>) -> Vec<u8> {
    let usually = state.wakeup.usually().await;
    bincode::serialize(&usually).expect("Should be serializable")
}

/// sends back bincode serialized
async fn tomorrow(aState(state): aState<State>) -> Vec<u8> {
    let tomorrow = state.wakeup.tomorrow().await;
    bincode::serialize(&tomorrow).expect("Should be serializable")
}

async fn set_usually(aState(state): aState<State>, body: Bytes) -> StatusCode {
    let time: Option<(i8, i8)> = bincode::deserialize(&body[..])
        .map_err(|_| "Could not deserialize into time")
        .unwrap();
    state
        .wakeup
        .set_usually(time)
        .await
        .map_err(|_| "Could not save new alarm time")
        .unwrap();

    StatusCode::OK
}

async fn set_tomorrow(aState(state): aState<State>, body: Bytes) -> StatusCode {
    let time: Option<(i8, i8)> = bincode::deserialize(&body[..])
        .map_err(|_| "Could not deserialize into time")
        .unwrap();
    state
        .wakeup
        .set_tomorrow(time)
        .await
        .map_err(|_| "Could not save new alarm time")
        .unwrap();

    StatusCode::OK
}

#[derive(Clone)]
pub struct State {
    wakeup: WakeUp,
}

pub async fn setup(wakeup: WakeUp, port: u16) -> Result<(), Error> {
    let app = Router::new()
        .route("/alarm/usually", get(usually).post(set_usually))
        .route("/alarm/tomorrow", get(tomorrow).post(set_tomorrow))
        .with_state(State { wakeup });

    // https is done at the loadbalancer
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .map_err(|err| Error::Binding { err, port })?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.map_err(Error::Serving)
}
