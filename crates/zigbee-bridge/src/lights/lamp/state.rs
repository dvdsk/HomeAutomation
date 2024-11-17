use tracing::instrument;

use super::property::{
    bri_is_close, temp_is_close, xy_is_close, ColorTempStartup, LampProperty,
};
use super::Change;
use super::Model;
use crate::lights::conversion::temp_to_xy;

#[derive(Debug, Clone)]
pub(super) struct LampState {
    pub(super) brightness: Option<f64>,
    pub(super) color_temp_k: Option<usize>,
    pub(super) color_xy: Option<(f64, f64)>,
    pub(super) on: Option<bool>,
    pub(super) color_temp_startup: ColorTempStartup,
}

impl Default for LampState {
    fn default() -> Self {
        Self {
            brightness: None,
            color_temp_k: None,
            color_xy: None,
            on: None,
            // Settings, will not be updated from MQTT state message
            // and will never trigger a publish
            color_temp_startup: ColorTempStartup::Previous,
        }
    }
}

impl LampState {
    #[instrument]
    pub(super) fn changes_relative_to(
        &self,
        other: &Self,
        model: Option<&Model>,
    ) -> Vec<LampProperty> {
        let mut res = Vec::new();
        if let Some(bri_self) = self.brightness {
            if other
                .brightness
                .is_none_or(|bri_other| !bri_is_close(bri_other, bri_self))
            {
                res.push(LampProperty::Brightness(bri_self));
            }
        }

        if let Some(temp_self) = self.color_temp_k {
            if other
                .color_temp_k
                .is_none_or(|temp_other| !temp_is_close(temp_other, temp_self))
            {
                res.push(LampProperty::ColorTempK(temp_self));
            }
        }

        if model.is_some_and(Model::is_color_lamp) {
            if let Some(xy_self) = self.color_xy {
                if other
                    .color_xy
                    .is_none_or(|xy_other| !xy_is_close(xy_other, xy_self))
                {
                    res.push(LampProperty::ColorXY(xy_self));
                }
            }
        }

        if let Some(on_self) = self.on {
            if other.on.is_none_or(|on_other| on_other != on_self) {
                res.push(LampProperty::On(on_self));
            }
        }

        if self.color_temp_startup != other.color_temp_startup {
            res.push(LampProperty::ColorTempStartup(self.color_temp_startup));
        }

        res
    }

    pub(crate) fn apply(self, change: Change) -> LampState {
        let mut new_state = self.clone();
        match change {
            Change::On(on) => new_state.on = Some(on),
            Change::Brightness(bri) => new_state.brightness = Some(bri),
            Change::ColorTemp(temp) => {
                new_state.color_temp_k = Some(temp);
                new_state.color_xy = Some(temp_to_xy(temp));
            }
            Change::ColorXy(xy) => new_state.color_xy = Some(xy),
        }
        new_state
    }

    pub(crate) fn property_list(&self) -> Vec<LampProperty> {
        // we do not send color xy as the lamp might not support it
        // if it does then property_list is never called but an exact
        // diff between the current and need state is send

        let mut list = Vec::new();
        if let Some(val) = self.brightness {
            list.push(LampProperty::Brightness(val));
        }
        if let Some(val) = self.color_temp_k {
            list.push(LampProperty::ColorTempK(val));
        }
        if let Some(val) = self.on {
            list.push(LampProperty::On(val));
        }
        list.push(LampProperty::ColorTempStartup(self.color_temp_startup));
        list
    }
}
