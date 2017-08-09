#ifndef LOCALSENSORS_H
#define LOCALSENSORS_H

#include <Arduino.h> //needed for Serial.print
#include "encodingScheme.h"
#include "compression.h"
#include "config.h"
#include <Wire.h>

//set both pins to locations on a bank that match this mask
constexpr int PIR_SHOES_WEST = 0b00000100;
constexpr int PIR_SHOES_EAST = 0b00001000;
constexpr int PIR_DOOR =       0b00010000;
constexpr int PIR_KITCHEN =    0b00100000;

void readAndEncode(uint8_t buffer[]);
uint8_t readPIRs();
uint16_t readLight();
void configure_fast();

/*ADDR = ‘H’    ( ADDR ≧  0.7VCC  )  → “1011100“ */
/*ADDR = 'L'    ( ADDR ≦  0.3VCC  )  → “0100011“ */
constexpr uint8_t ADDR_H = 0x5C;
constexpr uint8_t ADDR_L = 0x23;

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

namespace BH1750{
	void configure(const uint8_t mode, const uint8_t addr);
	uint16_t readLightLevel(const uint8_t addr);
}

#endif

