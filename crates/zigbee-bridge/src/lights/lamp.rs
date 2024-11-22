pub(super) use model::Model;
pub(super) use property::{Property, PropertyDiscriminants};

use self::property::{bri_is_close, color_is_close};
use tracing::{error, instrument};

use super::conversion::temp_to_xy;

mod model;
mod property;

#[derive(Clone, Copy, Debug)]
enum Color {
    TempK(usize),
    XY((f64, f64)),
}

impl Color {
    fn xy_from_temp(temp: usize) -> Color {
        Color::XY(temp_to_xy(temp))
    }
}

// TODO: some way to enforce read-only (thus known-updatable-only) fields?
#[derive(Default, Clone, Debug)]
pub(super) struct Lamp {
    model: Option<Model>,
    is_online: Option<bool>,
    brightness: Option<f64>,
    color: Option<Color>,
    is_on: Option<bool>,
    color_temp_startup: property::ColorTempStartup,
}

impl Lamp {
    #[instrument]
    pub(super) fn changes_relative_to(&self, other: &Self) -> Vec<Property> {
        let mut res = Vec::new();

        // Ignore model and is_online, because they are read-only

        if let Some(bri_self) = self.brightness {
            if other
                .brightness
                .is_none_or(|bri_other| !bri_is_close(bri_other, bri_self))
            {
                res.push(Property::Brightness(bri_self));
            }
        }

        if let Some(color_self) = self.color {
            let log_err = |err| {
                error!("Comparing incompatible lamp states, defaulting to same\nSelf: {self:?}\nOther: {other:?}\nErr: {err}");
            };

            let color_is_close = match other.color {
                None => false,
                Some(color_other) => color_is_close(color_other, color_self)
                    .unwrap_or_else(|err| {
                        log_err(err);
                        true
                    }),
            };

            if !color_is_close {
                match color_self {
                    Color::XY(xy) => res.push(Property::ColorXY(xy)),
                    Color::TempK(temp) => res.push(Property::ColorTempK(temp)),
                }
            }
        }

        if let Some(on_self) = self.is_on {
            if other.is_on.is_none_or(|on_other| on_other != on_self) {
                res.push(Property::On(on_self));
            }
        }

        if self.color_temp_startup != other.color_temp_startup {
            res.push(Property::ColorTempStartup(self.color_temp_startup));
        }

        res
    }

    pub(super) fn apply(&mut self, change: Property) {
        match change {
            Property::On(is_on) => self.is_on = Some(is_on),
            Property::Brightness(bri) => self.brightness = Some(bri),
            Property::ColorTempK(temp) => {
                // if we know the model, we know how to apply temp
                if let Some(model) = &self.model {
                    if model.supports_xy() {
                        self.color = Some(Color::xy_from_temp(temp));
                    } else {
                        let range = model.temp_k_range();
                        let temp = temp.clamp(range.start, range.end);
                        self.color = Some(Color::TempK(temp))
                    }
                }
            }
            Property::ColorXY(xy) => {
                // don't apply xy to unknown or non-color lamp
                if let Some(model) = &self.model {
                    if model.supports_xy() {
                        self.color = Some(Color::XY(xy))
                    }
                }
            }
            Property::ColorTempStartup(behaviour) => {
                self.color_temp_startup = behaviour
            }
            Property::Online(is_online) => self.is_online = Some(is_online),
        }
    }

    pub(crate) fn all_as_changes(&self) -> Vec<Property> {
        let mut list = Vec::new();

        // Ignore model and is_online, because they are read-only

        if let Some(val) = (&self).brightness {
            list.push(Property::Brightness(val));
        }
        if let Some(val) = &self.color {
            match val {
                Color::XY(xy) => list.push(Property::ColorXY(*xy)),
                Color::TempK(temp) => list.push(Property::ColorTempK(*temp)),
            }
        }
        if let Some(val) = (&self).is_on {
            list.push(Property::On(val));
        }
        list.push(Property::ColorTempStartup((&self).color_temp_startup));
        list
    }

    pub(crate) fn add_model_from(mut self, other: &Lamp) -> Self {
        if let Some(model) = &other.model {
            self.model = Some(model.clone());
        }
        self
    }

    pub(crate) fn set_model(&mut self, model: Model) {
        self.model = Some(model);
    }
}
