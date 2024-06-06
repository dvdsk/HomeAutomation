use core::fmt;

use defmt::unwrap;
use embassy_futures::{join, yield_now};
use embassy_time::{with_timeout, Delay, Duration, Timer};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use mhzx::MHZ;
use protocol::large_bedroom::bed::{Device, Reading};

use bosch_bme680::{Bme680, MeasurementData};
use protocol::make_error_string;
use sht31::mode::{Sht31Measure, Sht31Reader, SingleShot};
use sht31::SHT31;
use sps30_async as sps30;
use sps30_async::Sps30;

use crate::error_cache::SensorError;
use crate::error_cache::Error;
use crate::channel::Queues;

const SPS30_UART_BUF_SIZE: usize = 100;
const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

pub async fn read<I2C, TX1, RX1, TX2, RX2>(
    mut sht: SHT31<SingleShot, I2C>,
    mut bme: Bme680<I2C, impl DelayNs>,
    mut mhz: MHZ<TX1, RX1>,
    mut sps: Sps30<SPS30_DRIVER_BUF_SIZE, TX2, RX2, Delay>,
    publish: &Queues,
) where
    I2C: I2c,
    I2C::Error: defmt::Format,
    TX1: embedded_io_async::Write,
    TX1::Error: defmt::Format,
    RX1: embedded_io_async::Read,
    RX1::Error: defmt::Format,
    TX2: embedded_io_async::Write,
    TX2::Error: defmt::Format,
    RX2: embedded_io_async::Read,
    RX2::Error: defmt::Format,
{
    // sht works in two steps
    //  - send measure command before sleep
    //  - then read
    if let Err(err) = sht.measure().await {
        let err = SensorError::Sht31(err);
        let err = Error::Running(err);
        publish.queue_error(err)
    }
    Timer::after_secs(1).await;

    loop {
        defmt::info!("this is where we break");
        let sht_read = with_timeout(Duration::from_millis(100), sht.read());
        yield_now().await;
        let bme_measure = bme.measure();
        yield_now().await;
        let mhz_measure = with_timeout(Duration::from_millis(100), mhz.read_co2());
        yield_now().await;
        let sps_measure = with_timeout(Duration::from_millis(100), sps.read_measurement());
        yield_now().await; // with this yield enabled the program hangs within 12 seconds
        let (bme_res, sht_res, mhz_res, sps_res) =
            join::join4(bme_measure, sht_read, mhz_measure, sps_measure).await;
        // let (bme_res, sht_res, mhz_res) = join::join3(bme_measure, sht_read, mhz_measure).await;
        yield_now().await;

        publish_bme_result(bme_res, publish);
        yield_now().await;
        publish_sht_result(sht_res, publish);
        yield_now().await;
        publish_mhz_result(mhz_res, publish);
        yield_now().await;
        publish_sps_result(sps_res, publish);

        // sht works in two steps
        //  - send measure command before sleep
        //  - then read
        if let Err(err) = sht.measure().await {
            let err = SensorError::Sht31(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Timer::after_secs(1).await;
    }
}

fn publish_sps_result<TxError, RxError>(
    sps_res: Result<
        Result<Option<sps30::Measurement>, sps30::Error<TxError, RxError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) where
    TxError: fmt::Debug + defmt::Format,
    RxError: fmt::Debug + defmt::Format,
{
    match sps_res {
        Ok(Ok(Some(sps30::Measurement {
            mass_pm1_0,
            mass_pm2_5,
            mass_pm4_0,
            mass_pm10,
            mass_pm0_5,
            number_pm1_0,
            number_pm2_5,
            number_pm4_0,
            number_pm10,
            typical_particle_size,
        }))) => {
            publish.send_p0(Reading::MassPm1_0(mass_pm1_0));
            publish.send_p0(Reading::MassPm2_5(mass_pm2_5));
            publish.send_p0(Reading::MassPm4_0(mass_pm4_0));
            publish.send_p0(Reading::MassPm10(mass_pm10));
            publish.send_p0(Reading::MassPm0_5(mass_pm0_5));
            publish.send_p0(Reading::NumberPm1_0(number_pm1_0));
            publish.send_p0(Reading::NumberPm2_5(number_pm2_5));
            publish.send_p0(Reading::NumberPm4_0(number_pm4_0));
            publish.send_p0(Reading::NumberPm10(number_pm10));
            publish.send_p0(Reading::TypicalParticleSize(typical_particle_size));
        }
        Ok(Ok(None)) => {
            defmt::todo!("no idea when we hit this");
        }
        Ok(Err(err)) => {
            let err = SensorError::Sps30(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Sps30);
            publish.queue_error(err)
        }
    }
}

fn publish_mhz_result<TxError, RxError>(
    mhz_res: Result<
        Result<mhzx::Measurement, mhzx::Error<TxError, RxError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) where
    TxError: fmt::Debug + defmt::Format,
    RxError: fmt::Debug + defmt::Format,
{
    match mhz_res {
        Ok(Ok(mhzx::Measurement { co2, .. })) => {
            publish.send_p0(Reading::Co2(co2));
        }
        Ok(Err(err)) => {
            let err = SensorError::Mhz14(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Mhz14);
            publish.queue_error(err)
        }
    }
}

fn publish_sht_result(
    sht_res: Result<Result<sht31::prelude::Reading, sht31::SHTError>, embassy_time::TimeoutError>,
    publish: &Queues,
) {
    match sht_res {
        Ok(Ok(sht31::Reading {
            temperature,
            humidity,
        })) => {
            publish.send_p0(Reading::Temperature(temperature));
            publish.send_p0(Reading::Humidity(humidity));
        }
        Ok(Err(err)) => {
            let err = SensorError::Sht31(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
        Err(_timeout) => {
            let err = Error::Timeout(Device::Sht31);
            publish.queue_error(err)
        }
    }
}

fn publish_bme_result<E: fmt::Debug>(
    bme_res: Result<MeasurementData, bosch_bme680::BmeError<E>>,
    publish: &Queues,
) {
    match bme_res {
        Ok(MeasurementData {
            pressure,
            gas_resistance,
            ..
        }) => {
            let gas_resistance = unwrap!(gas_resistance); // sensor is on
            publish.send_p0(Reading::GassResistance(gas_resistance));
            publish.send_p0(Reading::Pressure(pressure));
        }
        Err(err) => {
            let err = SensorError::Bme680(err);
            let err = Error::Running(err);
            publish.queue_error(err)
        }
    }
}
