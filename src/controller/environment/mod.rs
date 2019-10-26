
pub struct Environment {
    light: [f32; 100],
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Environment {
            light: [0f32; 100],
        }
    }
}