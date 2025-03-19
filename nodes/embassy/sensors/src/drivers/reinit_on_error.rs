use protocol::Device;

use crate::errors::Error;
use crate::{Driver, ReInitableDriver};

pub struct ReInitOnErrorDriver<D>
where
    D: ReInitableDriver,
{
    state: State<D>,
    parts: D::Parts,
    device: Device,
}

enum State<D> {
    Ready { driver: D },
    Uninit,
}

impl<D: ReInitableDriver> Driver for ReInitOnErrorDriver<D> {
    type Measurement = D::Measurement;
    type Affector = ();

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
