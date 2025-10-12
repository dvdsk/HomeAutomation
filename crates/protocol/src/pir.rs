#[cfg(feature = "alloc")]
use crate::reading::LabelFormatter;
use postcard::experimental::max_size::MaxSize;

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Copy,
    Debug,
    defmt::Format,
    serde::Serialize,
    serde::Deserialize,
    MaxSize,
    PartialEq,
)]
#[repr(u8)]
pub enum Status {
    OngoingActivity = 2,
    NoActivity = 0,
    /// because the node went offline
    Unknown = 1,
}

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct PirLabelFormatter;

#[cfg(feature = "alloc")]
impl LabelFormatter for PirLabelFormatter {
    fn format(&self, value: f64, _: &crate::reading::Info) -> String {
        match value {
            0.0 => "activity ended",
            1.0 => "node went down mid activity",
            2.0 => "activity started",
            _ => "ButtonLabelFormatter error",
        }
        .to_string()
    }
    fn box_clone(&self) -> Box<dyn LabelFormatter> {
        Box::new(Self)
    }
}
