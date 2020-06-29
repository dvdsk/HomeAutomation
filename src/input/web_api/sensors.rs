//use bytes::Bytes;
use actix_web::web::{Data, Bytes};
use serde::{Deserialize, Serialize};
use actix_web::HttpResponse;

use super::*;
use crate::controller;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SensorValue {
    ButtonPress(ButtonPress),
    Temperature(f32),
    Humidity(f32),
    Pressure(f32),
}

pub fn handle(body: Bytes, state: Data<State>) -> HttpResponse {
    let res = bincode::deserialize::<SensorValue>(&body[..]);
    match res {
        Err(e) => error!("deserialize sensorval failed: {:?}",e),
        Ok(v) => {
            //temp to ButtonPress
            match v {
                SensorValue::ButtonPress(p) => {
                    let event = match p{
                        ButtonPress::Long(b) => controller::Event::PressLong(b),
                        ButtonPress::Short(b) => controller::Event::PressShort(b),
                    };
                    state.controller_addr.send(event).unwrap();
                }
                _ => (),
            }
        }
    }

	HttpResponse::Ok().finish()
}