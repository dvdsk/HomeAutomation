extern crate actix_web_httpauth;
use actix_web::FromRequest;
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::{WWWAuthenticate};
use actix_web::http::StatusCode;

use crate::ServerState;
use crate::actix_web::{HttpResponse, HttpRequest};
use crate::controller::{Command, TargetState, Event};


mod command_logins;

fn authenticated(req: &HttpRequest<ServerState>) -> bool {
	let mut config = Config::default();
  config.realm("Restricted area");

	if let Ok(auth) = BasicAuth::from_request(&req, &config){
		let username = auth.username();
		if let Some(password) = auth.password(){
		command_logins::LIST.into_iter()
			.filter(|x| x.username==username)
			.filter(|x| x.password==password)
			.next().is_some()
		} else {false }
	} else {false }
}

fn make_auth_error(req: &HttpRequest<ServerState>) -> HttpResponse {
	let challenge = Basic {realm: Some("Restricted area".to_string()),};
	req.build_response(StatusCode::UNAUTHORIZED)
	    .set(WWWAuthenticate(challenge))
	    .finish()
}

///////////////////// lamp commands ///////////////////////////////

pub fn toggle(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsToggle)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

pub fn dim(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsDim)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

pub fn dimmest(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsDimmest)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

pub fn normal(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsDay)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

pub fn evening(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsEvening)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

pub fn night(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::LampsNight)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}

//////////////////////// go to state commands /////////////////////////////////

pub fn lightloop(req: &HttpRequest<ServerState>) -> HttpResponse {
	if authenticated(req) {
		req.state().controller_addr.send(Event::Command(Command::ChangeState(TargetState::LightLoop))).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error(req)
	}
}