use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Model {
    TradfriCandle,
    TradfriE27,
    TradfriE14,
    HueGen4,
    TradfriOther(String),
    HueOther(String),
    Other(String),
}

impl Model {
    pub(crate) fn is_color_lamp(&self) -> bool {
        #[allow(clippy::match_same_arms)] // clearer comment
        match self {
            Model::TradfriE27 | Model::TradfriE14 | Model::HueGen4 => true,
            Model::TradfriCandle => false,
            // We assume no so that things at least don't break
            Model::TradfriOther(_) | Model::HueOther(_) | Model::Other(_) => {
                false
            }
        }
    }

    pub(crate) fn color_deviation(&self, color_temp: usize) -> (usize, f64) {
        match self {
            Model::TradfriCandle => todo!(),
            Model::TradfriE27 => todo!(),
            Model::TradfriE14 => todo!(),
            Model::HueGen4 => todo!(),
            Model::TradfriOther(_) => todo!(),
            Model::HueOther(_) => todo!(),
            Model::Other(_) => todo!(),
        }
    }

    // Value for actual color temp (after temp correction)
    pub(crate) fn blackbody_table(&self) -> HashMap<usize, f64> {
        match self {
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
            Model::TradfriE14 => vec![
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
        }
        .into_iter()
        .collect()
    }

    // Value to add to requested color temp
    pub(crate) fn temp_table(&self) -> HashMap<usize, isize> {
        match self {
            Model::TradfriCandle => {
                vec![(2200, 30), (2700, 20), (4000, 50)]
            }
            Model::TradfriE27 => vec![(2200, 110), (2700, 30), (4000, -110)],
            Model::TradfriE14 => vec![(2200, 70), (2700, -40), (4000, 20)],
            Model::HueGen4 | Model::HueOther(_) => {
                vec![(2200, 2), (2700, 5), (4000, -10), (5500, 45), (6500, 5)]
            }
            Model::TradfriOther(_) => todo!(),
            Model::Other(_) => todo!(),
        }
        .into_iter()
        .collect()
    }
}
