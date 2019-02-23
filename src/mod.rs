extern crate actix_web_httpauth;
use actix_web::FromRequest;
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::{WWWAuthenticate};
use actix_web::http::StatusCode;

use super::ServerState;
use super::actix_web::{HttpResponse, HttpRequest};
use super::command_server_logins;
use crate::controller::Command;

fn authenticated(req: &HttpRequest<ServerState>) -> bool {
	let mut config = Config::default();
  config.realm("Restricted area");

	if let Ok(auth) = BasicAuth::from_request(&req, &config){
		let username = auth.username();
		if let Some(password) = auth.password(){
		command_server_logins::LIST.into_iter()
			.filter(|x| x.username==username)
			.filter(|x| x.password==password)
			.next().is_some()
		} else {false }
	} else {false }
}

fn makeAuthError(req: &HttpRequest<ServerState>) -> HttpResponse {
	let challenge = Basic {realm: Some("Restricted area".to_string()),};
	req.build_response(StatusCode::UNAUTHORIZED)
	    .set(WWWAuthenticate(challenge))
	    .finish()
}

pub fn toggle(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsToggle).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}

pub fn dim(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsDim).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}

pub fn dimmest(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsDimmest).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}

pub fn normal(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsDay).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}

pub fn evening(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsEvening).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}

pub fn night(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Command::LampsNight).unwrap();
		HttpResponse::Ok().finish()
	} else {
		makeAuthError(req)
	}
}
