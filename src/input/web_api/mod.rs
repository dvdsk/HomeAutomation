use actix_web::web::{Form, Data};
use actix_web_httpauth::extractors::basic::{BasicAuth};
use actix_web_httpauth::headers::www_authenticate::basic::Basic;
use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;
use actix_web::http::StatusCode;

use chrono::{DateTime, NaiveDateTime, Utc};
use std::time::Duration;
use serde::Deserialize;

use actix_web::HttpResponse;

use crate::input::youtube_downloader::FeedbackChannel;
use crate::controller::{Command, Event};
use crate::controller::State as TargetState;
use crate::input::alarms::Alarm;

mod command_logins;
pub mod server;
use server::State;

fn authenticated(auth: BasicAuth) -> bool {

	let username = auth.user_id();
	if let Some(password) = auth.password(){
	command_logins::LIST.iter()
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

//////////////////////// control alarms commands /////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct AlarmDataMinFrom {
	min_till_alarm: String,
}

/*pub fn set_alarm_minutes_from_now(req: HttpRequest, params: Form<AlarmDataMinFrom>, 
       state: Data<State>, auth: BasicAuth) -> HttpResponse {
	if authenticated(auth) {
		//Code to parse alarm time
		dbg!(&params);

		if let Ok(minutes) = params.min_till_alarm.parse(){
			let time = Utc::now() + chrono::Duration::minutes(minutes);

			state.alarms.add_alarm(time).unwrap();
			HttpResponse::Ok().finish()
		} else {
			make_error(StatusCode::INTERNAL_SERVER_ERROR)
		}
	} else {
		make_auth_error()
	}
}*/

#[derive(Deserialize, Debug)]
pub struct AlarmDataUnixTS {
	timestamp: String,
}

pub async fn set_alarm_unix_timestamp(params: Form<AlarmDataUnixTS>, state: Data<State>) -> HttpResponse {
	//Code to parse alarm time
	dbg!(&params);

	if let Ok(ts) = params.timestamp.parse(){
		let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts, 0), Utc);
		dbg!(time);
		dbg!(Utc::now());

		if time>Utc::now() {
			let alarm = Alarm::from(time, Event::Alarm, Some(Duration::from_secs(3600*2)));

			if state.alarms.add_alarm(alarm).await.is_ok() {
				dbg!("done setting alarm");
				HttpResponse::Ok().finish()
			} else {
				dbg!();
				make_error( StatusCode::INTERNAL_SERVER_ERROR)
			}
		} else {
			dbg!();
			make_error(StatusCode::UNPROCESSABLE_ENTITY)
		}
	} else {
		dbg!();
		make_error(StatusCode::INTERNAL_SERVER_ERROR)
	}
}

pub fn list_alarms(state: Data<State>) -> HttpResponse {
	//Code to parse alarm time
	
	let alarms = state.alarms.list();
	let mut list = String::with_capacity(alarms.len()*100);
	for (id,alarm) in alarms {
		list.push_str(&format!("{:x}, {}, {:?}, {:?}",
			id,
			&alarm.time.to_rfc2822(),
			&alarm.action,
			&alarm.expiration,
		));
		list.push_str("\n");
	}
	dbg!(&list);
	HttpResponse::Ok().body(list)
}

//////////////////////// control mpd /////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct SongUrl {
	url: String,
}

pub async fn add_song_from_url(params: Form<SongUrl>, state: Data<State>) -> HttpResponse {
	let url = params.into_inner().url;
	dbg!();

	if state.youtube_dl.add_song_to_queue(url, FeedbackChannel::None)
		.await.is_ok(){
		dbg!();
		
		HttpResponse::Ok().finish()
	} else {
		make_error(StatusCode::INTERNAL_SERVER_ERROR)
	}
}