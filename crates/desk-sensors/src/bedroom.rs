use protocol::Reading;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum Bedroom {
    Large,
    Small,
}

impl Bedroom {
    pub(crate) fn make_temperature_reading(&self, value: f32) -> Reading {
        match self {
            Self::Large => {
                use protocol::large_bedroom::desk::Reading as DeskReading;
                wrap_large(DeskReading::Temperature(value))
            }
            Self::Small => {
                use protocol::small_bedroom::desk::Reading as DeskReading;
                wrap_small(DeskReading::Temperature(value))
            }
        }
    }
    pub(crate) fn make_humidity_reading(&self, value: f32) -> Reading {
        match self {
            Self::Large => {
                use protocol::large_bedroom::desk::Reading as DeskReading;
                wrap_large(DeskReading::Humidity(value))
            }
            Self::Small => {
                use protocol::small_bedroom::desk::Reading as DeskReading;
                wrap_small(DeskReading::Humidity(value))
            }
        }
    }
    pub(crate) fn make_pressure_reading(&self, value: f32) -> Reading {
        match self {
            Self::Large => {
                use protocol::large_bedroom::desk::Reading as DeskReading;
                wrap_large(DeskReading::Pressure(value))
            }
            Self::Small => {
                use protocol::small_bedroom::desk::Reading as DeskReading;
                wrap_small(DeskReading::Pressure(value))
            }
        }
    }

    pub(crate) fn make_setup_error(&self, e: heapless::String<200>) -> protocol::Error {
        match self {
            Self::Large => {
                use protocol::large_bedroom::desk;
                let e = desk::SensorError::BmeError(e);
                let e = desk::Error::Setup(e);
                let e = protocol::large_bedroom::Error::Desk(e);
                protocol::Error::LargeBedroom(e)
            }
            Self::Small => {
                use protocol::small_bedroom::desk;
                let e = desk::SensorError::BmeError(e);
                let e = desk::Error::Setup(e);
                let e = protocol::small_bedroom::Error::Desk(e);
                protocol::Error::SmallBedroom(e)
            }
        }
    }

    pub(crate) fn make_run_error(&self, e: heapless::String<200>) -> protocol::Error {
        match self {
            Self::Large => {
                use protocol::large_bedroom::desk;
                let e = desk::SensorError::BmeError(e);
                let e = desk::Error::Running(e);
                let e = protocol::large_bedroom::Error::Desk(e);
                protocol::Error::LargeBedroom(e)
            }
            Self::Small => {
                use protocol::small_bedroom::desk;
                let e = desk::SensorError::BmeError(e);
                let e = desk::Error::Running(e);
                let e = protocol::small_bedroom::Error::Desk(e);
                protocol::Error::SmallBedroom(e)
            }
        }
    }
}

fn wrap_small(reading: protocol::small_bedroom::desk::Reading) -> Reading {
    use protocol::small_bedroom::Reading::Desk;
    use protocol::Reading::SmallBedroom;
    SmallBedroom(Desk(reading))
}
fn wrap_large(reading: protocol::large_bedroom::desk::Reading) -> Reading {
    use protocol::large_bedroom::Reading::Desk;
    use protocol::Reading::LargeBedroom;
    LargeBedroom(Desk(reading))
}
