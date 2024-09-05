use bme280::i2c::BME280 as Bme280;
use bme280::{self, Measurements};
use hal::{Delay, I2cdev};
use linux_embedded_hal as hal;
use tracing::debug;

use tokio::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use protocol::large_bedroom::desk::Reading as DeskReading;
use protocol::large_bedroom::desk::{Error, SensorError};
use protocol::Reading;

use crate::{send_error, send_reading};

pub fn init() -> Result<Bme280<I2cdev>, Error> {
    let i2c_bus = I2cdev::new("/dev/i2c-1")
        .inspect_err(|e| tracing::error!("Could not open i2c bus: {e}"))
        .map_err(|e| protocol::make_error_string(e))
        .map_err(|e| Error::Setup(SensorError::BmeError(e)))?;

    let mut bme280 = Bme280::new_primary(i2c_bus);
    bme280
        .init(&mut Delay)
        .inspect_err(|e| tracing::error!("Could not init bme280 sensor: {e}"))
        .map_err(|e| protocol::make_error_string(e))
        .map_err(|e| Error::Setup(SensorError::BmeError(e)))?;
    Ok(bme280)
}

pub fn start_monitoring(tx: Sender<Result<Reading, protocol::Error>>) -> Result<(), Error> {
    let mut bme = init()?;

    let mut last_warning = Instant::now() - Duration::from_secs(10_000);
    thread::spawn(move || loop {
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
                send_reading(&tx, DeskReading::Temperature(temperature));
                send_reading(&tx, DeskReading::Humidity(humidity));
                send_reading(&tx, DeskReading::Pressure(pressure));
            }
            Err(e) => {
                if last_warning.elapsed() > Duration::from_secs(300) {
                    last_warning = Instant::now();
                    tracing::error!("Could not read bme280 sensor: {e}");
                }
                let err = protocol::make_error_string(e);
                send_error(&tx, Error::Running(SensorError::BmeError(err)))
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    });

    Ok(())
}
