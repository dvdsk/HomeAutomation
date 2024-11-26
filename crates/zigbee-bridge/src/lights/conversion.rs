use colorimetry::{cct::CCT, xyz::XYZ};

// Deviation from black body, must be between -0.05 and 0.05
const DUV_IKEA: f64 = -0.002;

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

#[allow(clippy::doc_markdown)]
/// 1000K < `color_temp` < 1_000_000K
pub(super) fn temp_to_xy(color_temp: usize) -> (f64, f64) {
    // TODO: use table from black body deviation measurements
    #[allow(clippy::cast_precision_loss)]
    let cct = CCT::try_new(color_temp as f64, DUV_IKEA).unwrap();
    let xyz: XYZ = cct.try_into().unwrap();

    let (x, y) = xyz_to_xy(xyz);
    (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))

}

fn xyz_to_xy(xyz: XYZ) -> (f64, f64) {
    let [x, y, z] = xyz.values();
    (x / (x + y + z), y / (x + y + z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_to_xy_works() {
        dbg!(temp_to_xy(2200));
        dbg!(temp_to_xy(2500));
        dbg!(temp_to_xy(3000));
        dbg!(temp_to_xy(4000));
    }
}
