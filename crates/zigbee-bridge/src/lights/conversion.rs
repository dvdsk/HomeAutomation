#[must_use]
pub fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

#[must_use]
pub fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}

#[must_use]
pub fn normalize(brightness: u8) -> f64 {
    f64::from(brightness) / 254.
}

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn denormalize(brightness: f64) -> u8 {
    (brightness * 254.).round() as u8
}

pub(crate) fn temp_to_xy(color_temp: usize) -> (f64, f64) {
    (1.0, 1.0)
}
