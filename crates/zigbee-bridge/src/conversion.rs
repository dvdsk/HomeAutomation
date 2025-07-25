use colorimetry::{cct::CCT, xyz::XYZ};

#[must_use]
pub(super) fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

#[must_use]
pub(super) fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}

#[must_use]
pub(super) fn normalize(brightness: u8) -> f64 {
    f64::from(brightness) / 254.
}

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(super) fn denormalize(brightness: f64) -> u8 {
    (brightness * 254.).round() as u8
}

pub(super) fn round_to_half(setpoint: f64) -> f64 {
    (setpoint * 2.).round() / 2.
}

pub(super) fn times_100_int(reference: f64) -> usize {
    (reference * 100.).round() as usize
}

#[allow(clippy::doc_markdown)]
/// 1000K < `color_temp` < 1_000_000K
pub(super) fn temp_to_xy(
    color_temp_k: usize,
    color_deviation: f64,
) -> (f64, f64) {
    #[allow(clippy::cast_precision_loss)]
    let cct = CCT::try_new(color_temp_k as f64, color_deviation).unwrap();
    let xyz: XYZ = cct.try_into().unwrap();

    let (x, y) = xyz_to_xy(xyz);
    (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
}

fn xyz_to_xy(xyz: XYZ) -> (f64, f64) {
    let [x, y, z] = xyz.values();
    (x / (x + y + z), y / (x + y + z))
}
