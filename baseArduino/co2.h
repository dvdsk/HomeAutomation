#ifndef CO2_H
#define CO2_H

#include <Arduino.h> //needed for Serial.print

class Co2
{
	public:
		void setup(uint16_t* slowData_);
		void rqCO2();
		void readCO2();
	private:
		uint16_t* slowData;
		inline byte calculate_checkV(const byte data[9]);
};

#endif

