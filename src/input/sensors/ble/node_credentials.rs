use secure_ble::UUID;

//shared
pub const FAST_MEASURMENTS_UUID: UUID = UUID::B128([137, 20, 153, 220, 231, 245, 91, 152, 153, 21, 183, 27, 175, 191, 112, 147]);
pub const SLOW_MEASURMENTS_UUID: UUID = UUID::B128([137, 20, 153, 220, 231, 245, 91, 152, 153, 21, 183, 27, 175, 191, 112, 147]);

//specific
pub mod node_bed {
    pub const KEY: [u8;16] 
        = [66,72,79,15,11,8,27,55,83,75,91,63,21,32,84,87];
    pub const HMAC_KEY: [u8;16] 
        = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    pub const ADRESS: &str
        = "C6:46:56:AC:2C:4C"; //TODO change
}

pub mod node_bathroom {
    pub const KEY: [u8;16] 
        = [66,72,79,15,11,8,27,55,83,75,91,63,21,32,84,87];
    pub const HMAC_KEY: [u8;16] 
        = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    pub const ADRESS: &str
        = "C6:46:56:AC:2C:4C";
}