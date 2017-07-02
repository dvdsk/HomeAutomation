#ifndef LOCALSENSORS_H
#define LOCALSENSORS_H

#include <Arduino.h> //needed for Serial.print
#include "config.h"

class LocalSensors
{
	public:
		void setup(uint16_t* fastData_);
		void updateFast_Local();

	private:
		void readPIRs();
		void readLight();
		uint16_t* fastData;
};
#endif

