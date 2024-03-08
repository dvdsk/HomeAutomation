/// will store a short history of the last sensor values
/// useful for debouncing events (someone passing in front 
/// light sensor vs a cloud)

use sensor_value::SensorValue;

// struct History<T> {
//     last_values: Vec<T>,
// }

pub struct Environment {
    // light: History<f32>,
}

impl Environment {
    pub fn update(&self, _: SensorValue) {}
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Environment {
            // light: History {
                // last_values: Vec::new(),
            // },
        }
    }
}
