use bme280::i2c::BME280 as Bme280;
use bme280::{self, Measurements};
use hal::{Delay, I2cdev};
use linux_embedded_hal as hal;
use protocol::{ErrorString, Reading};
use tracing::debug;

use std::thread;
use std::time::{Duration, Instant};
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
    let mut last_warning = Instant::now() - Duration::from_secs(10_000);
    thread::spawn(move || loop {
        let mut bme = match init() {
            Ok(driver) => driver,
            Err(e) => {
                tracing::error!("Could not setup bme280 sensor: {e}");
                tx.blocking_send(Err(room.make_setup_error(e)))
                    .expect("main should not return or panic");
                continue;
            }
        };

        loop {
            match bme.measure(&mut Delay) {
                Ok(
                    m @ Measurements {
                        temperature,
                        pressure,
                        humidity,
                        ..
                    },
                ) => {
                    debug!("got measurements: {m:?}");
                    tx.blocking_send(Ok(room.make_temperature_reading(temperature)))
                        .expect("main should not return or panic");
                    tx.blocking_send(Ok(room.make_humidity_reading(humidity)))
                        .expect("main should not return or panic");
                    tx.blocking_send(Ok(room.make_pressure_reading(pressure)))
                        .expect("main should not return or panic");
                }
                Err(e) => {
                    if last_warning.elapsed() > Duration::from_secs(300) {
                        last_warning = Instant::now();
                        tracing::error!("Could not read bme280 sensor: {e}");
                    }
                    let e = protocol::make_error_string(e);
                    tx.blocking_send(Err(room.make_run_error(e)))
                        .expect("main should not return or panic");
                    break;
                }
            }

            std::thread::sleep(Duration::from_secs(5));
        }
        std::thread::sleep(Duration::from_secs(20));
    });
}
