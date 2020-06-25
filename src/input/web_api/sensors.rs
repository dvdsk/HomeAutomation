use actix_web::web::{Form, Data};
use serde::{Deserialize, Serialize};
use actix_web::HttpResponse;

use super::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Button {
    LampLeft,
    LampMid,
    LampRight,

    DeskLeftMost,
    DeskLeft,
    DeskRight,
    DeskRightMost,

    DeskTop,
    DeskMid,
    DeskBottom,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ButtonPress {
    Long(Button),
    Short(Button),
}

#[derive(Deserialize, Debug)]
pub struct SongUrl {
	url: String,
}

pub fn handle(params: Form<SongUrl>, state: Data<State>) -> HttpResponse {
	let tx = &state.controller_addr;

	HttpResponse::Ok().finish()
}