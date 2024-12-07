use std::{collections::HashMap, ops::Range};

use tracing::warn;

use crate::LIGHT_MODELS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Model {
    /// LED2107C4 TRADFRI bulb E14, white spectrum, candle, opal, 470 lm
    TradfriCandle,
    /// LED2109G6 TRADFRI bulb E26/E27, color/white spectrum, globe, opal, 800/806/810 lm
    TradfriE27,
    /// LED1923R5 TRADFRI bulb GU10, color/white spectrum, 345/380 lm
    TradfriGU10,
    /// LED2111G6 TRADFRI bulb E14, color/white spectrum, globe, opal, 806 lm
    TradfriE14Color,
    /// LED2101G4 TRADFRI bulb E12/E14, white spectrum, globe, opal, 450/470 lm
    TradfriE14White,
    /// 9290012573A Hue white and color ambiance E26/E27/E14
    HueGen4,
    #[allow(unused)]
    TradfriOther(String),
    #[allow(unused)]
    HueOther(String),
    Other(String),
}

impl Model {
    pub(super) fn from_light(name: &str) -> Self {
        let map = HashMap::from(LIGHT_MODELS);
        match map.get(name) {
            Some(model) => model.clone(),
            None => {
                warn!("No model known for light {name}");
                Model::Other(String::from("Unknown"))
            }
        }
    }

    pub(super) fn supports_xy(&self) -> bool {
        #[allow(clippy::match_same_arms)] // clearer comment
        match self {
            Model::TradfriE27
            | Model::TradfriE14Color
            | Model::HueGen4
            | Model::TradfriGU10 => true,
            Model::TradfriCandle | Model::TradfriE14White => false,
            // We assume no so that things at least don't break
            Model::TradfriOther(_) | Model::HueOther(_) | Model::Other(_) => {
                false
            }
        }
    }

    pub(super) fn temp_k_range(&self) -> Range<usize> {
        match self {
            Model::TradfriCandle | Model::TradfriGU10 => 2200..4000,
            Model::TradfriE27 | Model::TradfriE14Color => 1780..4000,
            Model::TradfriE14White => 2000..4000,
            Model::HueGen4 => 2000..6500,
            Model::TradfriOther(_) => 2200..4000,
            Model::HueOther(_) => 2200..6500,
            Model::Other(_) => 2200..4000,
        }
    }

    #[allow(unused)]
    pub(super) fn color_deviation(&self, color_temp: usize) -> (usize, f64) {
        match self {
            Model::TradfriCandle => todo!(),
            Model::TradfriE27 => todo!(),
            Model::TradfriE14Color => todo!(),
            Model::HueGen4 => todo!(),
            Model::TradfriOther(_) => todo!(),
            Model::HueOther(_) => todo!(),
            Model::Other(_) => todo!(),
            Model::TradfriGU10 => todo!(),
            Model::TradfriE14White => todo!(),
        }
    }

    #[allow(unused)]
    // Value for actual color temp (after temp correction)
    fn blackbody_table(&self) -> HashMap<usize, f64> {
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

    #[allow(unused)]
    // Value to add to requested color temp
    fn temp_table(&self) -> HashMap<usize, isize> {
        match self {
            Model::TradfriCandle => {
                vec![(2200, 30), (2700, 20), (4000, 50)]
            }
            Model::TradfriE27 => vec![(2200, 110), (2700, 30), (4000, -110)],
            Model::TradfriE14Color => vec![(2200, 70), (2700, -40), (4000, 20)],
            Model::HueGen4 | Model::HueOther(_) => {
                vec![(2200, 2), (2700, 5), (4000, -10), (5500, 45), (6500, 5)]
            }
            Model::TradfriOther(_) => todo!(),
            Model::Other(_) => todo!(),
            Model::TradfriGU10 => todo!(),
            Model::TradfriE14White => todo!(),
        }
        .into_iter()
        .collect()
    }

    pub(crate) fn is_hue(&self) -> bool {
        match self {
            Model::TradfriCandle
            | Model::TradfriE27
            | Model::TradfriGU10
            | Model::TradfriE14Color
            | Model::TradfriE14White
            | Model::TradfriOther(_) => false,
            Model::HueGen4 | Model::HueOther(_) => true,
            // I guess we default to sending payloads separately
            Model::Other(_) => false,
        }
    }
}
