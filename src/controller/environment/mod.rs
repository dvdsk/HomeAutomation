use sensor_value::SensorValue;

struct History<T> {
    last_values: Vec<T>,
}

pub struct Environment {
    light: History<f32>,
}

impl Environment {
    pub fn update(&self, s: SensorValue) {}
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Environment {
            light: History {
                last_values: Vec::new(),
            },
        }
    }
}
