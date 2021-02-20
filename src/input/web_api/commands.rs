use actix_web::web::Data;
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web::http::Method;
use actix_web::FromRequest;
use actix_web::web::Bytes;
use actix_web::HttpResponse;

use super::*;
use crate::controller::State as TargetState;
use crate::controller::{Command, Event};

pub async fn tomorrow(state: Data<State>, auth: BasicAuth, req: HttpRequest, body: Bytes) -> HttpResponse {
    if !authenticated(auth) {
        return make_auth_error();
    }

    match *req.method() {
        Method::GET => {
            let tomorrow = state.wakeup.tomorrow();
            let bytes = bincode::serialize(&tomorrow).unwrap();
            HttpResponse::Ok().body(bytes)
        }
        Method::POST => {
            match bincode::deserialize(&body) {
                Ok((hour,min)) => {
                    state.wakeup.set_tomorrow(hour, min).await; //TODO handle result of this
                    HttpResponse::Ok().finish()
                }
                Err(_) => HttpResponse::BadRequest().body("invalid encoding"),
            }
        }
        _ => HttpResponse::BadRequest().finish(),
    }
}

///////////////////// lamp commands ///////////////////////////////
pub fn toggle(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsToggle))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

pub fn dim(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsDim))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

pub fn dimmest(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsDimmest))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

pub fn normal(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsDay))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

pub fn evening(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsEvening))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

pub fn night(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::LampsNight))
            .unwrap();
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}

//////////////////////// go to state commands /////////////////////////////////

pub fn lightloop(state: Data<State>, auth: BasicAuth) -> HttpResponse {
    if authenticated(auth) {
        state
            .controller_addr
            .send(Event::Command(Command::ChangeState(TargetState::LightLoop)))
            .unwrap(); //TODO FIXME
        HttpResponse::Ok().finish()
    } else {
        make_auth_error()
    }
}
