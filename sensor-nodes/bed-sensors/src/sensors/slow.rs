use defmt::unwrap;
use embassy_futures::join;
use embassy_time::{with_timeout, Delay, Duration};
use embassy_time::{Instant, Timer};

use mhzx::MHZ;
use protocol::large_bedroom::bed::{Device, Reading};

use bosch_bme680::MeasurementData;
use sps30_async as sps30;
use sps30_async::Sps30;

use crate::channel::Queues;
use crate::error_cache::Error;
use crate::error_cache::SensorError;

use super::retry::{Bme680Driver, Sht31Driver};
use super::UartError;

const SPS30_UART_BUF_SIZE: usize = 100;
const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

use super::concrete_types::ConcreteRx as Rx;
use super::concrete_types::ConcreteTx as Tx;

#[inline(always)]
pub async fn read(
    mut sht: Sht31Driver<'_>,
    mut bme: Bme680Driver<'_>,
    mut mhz: MHZ<Tx<'_>, Rx<'_>>,
    mut sps: Sps30<SPS30_DRIVER_BUF_SIZE, Tx<'_>, Rx<'_>, Delay>,
    publish: &'_ Queues,
) {
    let mut next_sample = Instant::now();
    loop {
        Timer::at(next_sample).await;
        next_sample = Instant::now() + Duration::from_secs(1);

        let sht_read = with_timeout(Duration::from_millis(100), sht.try_measure());
        let bme_measure = bme.try_measure(); // can not hang
        let mhz_measure = with_timeout(Duration::from_millis(100), mhz.read_co2());
        let sps_measure = with_timeout(Duration::from_millis(100), sps.read_measurement());
        let (bme_res, sht_res, mhz_res, sps_res) =
            join::join4(bme_measure, sht_read, mhz_measure, sps_measure).await;

        publish_bme_result(bme_res, publish);
        publish_sht_result(sht_res, publish);
        publish_mhz_result(mhz_res, publish);
        publish_sps_result(sps_res, publish);
    }
}

fn publish_sps_result(
    sps_res: Result<
        Result<Option<sps30::Measurement>, sps30::Error<UartError, UartError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) {
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

fn publish_mhz_result(
    mhz_res: Result<
        Result<mhzx::Measurement, mhzx::Error<UartError, UartError>>,
        embassy_time::TimeoutError,
    >,
    publish: &Queues,
) {
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
    sht_res: Result<Result<sht31::prelude::Reading, Error>, embassy_time::TimeoutError>,
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
        Ok(Err(err)) => publish.queue_error(err),
        Err(_timeout) => {
            let err = Error::Timeout(Device::Sht31);
            publish.queue_error(err)
        }
    }
}

fn publish_bme_result(bme_res: Result<MeasurementData, Error>, publish: &Queues) {
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
        Err(err) => publish.queue_error(err),
    }
}
