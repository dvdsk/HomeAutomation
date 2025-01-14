use std::collections::BTreeMap;

use super::Model;

pub(super) fn temp_correction(model: &Model, color_temp: usize) -> usize {
    interpolate(color_temp, temp_table(model)).round() as usize
}

pub(super) fn blackbody_deviation(model: &Model, color_temp: usize) -> f64 {
    interpolate(color_temp, blackbody_table(model))
}

// Value for actual color temp (after temp correction)
fn blackbody_table(model: &Model) -> BTreeMap<usize, f64> {
    match model {
        // 105.455.00 data
        Model::TradfriCandle => vec![
            (2170, 0.0028),
            (2391, 0.0018),
            (2683, 0.0006),
            (2990, 0.0006),
            (3193, 0.0012),
            (3590, 0.003),
            (3791, 0.0036),
            (3951, 0.0046),
        ],
        // 604.391.68 data
        Model::TradfriE27 => vec![
            (1964, -0.003),
            (2090, 0.0014),
            (2668, 0.0015),
            (3046, 0.001),
            (3499, 0.0015),
            (3605, -0.0011),
            (4098, -0.0017),
            (4110, -0.0009),
            (4250, 0.0007),
        ],
        // 204.391.94 data
        Model::TradfriE14Color => vec![
            (1841, -0.0032),
            (2055, -0.0034),
            (2120, 0.0012),
            (2738, -0.0005),
            (3070, 0.0004),
            (3467, 0.0045),
            (3555, 0.00),
            (3981, 0.0009),
            (4113, 0.0037),
        ],
        // A19 Color 9.5W data
        Model::HueGen4 | Model::HueOther(_) => vec![
            (1998, 0.0005),
            (2197, 0.00000),
            (2519, -0.0004),
            (2695, -0.0007),
            (2849, -0.0011),
            (3358, -0.0012),
            (3864, -0.012),
            (4010, -0.0009),
            (5455, -0.0005),
            (6495, 0.0014),
        ],
        Model::TradfriOther(_) => todo!(),
        Model::Other(_) => todo!(),
        Model::TradfriGU10 => todo!(),
        Model::TradfriE14White => todo!(),
    }
    .into_iter()
    .collect()
}

// Value to add to requested color temp
fn temp_table(model: &Model) -> BTreeMap<usize, f64> {
    match model {
        Model::TradfriCandle => {
            vec![(2200, 30.), (2700, 20.), (4000, 50.)]
        }
        Model::TradfriE27 => vec![(2200, 110.), (2700, 30.), (4000, -110.)],
        Model::TradfriE14Color => {
            vec![(2200, 70.), (2700, -40.), (4000, 20.)]
        }
        Model::HueGen4 | Model::HueOther(_) => {
            vec![
                (2200, 2.),
                (2700, 5.),
                (4000, -10.),
                (5500, 45.),
                (6500, 5.),
            ]
        }
        Model::TradfriOther(_) => todo!(),
        Model::Other(_) => todo!(),
        Model::TradfriGU10 => todo!(),
        Model::TradfriE14White => todo!(),
    }
    .into_iter()
    .collect()
}

fn interpolate(temp: usize, table: BTreeMap<usize, f64>) -> f64 {
    if let Some(val) = table.get(&temp) {
        return *val;
    }

    let larger = table
        .iter()
        .filter(|(k, _)| *k > &temp)
        .min_by_key(|(k, _)| *k);
    let smaller = table
        .iter()
        .filter(|(k, _)| *k < &temp)
        .max_by_key(|(k, _)| *k);

    if let (Some(smaller), Some(larger)) = (smaller, larger) {
        // Linear interpolation
        return temp as f64 * (larger.1 - smaller.1)
            / (*larger.0 as f64 - *smaller.0 as f64);
    }

    match smaller.or(larger) {
        Some(nearest) => *nearest.1,
        None => 0.0,
    }
}
