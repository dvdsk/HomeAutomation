#ifndef HUMIDITYSENSOR_H
#define HUMIDITYSENSOR_H

#include <Arduino.h> //needed for Serial.print


class TempHumid
{
  public:
    TempHumid(int dataPin, int clockPin);
    void readPIR();
    float readTemperatureC();
    float readHumidity();
  private:
    int _dataPin;
    int _clockPin;
    void skipCrcSHT(int _dataPin, int _clockPin);
    void waitForResultSHT(int _dataPin);
    int getData16SHT(int _dataPin, int _clockPin);
    void sendCommandSHT(int _command, int _dataPin, int _clockPin);
    float readTemperatureRaw();
};


//this prevents problems if someone accidently #include's your library twice.
#endif

