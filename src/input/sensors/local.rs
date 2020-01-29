extern crate linux_embedded_hal as hal;
extern crate bme280;

use hal::{Delay, I2cdev};
use bme280::BME280;
use super::fields::Field;

pub const START_ENCODE: u8 = 10*8;
pub const TEMPERATURE: Field<f32> = Field {
	offset: START_ENCODE, //bits
	length: 13, //bits (max 32 bit variables)
	
	decode_scale: 0.009999999776482582,
	decode_add: -20.0f32,
};
pub const HUMIDITY: Field<f32> = Field {
	offset: START_ENCODE
		+TEMPERATURE.length,
	length: 14,

	decode_scale: 0.00800000037997961,
	decode_add: 0.0,
};
pub const PRESSURE: Field<f32> = Field {
	offset: START_ENCODE
		+TEMPERATURE.length
		+HUMIDITY.length,
	length: 19,

	decode_scale: 0.18000000715255738,
	decode_add: 30000.0,
};
pub const STOP_ENCODE: usize = (START_ENCODE
	+TEMPERATURE.length
	+HUMIDITY.length
	+PRESSURE.length) as usize;

pub fn init() -> BME280<I2cdev, Delay> {
	// using Linux I2C Bus #1 in this example
	let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
	// initialize the BME280 using the primary I2C address 0x77
	let mut bme280 = BME280::new_primary(i2c_bus, Delay);
	// initialize the sensor
	bme280.init().unwrap();
	bme280
}

pub fn measure_and_record(bme: &mut BME280<I2cdev, Delay> ) -> (f32, f32, f32) {
	// measure temperature, pressure, and humidity
	let measurements = bme.measure().unwrap();

	(measurements.humidity, measurements.temperature, measurements.pressure)
}

