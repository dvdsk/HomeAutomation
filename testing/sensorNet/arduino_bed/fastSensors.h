#ifndef LOCALSENSORS_H
#define LOCALSENSORS_H

#include <Arduino.h> //needed for Serial.print
#include "encodingScheme.h"
#include "compression.h"

//set both pins to locations on a bank that match this mask
constexpr int PIR_SOUTH = 0b01000000;
constexpr int PIR_NORTH = 0b00100000;

void readAndEncode(uint8_t buffer[]);
void readPIRs();
void readLight();
#endif

