use bosch_bme680::Bme680;
use embassy_time::Delay;
use max44009::Max44009;
use nau7802_async::Nau7802;
use protocol::large_bedroom::bed::Device;
use sht31::mode::SingleShot;
use sht31::SHT31;

use super::super::concrete_types::ConcreteSharedI2c;
use super::ReInitableDriver;
use crate::error_cache::Error;
use crate::sensors::concrete_types::ConcreteBlockingI2c;

pub type Nau7802DriverBlocking<'a> = ReInitOnErrorDriver<Nau7802<ConcreteBlockingI2c<'a>, Delay>>;
pub type Nau7802Driver<'a> = ReInitOnErrorDriver<Nau7802<ConcreteSharedI2c<'a>, Delay>>;
pub type Max44Driver<'a> = ReInitOnErrorDriver<Max44009<ConcreteSharedI2c<'a>>>;
pub type Bme680Driver<'a> = ReInitOnErrorDriver<Bme680<ConcreteSharedI2c<'a>, Delay>>;
pub type Sht31Driver<'a> = ReInitOnErrorDriver<SHT31<SingleShot, ConcreteSharedI2c<'a>>>;

enum State<D> {
    Ready { driver: D },
    Uninit,
}

pub struct ReInitOnErrorDriver<D>
where
    D: ReInitableDriver,
{
    state: State<D>,
    parts: D::Parts,
    device: Device,
}

impl<D: ReInitableDriver> super::Driver for ReInitOnErrorDriver<D> {
    type Measurement = D::Measurement;

    #[inline(always)]
    async fn try_measure(&mut self) -> Result<D::Measurement, Error> {
        self.try_measure().await
    }

    #[inline(always)]
    fn device(&self) -> Device {
        self.device.clone()
    }
}

impl<D> ReInitOnErrorDriver<D>
where
    D: ReInitableDriver,
{
    #[inline(always)]
    pub fn new(parts: D::Parts, device: Device) -> Self {
        Self {
            state: State::Uninit,
            parts,
            device,
        }
    }

    #[inline(always)]
    pub async fn try_measure(&mut self) -> Result<D::Measurement, Error> {
        let mut owned_self = State::Uninit;
        core::mem::swap(&mut owned_self, &mut self.state);
        let (mut new_self, res) = advance_state(owned_self, &self.parts).await;
        core::mem::swap(&mut new_self, &mut self.state);
        res
    }
}

#[inline(always)]
async fn advance_state<D: ReInitableDriver>(
    state: State<D>,
    parts: &D::Parts,
) -> (State<D>, Result<D::Measurement, Error>) {
    match state {
        State::Ready {
            mut driver,
            // make a trait over driver that has a measure method
        } => match driver.measure().await {
            Ok(val) => {
                let new_state = State::Ready { driver };
                (new_state, Ok(val))
            }
            Err(err) => {
                let new_state = State::Uninit;
                (new_state, Err(Error::Running(err)))
            }
        },
        State::Uninit => {
            match D::init(parts.clone()).await {
                Ok(mut driver) => {
                    // uses driver::measure again, make that return a SensorError
                    match driver.measure().await {
                        Ok(val) => {
                            let new_state = State::Ready { driver };
                            (new_state, Ok(val))
                        }
                        Err(err) => {
                            let new_state = State::Uninit;
                            let err = Error::Running(err);
                            (new_state, Err(err))
                        }
                    }
                }
                Err(err) => {
                    // drivers might contain blocking code in their
                    // error path, this makes sure the executor
                    // will not block on retrying.
                    embassy_time::Timer::after_millis(250).await;
                    let new_state = State::Uninit;
                    let err = Error::Setup(err);
                    (new_state, Err(err))
                }
            }
        }
    }
}
