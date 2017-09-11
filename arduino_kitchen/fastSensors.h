#ifndef LOCALSENSORS_H
#define LOCALSENSORS_H

#include <Arduino.h> //needed for Serial.print
#include "encodingScheme.h"
#include "compression.h"
#include "config.h"
#include <Wire.h>
#include "BH1750.h"


//set both pins to locations on a bank that match this mask
constexpr int PIR_SHOES_WEST = 0b00000100;
constexpr int PIR_SHOES_EAST = 0b00001000;
constexpr int PIR_DOOR =       0b00010000;
constexpr int PIR_KITCHEN =    0b00100000;

class FastSensors{

	public:
	FastSensors();
	void readAndEncode(uint8_t buffer[]);
	uint8_t readPIRs();
	void begin();

	private:
	BH1750 lightSens1;
	BH1750 lightSens2;
};	
#endif

