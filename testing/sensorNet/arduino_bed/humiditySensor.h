#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print
#include "remoteNodes.h"
#include "localSensors.h"

namespace TempHumid
{   
	void requestTemp();
	void requestHumid();

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


const short PIN_TERM_DATA = 0b00000100;
const short PIN_TERM_CLOCK = 0b00000001;
const short PINS_OFF = 0b00000000;


//this prevents problems if someone accidently #include's your library twice.
#endif

