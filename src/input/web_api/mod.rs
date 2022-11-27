use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse};

mod commands;
mod music;
mod wakeup;

pub mod server;
use server::State;

fn authenticated(auth: BasicAuth) -> bool {
    let username = auth.user_id() else { return false };
    let Some(password) = auth.password() else { return false };
    username == env!("CMD_API_USERNAME") && password == env!("COMMAND_API_PASSWORD")
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
