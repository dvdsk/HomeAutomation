#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print
#include "remoteNodes.h"
#include "localSensors.h"

//type declaration for saving space and sanity in passing functions

class TempHumid
{
  public:
    void setup(int dataPin, int clockPin, RemoteNodes* radio_, LocalSensors* sensors_, uint16_t* slowData_);
    void readPIR();
    
		void getTempHumid();		

  private:
    int _dataPin;
    int _clockPin;

    void skipCrcSHT();
    int getData16SHT(int _dataPin, int _clockPin);
    void sendCommandSHT(int _command, int _dataPin, int _clockPin);  
    
    float readTemperatureRaw();                                 
    void waitForResultSHT(int _dataPin);

		float readTemperatureC();
    float readHumidity(float tempC);

		uint16_t* slowData;
		RemoteNodes* radio;
		LocalSensors* local;
};

static const unsigned char PIRDATA2 = 202;//TODO might need removing when fast polling data
//function is implemented

const short PIN_TERM_DATA = 0b00000100;
const short PIN_TERM_CLOCK = 0b00000001;
const short PINS_OFF = 0b00000000;


//this prevents problems if someone accidently #include's your library twice.
#endif

