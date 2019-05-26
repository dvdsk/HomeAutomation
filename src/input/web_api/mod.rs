extern crate actix_web_httpauth;

use actix_web::Form;
use actix_web::FromRequest;
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::{WWWAuthenticate};
use actix_web::http::StatusCode;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

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

fn make_error(req: &HttpRequest<ServerState>, header: StatusCode) -> HttpResponse {
	req.build_response(StatusCode::INTERNAL_SERVER_ERROR)
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

//////////////////////// control alarms commands /////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct AlarmDataMinFrom {
	min_till_alarm: String,
}

pub fn set_alarm_minutes_from_now((req, params): (HttpRequest<ServerState>, Form<AlarmDataMinFrom>)) -> HttpResponse {
	if authenticated(&req) {
		//Code to parse alarm time
		dbg!(&params);

		if let Ok(minutes) = params.min_till_alarm.parse(){
			let time = Utc::now() + chrono::Duration::minutes(minutes);

			req.state().alarms.add_alarm(time).unwrap();
			HttpResponse::Ok().finish()
		} else {
			make_error(&req, StatusCode::INTERNAL_SERVER_ERROR)
		}
	} else {
		make_auth_error(&req)
	}
}

#[derive(Deserialize, Debug)]
pub struct AlarmDataUnixTS {
	timestamp: String,
}

pub fn set_alarm_unix_timestamp((req, params): (HttpRequest<ServerState>, Form<AlarmDataUnixTS>)) -> HttpResponse {
	//Code to parse alarm time
	dbg!(&params);

	if let Ok(ts) = params.timestamp.parse(){
		let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts, 0), Utc);
		dbg!(time);
		dbg!(Utc::now());

		if time>Utc::now() {
			if req.state().alarms.add_alarm(time).is_ok() {
				dbg!(("done setting alarm"));
				HttpResponse::Ok().finish()
			} else {
				dbg!();
				make_error(&req, StatusCode::INTERNAL_SERVER_ERROR)
			}
		} else {
			dbg!();
			make_error(&req, StatusCode::UNPROCESSABLE_ENTITY)
		}
	} else {
		dbg!();
		make_error(&req, StatusCode::INTERNAL_SERVER_ERROR)
	}
}

pub fn list_alarms(req: &HttpRequest<ServerState>) -> HttpResponse {
	//Code to parse alarm time
	
	let alarms = req.state().alarms.list();
	let mut list = String::with_capacity(alarms.len()*30);
	for alarm in alarms {
		list.push_str(&alarm.to_rfc2822());
		list.push_str("\n");
	}
	dbg!(&list);
	HttpResponse::Ok().body(list)
}