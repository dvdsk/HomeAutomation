pub fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

pub fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}

pub fn normalize(brightness: u8) -> f64 {
    brightness as f64 / 254.
}

pub fn denormalize(brightness: f64) -> u8 {
    (brightness * 254.).round() as u8
}

pub(crate) fn temp_to_xy(color_temp: usize) -> (f64, f64) {
    (1.0, 1.0)
}
