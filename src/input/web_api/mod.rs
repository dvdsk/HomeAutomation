use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse};

use crate::PasswordDatabase;

mod commands;
mod music;
mod wakeup;

pub mod server;
use server::State;

fn authenticated(auth: BasicAuth, pass_db: &PasswordDatabase) -> bool {
    let username = auth.user_id();
    let Some(password) = auth.password() else {
        return false;
    };
    pass_db
        .verify_password(username.as_bytes(), password.as_bytes())
        .is_ok()
}

fn make_auth_error() -> HttpResponse {
    let challenge = Basic::with_realm("Restricted area");
    HttpResponse::Unauthorized()
        .set(WwwAuthenticate(challenge))
        .finish()
}

fn make_error(statuscode: StatusCode) -> HttpResponse {
    HttpResponse::build(statuscode).finish()
}
