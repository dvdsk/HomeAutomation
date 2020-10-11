use actix_identity::Identity;
use actix_web::web::{Data, Form};
use actix_web::Result as wResult;
use actix_web::{http, HttpRequest, HttpResponse};
use rand::{FromEntropy, Rng};
use serde::Deserialize;

use std::sync::{atomic::Ordering, Arc, Mutex};

use super::{Session, State};

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Found().finish()
}

pub fn login_page() -> HttpResponse {
    let page = include_str!("static_webpages/login.html");
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(page)
}

#[derive(Deserialize)]
pub struct Logindata {
    u: String,
    p: String,
}

/// State and POST Params
pub async fn login_get_and_check(
    id: Identity,
    state: Data<State>,
    req: HttpRequest,
    params: Form<Logindata>,
) -> wResult<HttpResponse> {
    trace!("checking login");

    //time function duration
    let now = std::time::Instant::now();

    //if login valid (check passwdb) load userinfo
    if state
        .passw_db
        .verify_password(params.u.as_str().as_bytes(), params.p.as_str().as_bytes())
        .is_err()
    {
        warn!("incorrect password");
        return Ok(HttpResponse::build(http::StatusCode::UNAUTHORIZED)
            .content_type("text/plain")
            .body("incorrect password or username"));
    } else {
        info!("user logged in");
    }

    let session = Session {}; //kept in case we want it back in the future
                              //find free session_numb, set new session number and store new session
    let session_id = state.free_session_ids.fetch_add(1, Ordering::Acquire);
    let mut sessions = state.sessions.write().unwrap();
    sessions.insert(session_id as u16, Arc::new(Mutex::new(session)));

    //sign and send session id cookie to user
    id.remember(session_id.to_string());
    info!("remembering session");

    let end = std::time::Instant::now();
    println!("{:?}", end - now);

    Ok(HttpResponse::Found()
        .header(
            http::header::LOCATION,
            req.path()["/login".len()..].to_owned(),
        )
        .finish())
}

pub fn make_random_cookie_key() -> [u8; 32] {
    let mut cookie_private_key = [0u8; 32];
    let mut rng = rand::rngs::StdRng::from_entropy();
    rng.fill(&mut cookie_private_key[..]);
    cookie_private_key
}
