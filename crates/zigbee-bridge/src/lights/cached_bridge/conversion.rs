pub(crate) fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

pub(crate) fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}

pub(crate) fn temp_to_xy(color_temp: usize) -> (f64, f64) {
    todo!()
}

