use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse};

mod wakeup;
mod commands;
mod music;

pub mod server;
use server::State;

fn authenticated(auth: BasicAuth) -> bool {
    let username = auth.user_id();
    let password = auth.password() ;
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
