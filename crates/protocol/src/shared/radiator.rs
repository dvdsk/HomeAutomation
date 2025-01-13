use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading;
#[cfg(feature = "alloc")]
use crate::reading::LabelFormatter;

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Copy,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
)]
#[strum_discriminants(derive(Hash))]
#[repr(u8)]
pub enum Source {
    Manual = 0,
    Schedule = 1,
    External = 2,
}

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct SetByLabelFormatter;

#[cfg(feature = "alloc")]
impl LabelFormatter for SetByLabelFormatter {
    fn format(&self, value: f64, info: &reading::Info) -> String {
        match value {
            0.0 => "Manual",
            1.0 => "Schedule",
            2.0 => "External",
            _ => "SetpointLabelFormatterError",
        }
        .to_string()
    }

    fn box_clone(&self) -> Box<dyn LabelFormatter> {
        Box::new(Self)
    }
}

