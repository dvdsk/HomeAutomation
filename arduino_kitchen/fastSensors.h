#ifndef LOCALSENSORS_H
#define LOCALSENSORS_H

#include <Arduino.h> //needed for Serial.print
#include "encodingScheme.h"
#include "compression.h"
#include "config.h"
#include <Wire.h>
#include "BH1750.h"


//set both pins to locations on a bank that match this mask
constexpr int PIR_SHOES_WEST = 0b00000100; //D2
constexpr int PIR_SHOES_EAST = 0b00001000; //D3
constexpr int PIR_DOOR =       0b00010000; //D4
constexpr int PIR_KITCHEN =    0b00100000; //D5

void readAndEncode(uint8_t buffer[]);
uint8_t readPIRs();
void configure_fast();

	
#endif

