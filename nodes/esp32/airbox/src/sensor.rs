use bme280_rs::{AsyncBme280, Sample};
use embassy_time::{Delay, Timer};
use esp_hal::gpio::{GpioPin, Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::I2C0;
use protocol::button::Press;
use protocol::large_bedroom::airbox::{DeviceError, Error, Reading};
use protocol::{large_bedroom, make_error_string};

use crate::fans::Fans;
use crate::{Queue, wrap_error};

fn wrap_reading(reading: Reading) -> protocol::Reading {
    protocol::Reading::LargeBedroom(large_bedroom::Reading::Airbox(reading))
}

pub async fn button(pin: GpioPin<10>, queue: &Queue, fans: &Fans) {
    let mut input = Input::new(pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        // TODO measure press time, deduplicate with bed_lb & bed_sb
        input.wait_for_low().await;
        queue
            .send(Ok(wrap_reading(Reading::Button(Press(1)))))
            .await;

        let next_power = (fans.current_power().await + 20).next_multiple_of(20);
        let next_power = if next_power > 100 { 0 } else { next_power };

        if let Err(e) = fans.set_power(next_power).await {
            queue.send(Err(e)).await;
        }

        Timer::after_millis(100).await;
    }
}

pub async fn measure(i2c: I2C0, queue: &Queue) {
    let i2c = match I2c::new(i2c, Config::default()) {
        Ok(i2c) => i2c,
        Err(e) => {
            let report =
                Error::Setup(DeviceError::BmeError(make_error_string(e)));
            queue.send(Err(wrap_error(report))).await;
            return;
        }
    }
    .into_async();
    let mut bme = AsyncBme280::new(i2c, Delay);

    loop {
        match bme.read_sample().await {
            Ok(Sample {
                pressure: Some(p),
                temperature: Some(t),
                ..
            }) => {
                queue.send(Ok(wrap_reading(Reading::Temperature(t)))).await;
                queue.send(Ok(wrap_reading(Reading::Pressure(p)))).await;
            }
            Ok(_) => {
                defmt::unreachable!("pressure and temperature are enabled")
            }
            Err(e) => {
                let report =
                    Error::Running(DeviceError::BmeError(make_error_string(e)));
                queue.send(Err(wrap_error(report))).await;
            }
        }
    }
}
