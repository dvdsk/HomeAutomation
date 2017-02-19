#ifndef CO2_H
#define CO2_H

#include <Arduino.h> //needed for Serial.print

class Co2
{
	public:
		Co2();
		void rqCO2();
		int readCO2();
	private:
		inline byte calculate_checkV(const byte data[9]);
};

#endif

