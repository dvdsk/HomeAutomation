use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Button {
}

impl Button {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
    fn dencode(bytes: impl AsRef<[u8]>) -> Result<Self, ()> {
        todo!()
    }
}
