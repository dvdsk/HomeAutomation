use actix_web_httpauth::extractors::basic::{BasicAuth};
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpRequest};
use actix_web::web::{Form, Data};

mod logins;
mod alarms;
mod music;
pub mod sensors;
mod commands;

pub mod server;
use server::State;

fn authenticated(auth: BasicAuth) -> bool {

	let username = auth.user_id();
	if let Some(password) = auth.password(){
	logins::LIST.iter()
		.filter(|x| x.username==username)
		.filter(|x| x.password==password)
		.next().is_some()
	} else {false }
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
