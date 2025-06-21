use std::{collections::HashMap, ops::Range};

use tracing::warn;

use crate::LIGHT_MODELS;

use super::color_correction;

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
    /// 600 lm
    /// LCT001
    HueGen1,
    /// 9290012573A Hue white and color ambiance E26/E27/E14
    /// 800 lm
    /// LCT007
    HueGen2,
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
            | Model::HueGen1
            | Model::HueGen2
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
            Model::HueGen1 | Model::HueGen2 => 2000..6500,
            Model::TradfriOther(_) => 2200..4000,
            Model::HueOther(_) => 2200..6500,
            Model::Other(_) => 2200..4000,
        }
    }

    pub(super) fn color_deviation(&self, color_temp: usize) -> f64 {
        let temp = color_temp.saturating_add_signed(
            color_correction::temp_correction(self, color_temp),
        );
        color_correction::blackbody_deviation(self, temp)
    }

    pub(crate) fn is_hue(&self) -> bool {
        match self {
            Model::TradfriCandle
            | Model::TradfriE27
            | Model::TradfriGU10
            | Model::TradfriE14Color
            | Model::TradfriE14White
            | Model::TradfriOther(_) => false,
            Model::HueGen1 | Model::HueGen2 | Model::HueOther(_) => true,
            // I guess we default to sending payloads separately
            Model::Other(_) => false,
        }
    }
}
