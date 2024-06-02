use hueclient::{CommandLight, LightState};

//adaptation from philipshue LightState that adds some
//values and removes unused
#[derive(Debug, Clone, PartialEq)]
pub struct Lamp {
    pub on: bool,
    pub bri: u8,
    pub hue: Option<u16>,
    pub sat: Option<u8>,
    pub xy: Option<(f32, f32)>,
    pub ct: Option<u16>,
}

impl Lamp {
    pub(crate) fn light_cmd(&self) -> CommandLight {
        let mut light_cmd = CommandLight::default().with_bri(self.bri);
        light_cmd.on = Some(self.on);

        let light_cmd = if let Some((x, y)) = self.xy {
            light_cmd.with_xy(x, y)
        } else if let Some(ct) = self.ct {
            light_cmd.with_ct(ct)
        } else {
            unreachable!("lamp must have ct or xy set");
        };
        light_cmd
    }
}

impl From<&LightState> for Lamp {
    fn from(state: &LightState) -> Self {
        Lamp {
            on: state.on,
            bri: state.bri.unwrap_or(0),
            hue: state.hue,
            sat: state.sat,
            xy: state.xy,
            ct: state.ct,
        }
    }
}

#[allow(dead_code)]
fn gamma_correct(mut x: f32) -> f32 {
    if x > 0.04045 {
        x = (x + 0.055) / (1f32 + 0.055);
        x.powf(2.4)
    } else {
        x / 12.92
    }
}

//r,g,b between 0 and one
//https://gist.github.com/popcorn245/30afa0f98eea1c2fd34d
#[allow(dead_code)]
pub fn xy_from_rgb(rgb: (f32, f32, f32)) -> (f32, f32) {
    let (r, g, b) = rgb;
    let r = gamma_correct(r);
    let g = gamma_correct(g);
    let b = gamma_correct(b);

    let xyz_x = r * 0.649926 + g * 0.103455 + b * 0.197109;
    let xyz_y = r * 0.234327 + g * 0.743075 + b * 0.022598;
    let xyz_z = g * 0.053077 + b * 1.035763;

    let hue_x = xyz_x / (xyz_x + xyz_y + xyz_z);
    let hue_y = xyz_y / (xyz_x + xyz_y + xyz_z);

    //TODO color gamut triangle stuff for finding closest valid value

    (hue_x, hue_y)
}
