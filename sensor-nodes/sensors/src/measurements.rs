// cannot get to this type through trait yet:
// https://github.com/rust-lang/rust/issues/86935
// so this is an alternative

pub use bosch_bme680::MeasurementData as Bme;
pub use sps30_async::Measurement as Sps30;
pub use mhzx::Measurement as Mhz;
pub use sht31::Reading as Sht31;
