use defmt::unwrap;
use embassy_futures::join::join5;
use embassy_futures::select;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_time::{with_timeout, Duration};

use protocol::large_bedroom::bed::Reading;

use crate::channel::Queues;
use sensors::measurements;
use sensors::{Bme680Driver, Driver, Sht31Driver, Sps30Driver};
use sensors::{Error, MhzDriver};

type Signal = embassy_sync::signal::Signal<NoopRawMutex, ()>;

/// Measure when signal is received or control the device's
/// affector when it is ordered
#[inline(always)]
async fn measure_and_control<D, F>(
    mut driver: D,
    order: &Channel<NoopRawMutex, D::Affector, 1>,
    send: F,
    signal: &Signal,
) where
    D: Driver,
    F: Fn(Result<D::Measurement, Error>) -> (),
{
    let timeout = Duration::from_secs(15);
    loop {
        match select::select(signal.wait(), order.receive()).await {
            select::Either::First(_) => {
                let try_measure = Driver::try_measure(&mut driver);
                let res = match with_timeout(timeout, try_measure).await {
                    Ok(res) => res,
                    Err(_) => Err(Error::Timeout(driver.device())),
                };
                send(res);
            }
            select::Either::Second(order) => {
                let affect = Driver::affect(&mut driver, order);
                let res = match with_timeout(timeout, affect).await {
                    Ok(Ok(_)) => continue,
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(Error::Timeout(driver.device())),
                };
                send(res);
            }
        }
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

pub(crate) struct Drivers<'a> {
    pub sht: Sht31Driver<'a>,
    pub bme: Bme680Driver<'a>,
    pub mhz: MhzDriver<'a>,
    pub sps: Sps30Driver<'a>,
}

pub(crate) struct DriverOrderers {
    pub sht: Channel<NoopRawMutex, (), 1>,
    pub bme: Channel<NoopRawMutex, (), 1>,
    pub mhz: Channel<NoopRawMutex, (), 1>,
    pub sps: Channel<NoopRawMutex, (), 1>,
}
impl DriverOrderers {
    pub(crate) fn new() -> Self {
        Self {
            sht: Channel::new(),
            bme: Channel::new(),
            mhz: Channel::new(),
            sps: Channel::new(),
        }
    }
}

#[inline(always)]
pub async fn read(
    drivers: Drivers<'_>,
    orderers: &DriverOrderers,
    publish: &'_ Queues,
) {
    let signals = [const { Signal::new() }; 4];

    join5(
        order_measurements_every_period(&signals),
        measure_and_control(
            drivers.sht,
            &orderers.sht,
            |res| publish_sht_result(res, &publish),
            &signals[0],
        ),
        measure_and_control(
            drivers.bme,
            &orderers.bme,
            |res| publish_bme_result(res, &publish),
            &signals[1],
        ),
        measure_and_control(
            drivers.mhz,
            &orderers.mhz,
            |res| publish_mhz_result(res, &publish),
            &signals[2],
        ),
        measure_and_control(
            drivers.sps,
            &orderers.sps,
            |res| publish_sps_result(res, &publish),
            &signals[3],
        ),
    )
    .await;
}

fn publish_sps_result(
    sps_res: Result<measurements::Sps30, Error>,
    publish: &Queues,
) {
    match sps_res {
        Ok(measurements::Sps30 {
            mass_pm1_0,
            mass_pm2_5,
            mass_pm4_0,
            mass_pm10,
            number_pm0_5,
            number_pm1_0,
            number_pm2_5,
            number_pm4_0,
            number_pm10,
            typical_particle_size,
            ..
        }) => {
            publish.send_p0(Reading::MassPm1_0(mass_pm1_0));
            publish.send_p0(Reading::MassPm2_5(mass_pm2_5));
            publish.send_p0(Reading::MassPm4_0(mass_pm4_0));
            publish.send_p0(Reading::MassPm10(mass_pm10));
            publish.send_p0(Reading::NumberPm0_5(number_pm0_5));
            publish.send_p0(Reading::NumberPm1_0(number_pm1_0));
            publish.send_p0(Reading::NumberPm2_5(number_pm2_5));
            publish.send_p0(Reading::NumberPm4_0(number_pm4_0));
            publish.send_p0(Reading::NumberPm10(number_pm10));
            publish
                .send_p0(Reading::TypicalParticleSize(typical_particle_size));
        }
        Err(err) => publish.queue_error(err),
    }
}

fn publish_mhz_result(
    mhz_res: Result<measurements::Mhz, Error>,
    publish: &Queues,
) {
    match mhz_res {
        Ok(measurements::Mhz { co2, .. }) => {
            publish.send_p0(Reading::Co2(co2));
        }
        Err(err) => {
            publish.queue_error(err);
        }
    }
}

fn publish_sht_result(
    sht_res: Result<measurements::Sht31, Error>,
    publish: &Queues,
) {
    match sht_res {
        Ok(measurements::Sht31 {
            temperature,
            humidity,
        }) => {
            publish.send_p0(Reading::Temperature(temperature));
            publish.send_p0(Reading::Humidity(humidity));
        }
        Err(err) => publish.queue_error(err),
    }
}

fn publish_bme_result(
    bme_res: Result<measurements::Bme, Error>,
    publish: &Queues,
) {
    match bme_res {
        Ok(measurements::Bme {
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
