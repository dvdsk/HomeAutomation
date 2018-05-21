/*

This is a library for the BH1750FVI Digital Light Sensor
breakout board.

The board uses I2C for communication. 2 pins are required to
interface to the device.

Datasheet:
http://rohmfs.rohm.com/en/products/databook/datasheet/ic/sensor/light/bh1750fvi-e.pdf

Written by Christopher Laws, March, 2013.

*/

#ifndef BH1750_h
#define BH1750_h

#if (ARDUINO >= 100)
#include <Arduino.h>
#else
#include <WProgram.h>
#endif
#include "Wire.h"

//#define BH1750_DEBUG 1

constexpr uint8_t BH1750_I2CADDR_H = 0x5C;
constexpr uint8_t BH1750_I2CADDR_L = 0x23;

// No active state
constexpr uint8_t BH1750_POWER_DOWN = 0x00;

// Wating for measurment command
constexpr uint8_t BH1750_POWER_ON = 0x01;

// Reset data register value - not accepted in POWER_DOWN mode
constexpr uint8_t BH1750_RESET = 0x07;

// Start measurement at 1lx resolution. Measurement time is approx 120ms.
constexpr uint8_t BH1750_CONTINUOUS_HIGH_RES_MODE = 0x10;

// Start measurement at 0.5lx resolution. Measurement time is approx 120ms.
constexpr uint8_t BH1750_CONTINUOUS_HIGH_RES_MODE_2 = 0x11;

// Start measurement at 4lx resolution. Measurement time is approx 16ms.
constexpr uint8_t BH1750_CONTINUOUS_LOW_RES_MODE = 0x13;

// Start measurement at 1lx resolution. Measurement time is approx 120ms.
// Device is automatically set to Power Down after measurement.
constexpr uint8_t BH1750_ONE_TIME_HIGH_RES_MODE = 0x20;

// Start measurement at 0.5lx resolution. Measurement time is approx 120ms.
// Device is automatically set to Power Down after measurement.
constexpr uint8_t BH1750_ONE_TIME_HIGH_RES_MODE_2 = 0x21;

// Start measurement at 1lx resolution. Measurement time is approx 120ms.
// Device is automatically set to Power Down after measurement.
constexpr uint8_t BH1750_ONE_TIME_LOW_RES_MODE = 0x23;

class BH1750 {
 public:
  BH1750(bool highAddr=false, uint8_t mode_=BH1750_CONTINUOUS_HIGH_RES_MODE);
  uint16_t readLightLevel(void);
	void begin();

 private:
	uint8_t addr;
	uint8_t mode;
	void configure(uint8_t mode);
  void write8(uint8_t data);

};

#endif
