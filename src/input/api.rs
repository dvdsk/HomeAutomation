use axum::Router;
use axum::body::Bytes;
use axum::routing::{get, post};

use crate::errors::Error;

use super::jobs::WakeUp;

async fn usually(wakeup: WakeUp) -> Bytes {
}

async fn setup(wakeup: WakeUp) -> Result<(), Error> {
    let app = Router::new()
        .route("/alarm/usually", get(usually))
        // .route("/alarm/tomorrow", get(tomorrow))
        // .route("/alarm/usually", post(usually))
        // .route("/alarm/tomorrow", post(tomorrow))
        .with_state(wakeup);

    // https is done at the loadbalancer
    let listener = tokio::net::TcpListener::bind("127.0.0.1:80")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
