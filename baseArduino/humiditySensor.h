#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print


class TempHumid
{
  public:
    TempHumid(int dataPin, int clockPin);
    void readPIR();
    float readTemperatureC(void (*f1)(void), void (*f2)(void), void (*f3)(void));
    float readHumidity(float tempC, void (*f1)(void), void (*f2)(void), void (*f3)(void));
  private:
    int _dataPin;
    int _clockPin;
    void skipCrcSHT(int _dataPin, int _clockPin);
    int getData16SHT(int _dataPin, int _clockPin);
    void sendCommandSHT(int _command, int _dataPin, int _clockPin);  
    float readTemperatureRaw(void (*f1)(void), void (*f2)(void), void (*f3)(void));
    void waitForResultSHT(int _dataPin, void (*f1)(void), void (*f2)(void), void (*f3)(void));
};


//this prevents problems if someone accidently #include's your library twice.
#endif

