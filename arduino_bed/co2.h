#ifndef CO2_H
#define CO2_H

#include <Arduino.h> //needed for Serial.print

namespace Co2
{
		void setup();
		void request();
		void reset();
		bool readyToRead();
		uint16_t readCO2();


		inline byte calculate_checkV(const byte data[9]);
};

#endif

