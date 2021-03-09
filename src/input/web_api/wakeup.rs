use actix_web::web::Data;
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web::http::Method;
use actix_web::web::Bytes;
use actix_web::HttpResponse;

use super::*;
use crate::input::jobs::WakeUp;

async fn set_tomorrow(wakeup: &WakeUp, body: Bytes) -> Result<HttpResponse, ()> {
    let time = bincode::deserialize(&body).map_err(|_| ())?;
    wakeup.set_tomorrow(time).await.map_err(|_| ())?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn tomorrow(state: Data<State>, auth: BasicAuth, req: HttpRequest, body: Bytes) -> HttpResponse {
    if !authenticated(auth) {
        return make_auth_error();
    }

    match *req.method() {
        Method::GET => {
            let tomorrow = state.wakeup.tomorrow();
            let bytes = bincode::serialize(&tomorrow).unwrap();
            HttpResponse::Ok().body(bytes)
        }
        Method::POST => {
            match set_tomorrow(&state.wakeup, body).await {
                Ok(resp) => resp,
                Err(_) => HttpResponse::BadRequest().finish()
            }
        }
        _ => HttpResponse::BadRequest().finish(),
    }
}

async fn set_usually(wakeup: &WakeUp, body: Bytes) -> Result<HttpResponse, ()> {
    let time = bincode::deserialize(&body).map_err(|_| ())?;
    wakeup.set_usually(time).await.map_err(|_| ())?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn usually(state: Data<State>, auth: BasicAuth, req: HttpRequest, body: Bytes) -> HttpResponse {
    if !authenticated(auth) {
        return make_auth_error();
    }

    match *req.method() {
        Method::GET => {
            let usually = dbg!(state.wakeup.usually());
            let bytes = bincode::serialize(&usually).unwrap();
            HttpResponse::Ok().body(bytes)
        }
        Method::POST => {
            match set_usually(&state.wakeup, body).await {
                Ok(resp) => resp,
                Err(_) => HttpResponse::BadRequest().finish()
            }
        }
        _ => HttpResponse::BadRequest().finish(),
    }
}
