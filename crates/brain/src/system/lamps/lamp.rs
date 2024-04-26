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
    pub reachable: bool,
}

impl From<&philipshue::hue::LightState> for Lamp {
    fn from(state: &philipshue::hue::LightState) -> Self {
        Lamp {
            on: state.on,
            bri: state.bri,
            hue: state.hue,
            sat: state.sat,
            xy: state.xy,
            ct: state.ct,
            reachable: state.reachable,
        }
    }
}

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
