//FOR REMOTE ARDUINO 1 (ARDUINO NANO), using termpin's 3 en 2 (data, sck) (PD3 en PD2)
#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print
#include "RF24.h"

class TempHumid
{
  public:
    TempHumid(int dataPin, int clockPin);
    void readPIR();
    float readTemperatureC(void (*f1)(byte sendBuffer[5]), byte sendBuffer[5], RF24& radio);// can radio be const?
    float readHumidity(float tempC, void (*f1)(byte sendBuffer[5]), byte sendBuffer[5], RF24& radio);
  private:
    int _dataPin;
    int _clockPin;
    void skipCrcSHT(int _dataPin, int _clockPin);
    int getData16SHT(int _dataPin, int _clockPin);
    void sendCommandSHT(int _command, int _dataPin, int _clockPin);  
    float readTemperatureRaw(void (*f1)(byte sendBuffer[5]), byte sendBuffer[5], RF24& radio);
    void waitForResultSHT(int _dataPin, void (*f1)(byte sendBuffer[5]), byte sendBuffer[5], RF24& radio);
};

const short PIN_TERM_DATA = 0b00001000;
const short PIN_TERM_CLOCK= 0b00000100;
const short PINS_OFF = 0b00000000;


//this prevents problems if someone accidently #include's your library twice.
#endif

