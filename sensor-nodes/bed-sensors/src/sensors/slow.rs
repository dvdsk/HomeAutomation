use defmt::unwrap;
use embassy_futures::join::join5;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;
use embassy_time::{with_timeout, Duration};

use mhzx::MHZ;
use protocol::large_bedroom::bed::Reading;

use bosch_bme680::MeasurementData;
use sps30_async as sps30;

use crate::channel::Queues;
use crate::error_cache::Error;

use super::retry::{Bme680Driver, Driver, Sht31Driver, Sps30Driver};

use super::concrete_types::ConcreteRx as Rx;
use super::concrete_types::ConcreteTx as Tx;

type Signal = embassy_sync::signal::Signal<NoopRawMutex, ()>;
#[inline(always)]
async fn measure_and_send_on_signal<D, F>(mut driver: D, send: F, signal: &Signal)
where
    D: Driver,
    F: Fn(Result<D::Measurement, Error>) -> (),
{
    let timeout = Duration::from_secs(15);
    loop {
        signal.wait().await;
        let try_measure = Driver::try_measure(&mut driver);
        let res = match with_timeout(timeout, try_measure).await {
            Ok(res) => res,
            Err(_) => Err(Error::Timeout(driver.device())),
        };
        send(res);
    }
}

async fn order_measurements_every_period(signals: &[Signal]) {
    loop {
        for signal in signals {
            signal.signal(());
        }
        Timer::after_secs(5).await;
    }
}

#[inline(always)]
pub async fn read(
    sht: Sht31Driver<'_>,
    bme: Bme680Driver<'_>,
    mhz: MHZ<Tx<'_>, Rx<'_>>,
    sps: Sps30Driver<'_>,
    publish: &'_ Queues,
) {
    let signals = [const { Signal::new() }; 4];

    join5(
        order_measurements_every_period(&signals),
        measure_and_send_on_signal(sht, |res| publish_sht_result(res, &publish), &signals[0]),
        measure_and_send_on_signal(bme, |res| publish_bme_result(res, &publish), &signals[1]),
        measure_and_send_on_signal(mhz, |res| publish_mhz_result(res, &publish), &signals[2]),
        measure_and_send_on_signal(sps, |res| publish_sps_result(res, &publish), &signals[3]),
    )
    .await;
}

fn publish_sps_result(sps_res: Result<sps30::Measurement, Error>, publish: &Queues) {
    match sps_res {
        Ok(sps30::Measurement {
            mass_pm1_0,
            mass_pm2_5,
            mass_pm4_0,
            mass_pm10,
            number_pm1_0,
            number_pm2_5,
            number_pm4_0,
            number_pm10,
            typical_particle_size,
            ..
            // mass_pm0_5,
        }) => {
            publish.send_p0(Reading::MassPm1_0(mass_pm1_0));
            publish.send_p0(Reading::MassPm2_5(mass_pm2_5));
            publish.send_p0(Reading::MassPm4_0(mass_pm4_0));
            publish.send_p0(Reading::MassPm10(mass_pm10));
            // publish.send_p0(Reading::MassPm0_5(mass_pm0_5));
            publish.send_p0(Reading::NumberPm1_0(number_pm1_0));
            publish.send_p0(Reading::NumberPm2_5(number_pm2_5));
            publish.send_p0(Reading::NumberPm4_0(number_pm4_0));
            publish.send_p0(Reading::NumberPm10(number_pm10));
            publish.send_p0(Reading::TypicalParticleSize(typical_particle_size));
        }
        Err(err) => publish.queue_error(err),
    }
}

fn publish_mhz_result(mhz_res: Result<mhzx::Measurement, Error>, publish: &Queues) {
    match mhz_res {
        Ok(mhzx::Measurement { co2, .. }) => {
            publish.send_p0(Reading::Co2(co2));
        }
        Err(err) => {
            publish.queue_error(err);
        }
    }
}

fn publish_sht_result(sht_res: Result<sht31::prelude::Reading, Error>, publish: &Queues) {
    match sht_res {
        Ok(sht31::Reading {
            temperature,
            humidity,
        }) => {
            publish.send_p0(Reading::Temperature(temperature));
            publish.send_p0(Reading::Humidity(humidity));
        }
        Err(err) => publish.queue_error(err),
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
