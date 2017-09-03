#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print
#include "config.h"

namespace TempHumid
{   
	void requestTemp();
	void requestHumid();
	void reset();

	bool readyToRead();

	float readTemperatureC();
  float readHumidity(float tempC);

	//funct used by above funct
  void sendCommandSHT(int _command);  
  void startWaitForResultSHT();
  int getData16SHT();
  void skipCrcSHT();
  
  float readTemperatureRaw();                                 
};


constexpr int TERM_DATA = A0;
constexpr int TERM_CLOCK = A1;

constexpr short PIN_TERM_DATA =  0b00000001; //SDA nano A0
constexpr short PIN_TERM_CLOCK = 0b00000010; //SCL nano A1
constexpr short PINS_OFF = 0b00000000;


//this prevents problems if someone accidently #include's your library twice.
#endif
