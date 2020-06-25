use actix_web::web::Data;
use actix_web_httpauth::extractors::basic::{BasicAuth};

use actix_web::HttpResponse;

use crate::controller::{Command, Event};
use crate::controller::State as TargetState;
use super::*;

///////////////////// lamp commands ///////////////////////////////

pub fn toggle(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsToggle)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

pub fn dim(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsDim)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

pub fn dimmest(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsDimmest)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

pub fn normal(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsDay)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

pub fn evening(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsEvening)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

pub fn night(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::LampsNight)).unwrap();
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}

//////////////////////// go to state commands /////////////////////////////////

pub fn lightloop(state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		state.controller_addr.send(Event::Command(Command::ChangeState(TargetState::LightLoop))).unwrap(); //TODO FIXME
		HttpResponse::Ok().finish()
	} else {
		make_auth_error()
	}
}