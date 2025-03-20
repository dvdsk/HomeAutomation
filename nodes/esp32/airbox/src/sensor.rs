use bme280_rs::{AsyncBme280, Sample};
use embassy_time::{Delay, Duration, TimeoutError, Timer, WithTimeout};
use esp_hal::gpio::{GpioPin, Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::I2C0;
use esp_hal::time::Rate;
use protocol::button::Press;
use protocol::large_bedroom::airbox::{Device, DeviceError, Error, Reading};
use protocol::{large_bedroom, make_error_string};

use crate::fans::Fans;
use crate::{Queue, wrap_error};

fn wrap_reading(reading: Reading) -> protocol::Reading {
    protocol::Reading::LargeBedroom(large_bedroom::Reading::Airbox(reading))
}

fn next_fan_power(current: u8) -> u8 {
    let next = match current {
        ..15 => 20,
        15..35 => 40,
        35..55 => 60,
        55..75 => 80,
        75..95 => 100,
        95.. => 0,
    };
    defmt::info!("current: {}, next_power: {}", current, next);
    next
}

pub async fn button(pin: GpioPin<10>, queue: &Queue, fans: &Fans<'_>) {
    let mut input = Input::new(pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        // TODO measure press time, deduplicate with bed_lb & bed_sb
        input.wait_for_low().await;
        Timer::after_millis(50).await;
        if input.is_high() {
            continue
        }

        let _ignore_err =
            queue.try_send(Ok(wrap_reading(Reading::Button(Press(1)))));

        let next = next_fan_power(fans.current_power().await);
        if let Err(e) = fans.set_power(next).await {
            defmt::error!("error setting fan power: {}", e);
            let _ignore_err = queue.try_send(Err(e));
        }

        Timer::after_millis(500).await;
    }
}

pub async fn measure(i2c: I2C0, queue: &Queue) {
    defmt::info!("HOI");
    let i2c = match I2c::new(
        i2c,
        Config::default().with_frequency(Rate::from_khz(4)),
    ) {
        Ok(i2c) => i2c,
        Err(e) => {
            defmt::error!("Could not start i2c");
            let report =
                Error::Setup(DeviceError::BmeError(make_error_string(e)));
            let _ignore_err = queue.try_send(Err(wrap_error(report)));
            return;
        }
    }
    .into_async();

    let mut i2c = Some(i2c);
    loop {
        let mut bme = AsyncBme280::new(defmt::unwrap!(i2c.take()), Delay);
        match bme.init().with_timeout(Duration::from_millis(200)).await {
            Ok(Ok(())) => (),
            Ok(Err(e)) => {
                defmt::error!("Could not init bme280: {}", e);
                let report =
                    Error::Setup(DeviceError::BmeError(make_error_string(e)));
                let _ignore_err = queue.try_send(Err(wrap_error(report)));
                i2c = Some(bme.release());
                continue;
            }
            Err(TimeoutError) => {
                defmt::error!("Could not init bme280, timeout");
                let report = Error::SetupTimedOut(Device::Bme280);
                let _ignore_err = queue.try_send(Err(wrap_error(report)));
                i2c = Some(bme.release());
                continue;
            }
        }

        read_every_5_seconds_until_error(queue, &mut bme).await;
        i2c = Some(bme.release());
        Timer::after_secs(5).await;
    }
}

async fn read_every_5_seconds_until_error(
    queue: &Queue,
    bme: &mut AsyncBme280<I2c<'_, esp_hal::Async>, Delay>,
) {
    loop {
        defmt::info!("loop start");
        match bme
            .read_sample()
            .with_timeout(Duration::from_millis(200))
            .await
        {
            Ok(Ok(Sample {
                pressure: Some(p),
                temperature: Some(t),
                ..
            })) => {
                defmt::info!("pressure: {}, temperature: {}", p, t);
                let _ignore_err =
                    queue.try_send(Ok(wrap_reading(Reading::Temperature(t))));
                let _ignore_err =
                    queue.try_send(Ok(wrap_reading(Reading::Pressure(p))));
            }
            Ok(Ok(_)) => {
                defmt::unreachable!("pressure and temperature are enabled")
            }
            Ok(Err(e)) => {
                defmt::error!("Could not read bme280");
                let report =
                    Error::Running(DeviceError::BmeError(make_error_string(e)));
                let _ignore_err = queue.try_send(Err(wrap_error(report)));
                return;
            }
            Err(TimeoutError) => {
                defmt::error!("Could not read bme280, timeout");
                let report = Error::Timeout(Device::Bme280);
                let _ignore_err = queue.try_send(Err(wrap_error(report)));
                return;
            }
        }
        Timer::after_secs(5).await;
    }
}
