use bme280::i2c::BME280 as Bme280;
use bme280::{self, Measurements};
use governor::clock::{QuantaClock, QuantaInstant};
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use hal::{Delay, I2cdev};
use linux_embedded_hal as hal;
use protocol::{ErrorString, Reading};
use tracing::debug;

use std::num::NonZeroU32;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::bedroom;

pub fn init() -> Result<Bme280<I2cdev>, ErrorString> {
    let i2c_bus = I2cdev::new("/dev/i2c-1")
        .inspect_err(|e| tracing::error!("Could not open i2c bus: {e}"))
        .map_err(|e| protocol::make_error_string(e))?;

    let mut bme280 = Bme280::new_primary(i2c_bus);
    bme280
        .init(&mut Delay)
        .inspect_err(|e| tracing::error!("Could not init bme280 sensor: {e}"))
        .map_err(|e| protocol::make_error_string(e))?;
    Ok(bme280)
}

pub fn start_monitoring(tx: Sender<Result<Reading, protocol::Error>>, room: bedroom::Bedroom) {
    let quota =
        Quota::per_hour(NonZeroU32::new(4).unwrap()).allow_burst(NonZeroU32::new(20).unwrap());
    let mut err_report_limiter = governor::RateLimiter::direct(quota);
    let mut bme = None;

    thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));

        if bme.is_none() {
            bme = match init() {
                Ok(driver) => Some(driver),
                Err(e) => {
                    tracing::error!("Could not setup bme280 sensor: {e}");
                    tx.blocking_send(Err(room.make_setup_error(e)))
                        .expect("main should not return or panic");
                    continue;
                }
            };
        }

        if let Some(mut driver) = bme.take() {
            match driver.measure(&mut Delay) {
                Ok(measurements) => {
                    bme = Some(driver);
                    send_measurements(measurements, &tx, room);
                }
                Err(e) => {
                    send_error(e, &tx, room, &mut err_report_limiter);
                    break;
                }
            }
        }
    });
}

fn send_error(
    e: bme280::Error<linux_embedded_hal::I2CError>,
    tx: &Sender<Result<Reading, protocol::Error>>,
    room: bedroom::Bedroom,
    limiter: &mut RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
) {
    if limiter.check().is_err() {
        return;
    }

    tracing::error!("Could not read bme280 sensor: {e}");
    let e = protocol::make_error_string(e);
    tx.blocking_send(Err(room.make_run_error(e)))
        .expect("main should not return or panic");
}

fn send_measurements(
    measurements: Measurements,
    tx: &Sender<Result<Reading, protocol::Error>>,
    room: bedroom::Bedroom,
) {
    debug!("got measurements: {measurements:?}");

    let Measurements {
        temperature,
        pressure,
        humidity,
    } = measurements;

    tx.blocking_send(Ok(room.make_temperature_reading(temperature)))
        .expect("main should not return or panic");
    tx.blocking_send(Ok(room.make_humidity_reading(humidity)))
        .expect("main should not return or panic");
    tx.blocking_send(Ok(room.make_pressure_reading(pressure)))
        .expect("main should not return or panic");
}
