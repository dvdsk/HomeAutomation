use crate::affector;

pub(crate) mod error;
pub(crate) mod sensor;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // can not use Box on embedded
pub enum Msg<const M: usize> {
    Readings(sensor::SensorMessage<M>),
    ErrorReport(error::ErrorReport),
    AffectorList(affector::ListMessage<M>),
}

impl<const M: usize> Msg<M> {
    pub const READINGS: u8 = 1;
    pub const ERROR_REPORT: u8 = 2;
    pub const AFFECTOR_LIST: u8 = 3;

    #[must_use]
    pub fn header(&self) -> u8 {
        let header = match self {
            Msg::Readings(_) => Self::READINGS,
            Msg::ErrorReport(_) => Self::ERROR_REPORT,
            Msg::AffectorList(_) => Self::AFFECTOR_LIST,
        };
        assert_ne!(header, 0, "0 is reserved for cobs encoding");
        header
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeMsgError> {
        let bytes = bytes.as_mut();
        assert!(!bytes.is_empty(), "can not decode nothing (zero bytes)");

        let msg_type = bytes[0];
        let bytes = &mut bytes[1..];

        if msg_type == Self::READINGS {
            Ok(Self::Readings(sensor::SensorMessage::<M>::decode(bytes)?))
        } else if msg_type == Self::ERROR_REPORT {
            Ok(Self::ErrorReport(error::ErrorReport::decode(bytes)?))
        } else if msg_type == Self::AFFECTOR_LIST {
            Ok(Self::AffectorList(affector::ListMessage::<M>::decode(
                bytes,
            )?))
        } else {
            Err(DecodeMsgError::IncorrectMsgType(msg_type))
        }
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = match self {
            Msg::Readings(readings) => readings.encode(),
            Msg::ErrorReport(report) => report.encode(),
            Msg::AffectorList(list) => list.encode(),
        };

        bytes.insert(0, self.header());
        bytes
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum DecodeMsgError {
    #[cfg_attr(feature = "thiserror", error("Could not decode SensorMessage: {0}"))]
    CorruptEncoding(postcard::Error),
    #[cfg_attr(
        feature = "thiserror",
        error("Got an unknown message type, expected zero or one got: {0}")
    )]
    IncorrectMsgType(u8),
}

pub(crate) const fn cobs_overhead(uncobsed_size: usize) -> usize {
    // COBS requires a minimum of 1 byte overhead, and a maximum of ⌈n/254⌉
    // bytes for n data bytes (one byte in 254, rounded up). (wiki)

    // div_ceil does not round up to one for zero, but messages of 0 bytes
    // make no sense
    assert!(uncobsed_size > 0, "does not support zero length messages");
    uncobsed_size.div_ceil(254)
}
